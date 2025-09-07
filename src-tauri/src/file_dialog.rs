use std::path::{PathBuf, Path};
use tauri::Window;
use tokio::sync::oneshot;
use tauri_plugin_dialog::DialogExt;
use std::process::Command;
use std::env;
use std::fs;

#[tauri::command]
pub async fn pick_save_file(window: Window) -> Result<String, String> {
    let (tx, rx) = oneshot::channel::<Option<PathBuf>>();

    window.dialog().file().save_file(move |file_path| {
        let pb_opt: Option<PathBuf> = file_path.and_then(|fp| fp.into_path().ok());
        let _ = tx.send(pb_opt);
    });

    match rx.await {
        Ok(Some(pathbuf)) => pathbuf
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Invalid Unicode in path".to_string()),
        Ok(None) => Err("File selection cancelled".to_string()),
        Err(_) => Err("File selection failed".to_string()),
    }
}

#[tauri::command]
pub async fn pick_open_file(window: Window) -> Result<String, String> {
    let (tx, rx) = oneshot::channel::<Option<PathBuf>>();

    window.dialog().file().pick_file(move |file_path| {
        let pb_opt: Option<PathBuf> = file_path.and_then(|fp| fp.into_path().ok());
        let _ = tx.send(pb_opt);
    });

    match rx.await {
        Ok(Some(pathbuf)) => pathbuf
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Invalid Unicode in path".to_string()),
        Ok(None) => Err("File selection cancelled".to_string()),
        Err(_) => Err("File selection failed".to_string()),
    }
}

#[tauri::command]
pub async fn check_file_exists(path: String) -> Result<bool, String> {
    Ok(Path::new(&path).exists())
}

#[tauri::command]
pub async fn elevated_copy(src: String) -> Result<String, String> {
    let local_appdata = env::var("LOCALAPPDATA").map_err(|e| format!("Failed to get LOCALAPPDATA: {}", e))?;
    let app_dir = Path::new(&local_appdata).join("Pulsar");
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).map_err(|e| format!("Failed to create app dir {}: {}", app_dir.display(), e))?;
    }

    let src_path = Path::new(&src);
    let file_name = src_path.file_name().and_then(|s| s.to_str()).ok_or_else(|| "Invalid source filename".to_string())?;
    let dest_path = app_dir.join(file_name);

    let copy_cmd = format!("-NoProfile -Command Copy-Item -LiteralPath \"{}\" -Destination \"{}\" -Force", src, dest_path.display());
    let start_process_cmd = format!("Start-Process -FilePath powershell -ArgumentList '{}' -Verb RunAs -Wait", copy_cmd);

    let status = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg("Start-Process")
        .arg("-FilePath")
        .arg("powershell")
        .arg("-ArgumentList")
        .arg(format!("-NoProfile -Command Copy-Item -LiteralPath '{}' -Destination '{}' -Force", src.replace("'", "''"), dest_path.to_string_lossy().replace("'", "''")))
        .arg("-Verb")
        .arg("RunAs")
        .arg("-Wait")
        .status()
        .map_err(|e| format!("Failed to launch elevated copy: {}", e))?;

    if status.success() {
        Ok(dest_path.to_string_lossy().into_owned())
    } else {
        Err(format!("Elevated copy failed with exit code: {}", status.code().map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string())))
    }
}
