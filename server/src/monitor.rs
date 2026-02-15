use shared::{TaskStatus, UploadConfig};
use uuid::Uuid;

use crate::{recording::spawn_recorder, state::SharedState};

pub async fn run_monitor(state: SharedState) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
    loop {
        interval.tick().await;

        let downloads = match state.db.get_downloads().await {
            Ok(d) => d,
            Err(e) => {
                tracing::error!("Monitor failed to load downloads from DB: {}", e);
                continue;
            }
        };

        let all_uploads = match state.db.get_uploads().await {
            Ok(u) => u,
            Err(e) => {
                tracing::error!("Monitor failed to load uploads from DB: {}", e);
                vec![]
            }
        };

        for download in downloads {
            tracing::info!("Monitor checking download config...{:?}", download.name);

            let is_busy = state.tasks.iter().any(|r| {
                r.value().url == download.url
                    && matches!(r.value().status, TaskStatus::Recording | TaskStatus::Uploading)
            });
            if is_busy {
                continue;
            }

            state.checking_urls.insert(download.url.clone(), ());
            match state.checker.check_live(&download.url).await {
                Ok(is_live) => {
                    if is_live {
                        tracing::info!("Streamer {} is live, starting recording", download.name);
                        let task_id = Uuid::new_v4().to_string();
                        let filename = format!("{}-{}.mp4", download.name, task_id);

                        let upload_configs: Vec<UploadConfig> = download
                            .linked_upload_ids
                            .iter()
                            .filter_map(|uid| all_uploads.iter().find(|u| &u.id == uid))
                            .map(|u| u.config.clone())
                            .collect();

                        let task = shared::StreamTask {
                            id: task_id.clone(),
                            name: download.name.clone(),
                            url: download.url.clone(),
                            status: TaskStatus::Idle,
                            filename: filename.clone(),
                            upload_configs,
                        };

                        tracing::info!("task info: {:?}", task);

                        let custom_recording_settings = if download.use_custom_recording_settings {
                            download.recording_settings.clone()
                        } else {
                            None
                        };
                        state.tasks.insert(task_id.clone(), task);
                        spawn_recorder(
                            task_id,
                            download.url.clone(),
                            filename,
                            state.clone(),
                            custom_recording_settings,
                        )
                        .await;
                    }
                }
                Err(e) => {
                    tracing::error!("Streamer {} is not live, error: {:?}", download.name, e);
                }
            }
            state.checking_urls.remove(&download.url);
        }
    }
}
