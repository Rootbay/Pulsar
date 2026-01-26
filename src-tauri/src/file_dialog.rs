use std::path::{PathBuf, Path};
use tauri::Window;
use tokio::sync::oneshot;
use tauri_plugin_dialog::DialogExt;
use std::process::Command;
use std::env;
use std::fs;
use crate::error::{Error, Result};

#[tauri::command]
pub async fn pick_save_file(window: Window) -> Result<String> {
    let (tx, rx) = oneshot::channel::<Option<PathBuf>>();

    window.dialog().file().save_file(move |file_path| {
        let pb_opt: Option<PathBuf> = file_path.and_then(|fp| fp.into_path().ok());
        let _ = tx.send(pb_opt);
    });

    match rx.await {
        Ok(Some(pathbuf)) => pathbuf
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| Error::Internal("Invalid Unicode in path".to_string())),
        Ok(None) => Err(Error::Internal("File selection cancelled".to_string())),
        Err(_) => Err(Error::Internal("File selection failed".to_string())),
    }
}

#[tauri::command]
pub async fn pick_open_file(window: Window) -> Result<String> {
    let (tx, rx) = oneshot::channel::<Option<PathBuf>>();

    window.dialog().file().pick_file(move |file_path| {
        let pb_opt: Option<PathBuf> = file_path.and_then(|fp| fp.into_path().ok());
        let _ = tx.send(pb_opt);
    });

    match rx.await {
        Ok(Some(pathbuf)) => pathbuf
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| Error::Internal("Invalid Unicode in path".to_string())),
        Ok(None) => Err(Error::Internal("File selection cancelled".to_string())),
        Err(_) => Err(Error::Internal("File selection failed".to_string())),
    }
}

#[tauri::command]
pub async fn check_file_exists(path: String) -> Result<bool> {
    Ok(tokio::fs::try_exists(path).await.unwrap_or(false))
}

#[tauri::command]
pub async fn elevated_copy(src: String) -> Result<String> {
    let local_appdata = env::var("LOCALAPPDATA").map_err(|e| Error::Internal(format!("Failed to get LOCALAPPDATA: {}", e)))?;
    let app_dir = Path::new(&local_appdata).join("Pulsar");
    if !tokio::fs::try_exists(&app_dir).await.unwrap_or(false) {
        tokio::fs::create_dir_all(&app_dir).await.map_err(|e| Error::Internal(format!("Failed to create app dir {}: {}", app_dir.display(), e)))?;
    }

    let src_path = Path::new(&src);
    let file_name = src_path.file_name().and_then(|s| s.to_str()).ok_or_else(|| Error::Internal("Invalid source filename".to_string()))?;
    let dest_path = app_dir.join(file_name);

    let sanitized_src = src.replace("'", "''");
    let sanitized_dest = dest_path.to_string_lossy().replace("'", "''");
    let argument_list = format!(
        "-NoProfile -Command Copy-Item -LiteralPath '{}' -Destination '{}' -Force",
        sanitized_src,
        sanitized_dest
    );

    let status = tokio::process::Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg("Start-Process")
        .arg("-FilePath")
        .arg("powershell")
        .arg("-ArgumentList")
        .arg(argument_list)
        .arg("-Verb")
        .arg("RunAs")
        .arg("-Wait")
        .status()
        .await
        .map_err(|e| Error::Internal(format!("Failed to launch elevated copy: {}", e)))?;

    if status.success() {
        Ok(dest_path.to_string_lossy().into_owned())
    } else {
        Err(Error::Internal(format!("Elevated copy failed with exit code: {}", status.code().map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string()))))
    }
}

