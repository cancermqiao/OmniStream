use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use url::Url;

use super::ResolvedStream;

const API_BASE: &str = "https://m.ximalaya.com";

pub(super) async fn resolve(url: &str) -> Result<Option<ResolvedStream>> {
    let room_id = room_id_from_url(url)?;
    let client =
        Client::builder().user_agent("Mozilla/5.0").timeout(Duration::from_secs(20)).build()?;

    // Query the room's current session rather than a stale liveId in a share link.
    let detail = api_get(&client, "/lamia/v12/live/room", &[("roomId", &room_id)]).await?;
    let Some((live_id, title)) = live_session(&detail)? else {
        return Ok(None);
    };
    let playback = api_get(
        &client,
        "/lamia-authorize-web/play",
        &[("roomId", &room_id), ("liveId", &live_id)],
    )
    .await?;
    let input_url = playback_url(&playback)?;
    Ok(Some(ResolvedStream { input_url, title, direct_input: true }))
}

async fn api_get(client: &Client, path: &str, params: &[(&str, &str)]) -> Result<Value> {
    let mut url = Url::parse(&format!("{API_BASE}{path}"))?;
    url.query_pairs_mut().extend_pairs(params.iter().copied());
    client
        .get(url)
        .header("Referer", "https://m.ximalaya.com/")
        .header("Accept-Encoding", "identity")
        .send()
        .await
        .with_context(|| format!("Ximalaya request failed: {path}"))?
        .error_for_status()
        .with_context(|| format!("Ximalaya HTTP error: {path}"))?
        .json()
        .await
        .with_context(|| format!("Ximalaya returned invalid JSON: {path}"))
}

fn room_id_from_url(raw: &str) -> Result<String> {
    let url = Url::parse(raw)?;
    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
    if host != "ximalaya.com" && !host.ends_with(".ximalaya.com") {
        return Err(anyhow!("Not a Ximalaya URL"));
    }
    let id = url
        .query_pairs()
        .find(|(key, _)| key == "roomId")
        .map(|(_, value)| value.into_owned())
        .or_else(|| {
            url.path().strip_prefix("/live/room/").map(|s| s.trim_end_matches('/').to_owned())
        });
    id.filter(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()) && id != "0")
        .ok_or_else(|| anyhow!("Ximalaya live URL must contain roomId or /live/room/<roomId>"))
}

fn api_data(response: &Value) -> Result<&Value> {
    match response.get("ret").and_then(Value::as_i64) {
        Some(0) => response
            .get("data")
            .filter(|data| !data.is_null())
            .ok_or_else(|| anyhow!("Ximalaya API response missing data")),
        code => Err(anyhow!("Ximalaya API failed: ret={code:?}")),
    }
}

fn live_session(response: &Value) -> Result<Option<(String, Option<String>)>> {
    let record = api_data(response)?
        .get("recordInfo")
        .filter(|value| value.is_object())
        .ok_or_else(|| anyhow!("Ximalaya room response missing recordInfo"))?;
    let status = record
        .get("status")
        .and_then(Value::as_i64)
        .ok_or_else(|| anyhow!("Ximalaya room response missing status"))?;
    if status != 9 {
        return Ok(None);
    }
    let id = record
        .get("id")
        .and_then(Value::as_u64)
        .filter(|id| *id > 0)
        .ok_or_else(|| anyhow!("Ximalaya live room response missing live id"))?;
    let title = record
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned);
    Ok(Some((id.to_string(), title)))
}

fn playback_url(response: &Value) -> Result<String> {
    let data = api_data(response)?;
    // Prefer continuous HTTPS FLV. The HLS CDN can return short-lived edge slices
    // that end or expire while the room is still live.
    for key in ["flvUrls2", "tsUrls2", "tsUrls", "aacUrls", "flvUrls"] {
        if let Some(urls) = data.get(key).and_then(Value::as_array) {
            for raw in urls.iter().filter_map(Value::as_str) {
                if let Ok(url) = Url::parse(raw)
                    && matches!(url.scheme(), "http" | "https")
                    && url.host_str().is_some()
                {
                    return Ok(raw.to_owned());
                }
            }
        }
    }
    Err(anyhow!("Ximalaya live playback response contains no playable HTTP stream"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accepts_share_and_room_links_but_not_album_ids() {
        for url in [
            "https://m.ximalaya.com/gatekeeper/live-room-h5?liveId=123&roomId=412",
            "https://m.ximalaya.com/live/room/412?liveId=123",
            "https://www.ximalaya.com/live/room/412/",
        ] {
            assert_eq!(room_id_from_url(url).unwrap(), "412");
        }
        for url in [
            "https://www.ximalaya.com/album/412",
            "https://m.ximalaya.com/live/room/",
            "https://m.ximalaya.com/live/room/412/other",
            "https://evilximalaya.com/live/room/412",
            "https://m.ximalaya.com/live/room/412?roomId=invalid",
        ] {
            assert!(room_id_from_url(url).is_err());
        }
    }

    #[test]
    fn distinguishes_live_from_replay_and_preannouncement() {
        let response = json!({"ret":0,"data":{"recordInfo":{"status":9,"id":123,"name":" 测试 "}}});
        assert_eq!(live_session(&response).unwrap(), Some(("123".into(), Some("测试".into()))));
        for status in [0, 1, 5] {
            let response = json!({"ret":0,"data":{"recordInfo":{"status":status,"id":123}}});
            assert!(live_session(&response).unwrap().is_none());
        }
        for response in [
            json!({"ret":401}),
            json!({"ret":0,"data":null}),
            json!({"ret":0,"data":{"recordInfo":{}}}),
            json!({"ret":0,"data":{"recordInfo":{"status":9}}}),
        ] {
            assert!(live_session(&response).is_err());
        }
    }

    #[test]
    fn selects_continuous_flv_and_handles_invalid_or_missing_streams() {
        let response = json!({"ret":0,"data":{"flvUrls2":["https://example.com/live.flv"],
            "tsUrls2":["https://example.com/live.m3u8"]}});
        assert_eq!(playback_url(&response).unwrap(), "https://example.com/live.flv");
        let response = json!({"ret":0,"data":{"tsUrls2":["https://example.com/live.m3u8"],
            "aacUrls":["http://example.com/audio.m3u8"]}});
        assert_eq!(playback_url(&response).unwrap(), "https://example.com/live.m3u8");
        let response = json!({"ret":0,"data":{"tsUrls2":[null,"file:///tmp/audio"],
            "flvUrls":["https://example.com/live.flv"]}});
        assert_eq!(playback_url(&response).unwrap(), "https://example.com/live.flv");
        assert!(playback_url(&json!({"ret":0,"data":{}})).is_err());
        assert!(playback_url(&json!({"ret":403})).is_err());
    }
}
