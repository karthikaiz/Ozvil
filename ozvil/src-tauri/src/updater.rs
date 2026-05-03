use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

/// Background update check — runs once on startup.
/// Shows the built-in Tauri update dialog if a new version is available.
/// Fails silently; network errors do not surface to the user.
pub async fn check_for_update(app: AppHandle) {
    let updater = match app.updater() {
        Ok(u) => u,
        Err(_) => return,
    };

    let update = match updater.check().await {
        Ok(Some(u)) => u,
        _ => return,
    };

    let version = update.version.clone();
    let notes = update.body.clone().unwrap_or_default();

    // Emit to the frontend so the UI can show a non-blocking update banner
    let _ = app.emit(
        "update-available",
        UpdateAvailablePayload {
            version: version.clone(),
            notes: notes.clone(),
        },
    );
}

#[derive(Clone, serde::Serialize)]
struct UpdateAvailablePayload {
    version: String,
    notes: String,
}

/// Tauri command: check for update and return info to the UI
#[tauri::command]
pub async fn check_update_command(
    app: AppHandle,
) -> Result<Option<UpdateInfo>, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    match updater.check().await.map_err(|e| e.to_string())? {
        Some(update) => Ok(Some(UpdateInfo {
            version: update.version.clone(),
            date: update.date.map(|d| d.to_string()),
            notes: update.body.clone().unwrap_or_default(),
            download_url: update.download_url.to_string(),
        })),
        None => Ok(None),
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub date: Option<String>,
    pub notes: String,
    pub download_url: String,
}

/// Tauri command: download and install the pending update
#[tauri::command]
pub async fn install_update_command(app: AppHandle) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    let update = updater
        .check()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No update available".to_string())?;

    // Download with progress events emitted to the frontend
    let app_handle = app.clone();
    update
        .download_and_install(
            |chunk_len, content_len| {
                let _ = app_handle.emit(
                    "update-download-progress",
                    DownloadProgress {
                        chunk_length: chunk_len,
                        content_length: content_len,
                    },
                );
            },
            || {
                let _ = app_handle.emit("update-install-ready", ());
            },
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadProgress {
    chunk_length: usize,
    content_length: Option<u64>,
}
