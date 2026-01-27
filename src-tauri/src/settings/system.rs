use crate::error::{Error, Result};
use std::env;

#[cfg(target_os = "windows")]
use windows::{
    core::PCWSTR,
    Win32::Foundation::{ERROR_SUCCESS, ERROR_FILE_NOT_FOUND},
    Win32::System::Registry::{
        RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
        KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ,
    },
};

#[cfg(target_os = "windows")]
fn wide_null(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::prelude::OsStrExt;
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

pub fn set_autostart(enabled: bool) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let exe_path = env::current_exe().map_err(|e| Error::Internal(e.to_string()))?;
        let exe_path_str = exe_path.to_str().ok_or_else(|| Error::Internal("Invalid exe path".to_string()))?;
        
        unsafe {
            let key_name = wide_null("Software\Microsoft\Windows\CurrentVersion\Run");
            let mut key: HKEY = HKEY::default();
            let status = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(key_name.as_ptr()),
                0,
                KEY_SET_VALUE | KEY_QUERY_VALUE,
                &mut key,
            );
            
            if status != ERROR_SUCCESS {
                return Err(Error::Internal("Failed to open autostart registry key".to_string()));
            }

            let value_name = wide_null("Pulsar");
            if enabled {
                let val = wide_null(&format!("\"{}\" --minimized", exe_path_str));
                let status = RegSetValueExW(
                    key,
                    PCWSTR(value_name.as_ptr()),
                    0,
                    REG_SZ,
                    Some(std::slice::from_raw_parts(val.as_ptr() as *const u8, val.len() * 2)),
                );
                let _ = RegCloseKey(key);
                if status != ERROR_SUCCESS {
                    return Err(Error::Internal("Failed to set autostart value".to_string()));
                }
            } else {
                let status = RegDeleteValueW(key, PCWSTR(value_name.as_ptr()));
                let _ = RegCloseKey(key);
                if status != ERROR_SUCCESS && status != ERROR_FILE_NOT_FOUND {
                    return Err(Error::Internal("Failed to delete autostart value".to_string()));
                }
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // MacOS/Linux implementation would go here (e.g. LaunchAgents or .config/autostart)
        // For now, we only implement Windows as requested/detected.
    }
    
    Ok(())
}

#[tauri::command]
pub async fn simulate_autotype() -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::Input::KeyboardAndMouse::*;
        
        let test_text = "Pulsar Autotype Test";
        
        // Wait a bit to let user focus
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        unsafe {
            for c in test_text.chars() {
                let mut inputs = [INPUT::default(); 2];
                
                // Key down
                inputs[0].r#type = INPUT_KEYBOARD;
                inputs[0].Anonymous.ki = KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: c as u16,
                    dwFlags: KEYEVENTF_UNICODE,
                    time: 0,
                    dwExtraInfo: 0,
                };
                
                // Key up
                inputs[1].r#type = INPUT_KEYBOARD;
                inputs[1].Anonymous.ki = KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: c as u16,
                    dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                };
                
                SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // Implement for other OSs
    }
    
    Ok(())
}

