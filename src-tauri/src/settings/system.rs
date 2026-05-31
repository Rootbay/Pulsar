use crate::error::{Error, Result};
use std::env;

#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::fs;

#[cfg(target_os = "windows")]
use windows::{
    core::PCWSTR,
    Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS},
    Win32::System::Registry::{
        RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
        KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ,
    },
};

#[cfg(target_os = "macos")]
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode, CGMouseButton, ScrollType,
};
#[cfg(target_os = "macos")]
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

#[cfg(target_os = "windows")]
fn wide_null(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::prelude::OsStrExt;
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

pub fn set_autostart(enabled: bool) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let exe_path = env::current_exe().map_err(|e| Error::Internal(e.to_string()))?;
        let exe_path_str = exe_path
            .to_str()
            .ok_or_else(|| Error::Internal("Invalid exe path".to_string()))?;

        unsafe {
            let key_name = wide_null("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
            let mut key: HKEY = HKEY::default();
            let status = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(key_name.as_ptr()),
                0,
                KEY_SET_VALUE | KEY_QUERY_VALUE,
                &mut key,
            );

            if status != ERROR_SUCCESS {
                return Err(Error::Internal(
                    "Failed to open autostart registry key".to_string(),
                ));
            }

            let value_name = wide_null("Pulsar");
            if enabled {
                let val = wide_null(&format!("\"{}\" --minimized", exe_path_str));
                let status = RegSetValueExW(
                    key,
                    PCWSTR(value_name.as_ptr()),
                    0,
                    REG_SZ,
                    Some(std::slice::from_raw_parts(
                        val.as_ptr() as *const u8,
                        val.len() * 2,
                    )),
                );
                let _ = RegCloseKey(key);
                if status != ERROR_SUCCESS {
                    return Err(Error::Internal("Failed to set autostart value".to_string()));
                }
            } else {
                let status = RegDeleteValueW(key, PCWSTR(value_name.as_ptr()));
                let _ = RegCloseKey(key);
                if status != ERROR_SUCCESS && status != ERROR_FILE_NOT_FOUND {
                    return Err(Error::Internal(
                        "Failed to delete autostart value".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or_else(|| Error::Internal("Home dir not found".into()))?;
        let plist_path = home.join("Library/LaunchAgents/app.pulsar.plist");

        if enabled {
            let exe_path = env::current_exe().map_err(|e| Error::Internal(e.to_string()))?;
            let plist_content = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>app.pulsar</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>--minimized</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>"#,
                exe_path.display()
            );
            if let Some(parent) = plist_path.parent() {
                fs::create_dir_all(parent).map_err(|e| Error::Internal(e.to_string()))?;
            }
            fs::write(plist_path, plist_content).map_err(|e| Error::Internal(e.to_string()))?;
        } else if plist_path.exists() {
            fs::remove_file(plist_path).map_err(|e| Error::Internal(e.to_string()))?;
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        let config_dir =
            dirs::config_dir().ok_or_else(|| Error::Internal("Config dir not found".into()))?;
        let autostart_dir = config_dir.join("autostart");
        fs::create_dir_all(&autostart_dir).map_err(|e| Error::Internal(e.to_string()))?;
        let desktop_file = autostart_dir.join("pulsar.desktop");

        if enabled {
            let exe_path = env::current_exe().map_err(|e| Error::Internal(e.to_string()))?;
            let content = format!(
                r#"[Desktop Entry]
Type=Application
Name=Pulsar
Exec={} --minimized
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true"#,
                exe_path.display()
            );
            fs::write(desktop_file, content).map_err(|e| Error::Internal(e.to_string()))?;
        } else if desktop_file.exists() {
            fs::remove_file(desktop_file).map_err(|e| Error::Internal(e.to_string()))?;
        }
        Ok(())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Ok(())
    }
}

#[tauri::command]
pub async fn simulate_autotype() -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::Input::KeyboardAndMouse::*;
        let test_text = "Pulsar Autotype Test - 123456";
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        unsafe {
            for c in test_text.chars() {
                let mut inputs = [INPUT::default(); 2];

                inputs[0].r#type = INPUT_KEYBOARD;
                inputs[0].Anonymous.ki = KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: c as u16,
                    dwFlags: KEYEVENTF_UNICODE,
                    time: 0,
                    dwExtraInfo: 0,
                };

                inputs[1].r#type = INPUT_KEYBOARD;
                inputs[1].Anonymous.ki = KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: c as u16,
                    dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                };

                if SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) == 0 {
                    return Err(Error::Internal("Failed to send keyboard input".to_string()));
                }

                let delay = 20 + (rand::random::<u8>() % 30) as u64;
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }

            let mut enter_inputs = [INPUT::default(); 2];
            enter_inputs[0].r#type = INPUT_KEYBOARD;
            enter_inputs[0].Anonymous.ki = KEYBDINPUT {
                wVk: VK_RETURN,
                wScan: 0,
                dwFlags: KEYBD_EVENT_FLAGS(0),
                time: 0,
                dwExtraInfo: 0,
            };
            enter_inputs[1].r#type = INPUT_KEYBOARD;
            enter_inputs[1].Anonymous.ki = KEYBDINPUT {
                wVk: VK_RETURN,
                wScan: 0,
                dwFlags: KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            };
            SendInput(&enter_inputs, std::mem::size_of::<INPUT>() as i32);
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        use core_graphics::event::{CGEvent, CGEventTapLocation};
        use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

        let test_text = "Pulsar Autotype Test - 123456";
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
            .map_err(|_| Error::Internal("Failed to create event source".to_string()))?;

        for c in test_text.chars() {
            let mut utf16 = [0u16; 2];
            let encoded = c.encode_utf16(&mut utf16);

            let event_down = CGEvent::new_keyboard_event(source.clone(), 0, true)
                .map_err(|_| Error::Internal("Failed to create keyboard event down".to_string()))?;
            event_down.set_string(encoded);
            event_down.post(CGEventTapLocation::HID);

            let event_up = CGEvent::new_keyboard_event(source.clone(), 0, false)
                .map_err(|_| Error::Internal("Failed to create keyboard event up".to_string()))?;
            event_up.set_string(encoded);
            event_up.post(CGEventTapLocation::HID);

            let delay = 20 + (rand::random::<u8>() % 30) as u64;
            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
        }

        let event_down = CGEvent::new_keyboard_event(source.clone(), 36, true)
            .map_err(|_| Error::Internal("Failed to create enter key down".to_string()))?;
        event_down.post(CGEventTapLocation::HID);
        let event_up = CGEvent::new_keyboard_event(source.clone(), 36, false)
            .map_err(|_| Error::Internal("Failed to create enter key up".to_string()))?;
        event_up.post(CGEventTapLocation::HID);

        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        use tokio::process::Command;

        let test_text = "Pulsar Autotype Test - 123456";
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        let type_status = Command::new("xdotool")
            .arg("type")
            .arg("--delay")
            .arg("50")
            .arg(test_text)
            .status()
            .await;

        match type_status {
            Ok(s) if s.success() => {
                let _ = Command::new("xdotool")
                    .arg("key")
                    .arg("Return")
                    .status()
                    .await;
                Ok(())
            }
            _ => {
                Err(Error::Internal(
                    "Autotype failed: 'xdotool' not found or failed. Please install xdotool to use this feature on Linux.".to_string(),
                ))
            }
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err(Error::Internal(
            "Autotype simulation not implemented for this OS".to_string(),
        ))
    }
}

#[cfg(target_os = "windows")]
unsafe fn send_unicode_char(c: char) -> Result<()> {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    let mut inputs = [INPUT::default(); 2];

    inputs[0].r#type = INPUT_KEYBOARD;
    inputs[0].Anonymous.ki = KEYBDINPUT {
        wVk: VIRTUAL_KEY(0),
        wScan: c as u16,
        dwFlags: KEYEVENTF_UNICODE,
        time: 0,
        dwExtraInfo: 0,
    };

    inputs[1].r#type = INPUT_KEYBOARD;
    inputs[1].Anonymous.ki = KEYBDINPUT {
        wVk: VIRTUAL_KEY(0),
        wScan: c as u16,
        dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
        time: 0,
        dwExtraInfo: 0,
    };

    if SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) == 0 {
        return Err(Error::Internal("Failed to send unicode char".to_string()));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
unsafe fn send_virtual_key(vk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY) -> Result<()> {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    let mut inputs = [INPUT::default(); 2];
    inputs[0].r#type = INPUT_KEYBOARD;
    inputs[0].Anonymous.ki = KEYBDINPUT {
        wVk: vk,
        wScan: 0,
        dwFlags: KEYBD_EVENT_FLAGS(0),
        time: 0,
        dwExtraInfo: 0,
    };
    inputs[1].r#type = INPUT_KEYBOARD;
    inputs[1].Anonymous.ki = KEYBDINPUT {
        wVk: vk,
        wScan: 0,
        dwFlags: KEYEVENTF_KEYUP,
        time: 0,
        dwExtraInfo: 0,
    };
    if SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) == 0 {
        return Err(Error::Internal("Failed to send key stroke".to_string()));
    }
    Ok(())
}

#[tauri::command]
pub async fn perform_autotype(
    app_handle: tauri::AppHandle,
    username: Option<String>,
    password: Option<String>,
) -> Result<()> {
    use tauri::Manager;

    // 1. Minimize main window to restore OS focus to the previously active application
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.minimize();
    }

    // 2. Wait for the focus transition to settle down completely (500ms)
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 3. Perform platform-specific keyboard input simulation
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::Input::KeyboardAndMouse::*;

        unsafe {
            // Type username if present
            if let Some(ref user) = username {
                for c in user.chars() {
                    send_unicode_char(c)?;
                    let delay = 15 + (rand::random::<u8>() % 20) as u64;
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                }

                // Press Tab to navigate to password field if password is also present
                if password.is_some() {
                    send_virtual_key(VK_TAB)?;
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                }
            }

            // Type password if present
            if let Some(ref pass) = password {
                for c in pass.chars() {
                    send_unicode_char(c)?;
                    let delay = 15 + (rand::random::<u8>() % 20) as u64;
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                }

                // Press Enter to submit
                send_virtual_key(VK_RETURN)?;
            }
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        use core_graphics::event::{CGEvent, CGEventTapLocation};
        use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

        let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
            .map_err(|_| Error::Internal("Failed to create event source".to_string()))?;

        // Type username if present
        if let Some(ref user) = username {
            for c in user.chars() {
                let mut utf16 = [0u16; 2];
                let encoded = c.encode_utf16(&mut utf16);

                let event_down = CGEvent::new_keyboard_event(source.clone(), 0, true)
                    .map_err(|_| Error::Internal("Failed to create keyboard event down".to_string()))?;
                event_down.set_string(encoded);
                event_down.post(CGEventTapLocation::HID);

                let event_up = CGEvent::new_keyboard_event(source.clone(), 0, false)
                    .map_err(|_| Error::Internal("Failed to create keyboard event up".to_string()))?;
                event_up.set_string(encoded);
                event_up.post(CGEventTapLocation::HID);

                let delay = 15 + (rand::random::<u8>() % 20) as u64;
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }

            // Press Tab (keycode 48) if password is also present
            if password.is_some() {
                let event_down = CGEvent::new_keyboard_event(source.clone(), 48, true)
                    .map_err(|_| Error::Internal("Failed to create tab event down".to_string()))?;
                event_down.post(CGEventTapLocation::HID);
                let event_up = CGEvent::new_keyboard_event(source.clone(), 48, false)
                    .map_err(|_| Error::Internal("Failed to create tab event up".to_string()))?;
                event_up.post(CGEventTapLocation::HID);
                tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            }
        }

        // Type password if present
        if let Some(ref pass) = password {
            for c in pass.chars() {
                let mut utf16 = [0u16; 2];
                let encoded = c.encode_utf16(&mut utf16);

                let event_down = CGEvent::new_keyboard_event(source.clone(), 0, true)
                    .map_err(|_| Error::Internal("Failed to create keyboard event down".to_string()))?;
                event_down.set_string(encoded);
                event_down.post(CGEventTapLocation::HID);

                let event_up = CGEvent::new_keyboard_event(source.clone(), 0, false)
                    .map_err(|_| Error::Internal("Failed to create keyboard event up".to_string()))?;
                event_up.set_string(encoded);
                event_up.post(CGEventTapLocation::HID);

                let delay = 15 + (rand::random::<u8>() % 20) as u64;
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }

            // Press Enter (keycode 36)
            let event_down = CGEvent::new_keyboard_event(source.clone(), 36, true)
                .map_err(|_| Error::Internal("Failed to create enter event down".to_string()))?;
            event_down.post(CGEventTapLocation::HID);
            let event_up = CGEvent::new_keyboard_event(source.clone(), 36, false)
                .map_err(|_| Error::Internal("Failed to create enter event up".to_string()))?;
            event_up.post(CGEventTapLocation::HID);
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        use tokio::process::Command;

        // Type username if present
        if let Some(ref user) = username {
            let _ = Command::new("xdotool")
                .arg("type")
                .arg("--delay")
                .arg("30")
                .arg(user)
                .status()
                .await;

            // Press Tab if password is also present
            if password.is_some() {
                let _ = Command::new("xdotool").arg("key").arg("Tab").status().await;
                tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            }
        }

        // Type password if present
        if let Some(ref pass) = password {
            let _ = Command::new("xdotool")
                .arg("type")
                .arg("--delay")
                .arg("30")
                .arg(pass)
                .status()
                .await;

            // Press Return/Enter
            let _ = Command::new("xdotool").arg("key").arg("Return").status().await;
        }

        Ok(())
    }

    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err(Error::Internal(
            "Autotype simulation not implemented for this OS".to_string(),
        ))
    }
}

#[cfg(target_os = "windows")]
pub fn get_active_window_title() -> Option<String> {
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW};

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == 0 {
            return None;
        }

        let mut buffer = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut buffer);
        if len == 0 {
            return None;
        }

        let title = String::from_utf16_lossy(&buffer[..len as usize]);
        Some(title)
    }
}

pub async fn handle_global_autotype_trigger(app_handle: tauri::AppHandle) -> Result<()> {
    use tauri::Manager;
    use crate::state::AppState;
    use crate::db::passwords::get_password_items_impl;
    use crate::db::utils::{get_key, get_db_pool};
    use tauri::Emitter;

    // 1. Get the active window's title
    #[cfg(target_os = "windows")]
    let window_title = match get_active_window_title() {
        Some(title) => title,
        None => return Ok(()),
    };

    #[cfg(not(target_os = "windows"))]
    let window_title = "".to_string();

    if window_title.is_empty() {
        return Ok(());
    }

    let title_lower = window_title.to_lowercase();

    // 2. Fetch keys and database pool from AppState
    let state = app_handle.state::<AppState>();
    
    let key = match get_key(&state).await {
        Ok(k) => k,
        Err(_) => {
            let _ = app_handle.emit("pulsar:autotype-error", "Vault is locked. Please unlock Pulsar first.");
            return Ok(());
        }
    };

    let db_pool = match get_db_pool(&state).await {
        Ok(db) => db,
        Err(_) => return Ok(()),
    };

    // 3. Load all decrypted password items
    let items = match get_password_items_impl(&db_pool, key.as_slice()).await {
        Ok(items) => items,
        Err(e) => {
            eprintln!("Failed to load items: {:?}", e);
            return Ok(());
        }
    };

    // 4. Match items against window title
    let mut matches = Vec::new();
    for item in items {
        let item_title = item.title.to_lowercase();
        if !item_title.is_empty() && title_lower.contains(&item_title) {
            matches.push(item);
            continue;
        }

        if let Some(ref url) = item.url {
            let url_lower = url.to_lowercase();
            let clean_url = url_lower
                .replace("https://", "")
                .replace("http://", "")
                .replace("www.", "");
            let domain = clean_url.split('/').next().unwrap_or(&clean_url);
            let domain_name = domain.split('.').next().unwrap_or(domain);

            if !domain_name.is_empty() && title_lower.contains(domain_name) {
                matches.push(item);
            }
        }
    }

    // 5. Trigger autotype sequence
    if matches.is_empty() {
        let err_msg = format!("No matching credentials found for window title: \"{}\"", window_title);
        let _ = app_handle.emit("pulsar:autotype-error", err_msg);
    } else if matches.len() == 1 {
        let target = &matches[0];
        let info_msg = format!("Auto-Type matching entry: \"{}\"...", target.title);
        let _ = app_handle.emit("pulsar:autotype-info", info_msg);

        // Sleep 150ms to let key events settle
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;

        #[cfg(target_os = "windows")]
        {
            use windows::Win32::UI::Input::KeyboardAndMouse::*;

            unsafe {
                if let Some(ref user) = target.username {
                    for c in user.chars() {
                        let _ = send_unicode_char(c);
                        let delay = 15 + (rand::random::<u8>() % 20) as u64;
                        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                    }

                    let _ = send_virtual_key(VK_TAB)?;
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                }

                for c in target.password.chars() {
                    let _ = send_unicode_char(c)?;
                    let delay = 15 + (rand::random::<u8>() % 20) as u64;
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                }

                let _ = send_virtual_key(VK_RETURN)?;
            }
        }

        #[cfg(target_os = "macos")]
        {
            use core_graphics::event::{CGEvent, CGEventTapLocation};
            use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

            if let Ok(source) = CGEventSource::new(CGEventSourceStateID::CombinedSessionState) {
                if let Some(ref user) = target.username {
                    for c in user.chars() {
                        let mut utf16 = [0u16; 2];
                        let encoded = c.encode_utf16(&mut utf16);

                        let event_down = CGEvent::new_keyboard_event(source.clone(), 0, true).unwrap();
                        event_down.set_string(encoded);
                        event_down.post(CGEventTapLocation::HID);

                        let event_up = CGEvent::new_keyboard_event(source.clone(), 0, false).unwrap();
                        event_up.set_string(encoded);
                        event_up.post(CGEventTapLocation::HID);

                        let delay = 15 + (rand::random::<u8>() % 20) as u64;
                        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                    }

                    let event_down = CGEvent::new_keyboard_event(source.clone(), 48, true).unwrap();
                    event_down.post(CGEventTapLocation::HID);
                    let event_up = CGEvent::new_keyboard_event(source.clone(), 48, false).unwrap();
                    event_up.post(CGEventTapLocation::HID);
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                }

                for c in target.password.chars() {
                    let mut utf16 = [0u16; 2];
                    let encoded = c.encode_utf16(&mut utf16);

                    let event_down = CGEvent::new_keyboard_event(source.clone(), 0, true).unwrap();
                    event_down.set_string(encoded);
                    event_down.post(CGEventTapLocation::HID);

                    let event_up = CGEvent::new_keyboard_event(source.clone(), 0, false).unwrap();
                    event_up.set_string(encoded);
                    event_up.post(CGEventTapLocation::HID);

                    let delay = 15 + (rand::random::<u8>() % 20) as u64;
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                }

                let event_down = CGEvent::new_keyboard_event(source.clone(), 36, true).unwrap();
                event_down.post(CGEventTapLocation::HID);
                let event_up = CGEvent::new_keyboard_event(source.clone(), 36, false).unwrap();
                event_up.post(CGEventTapLocation::HID);
            }
        }

        #[cfg(target_os = "linux")]
        {
            use tokio::process::Command;

            if let Some(ref user) = target.username {
                let _ = Command::new("xdotool")
                    .arg("type")
                    .arg("--delay")
                    .arg("30")
                    .arg(user)
                    .status()
                    .await;

                let _ = Command::new("xdotool").arg("key").arg("Tab").status().await;
                tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            }

            let _ = Command::new("xdotool")
                .arg("type")
                .arg("--delay")
                .arg("30")
                .arg(&target.password)
                .status()
                .await;

            let _ = Command::new("xdotool").arg("key").arg("Return").status().await;
        }
    } else {
        let err_msg = format!(
            "Multiple matching credentials found for \"{}\". Please select your account inside Pulsar.",
            window_title
        );
        let _ = app_handle.emit("pulsar:autotype-error", err_msg);
    }

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn start_global_hotkey_listener(app_handle: tauri::AppHandle) {
    use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, MOD_ALT, MOD_CONTROL};
    use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

    let handle = app_handle.clone();
    std::thread::spawn(move || unsafe {
        // Register Ctrl+Alt+A (virtual key 0x41 is 'A')
        // Hotkey identifier is 1
        let success = RegisterHotKey(
            None,
            1,
            MOD_CONTROL | MOD_ALT,
            0x41, // 'A'
        );

        if success.is_err() {
            eprintln!("Failed to register global hotkey Ctrl+Alt+A");
            return;
        }

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            if msg.message == WM_HOTKEY && msg.wParam.0 == 1 {
                let handle_clone = handle.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = handle_global_autotype_trigger(handle_clone).await {
                        eprintln!("Global autotype failed: {:?}", e);
                    }
                });
            }
        }
    });
}

#[cfg(not(target_os = "windows"))]
pub fn start_global_hotkey_listener(_app_handle: tauri::AppHandle) {}


