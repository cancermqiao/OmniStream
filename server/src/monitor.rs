use shared::{TaskStatus, UploadConfig};

use crate::{
    state::SharedState,
    task_launcher::{LaunchTaskParams, launch_recording_task},
};

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

                        let upload_configs: Vec<UploadConfig> = download
                            .linked_upload_ids
                            .iter()
                            .filter_map(|uid| all_uploads.iter().find(|u| &u.id == uid))
                            .map(|u| u.config.clone())
                            .collect();

                        let custom_recording_settings = if download.use_custom_recording_settings {
                            download.recording_settings.clone()
                        } else {
                            None
                        };
                        let initial_filename = format!("{}-pending.mp4", download.name);
                        let task = launch_recording_task(
                            state.clone(),
                            LaunchTaskParams {
                                initial_filename,
                                name: download.name.clone(),
                                url: download.url.clone(),
                                upload_configs,
                                custom_recording_settings,
                            },
                        )
                        .await;

                        tracing::info!("task info: {:?}", task);
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
