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
            fs::create_dir_all(plist_path.parent().unwrap())
                .map_err(|e| Error::Internal(e.to_string()))?;
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

            let event_down =
                CGEvent::new_keyboard_event(source.clone(), 0, true).map_err(|_| {
                    Error::Internal("Failed to create keyboard event down".to_string())
                })?;
            event_down.set_string(encoded);
            event_down.post(CGEventTapLocation::HID);

            let event_up =
                CGEvent::new_keyboard_event(source.clone(), 0, false).map_err(|_| {
                    Error::Internal("Failed to create keyboard event up".to_string())
                })?;
            event_up.set_string(encoded);
            event_up.post(CGEventTapLocation::HID);

            let delay = 20 + (rand::random::<u8>() % 30) as u64;
            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
        }

        // Send Enter
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
        // Linux implementation often requires external tools or specific permissions
        // for uinput. Placeholder for now.
        Ok(())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err(Error::Internal(
            "Autotype simulation not implemented for this OS".to_string(),
        ))
    }
}

