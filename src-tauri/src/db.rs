use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::path::{Path, PathBuf};
use tokio::fs;
use std::env;
use std::io::Read;
use zeroize::Zeroizing;

pub async fn init_db(db_path: &Path, password: Option<&[u8]>) -> Result<SqlitePool, String> {
    let db_path_abs: PathBuf = if db_path.is_absolute() {
        db_path.to_path_buf()
    } else {
        env::current_dir()
            .map_err(|e| format!("Failed to get current dir: {}", e))? 
            .join(db_path)
    };

    let db_exists = db_path_abs.exists();

    let file_header: Option<String> = if db_path_abs.exists() {
        match std::fs::File::open(&db_path_abs) {
            Ok(mut f) => {
                let mut buf = [0u8; 16];
                match f.read_exact(&mut buf) {
                    Ok(()) => {
                        let hex: String = buf.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
                        let ascii: String = buf.iter().map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' }).collect();
                        Some(format!("hex: {} ascii: {}", hex, ascii))
                    }
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    } else {
        None
    };

    if db_path_abs.exists() {
        if db_path_abs.is_dir() {
            return Err(format!("Database path {} is a directory", db_path_abs.display()));
        }

        let meta = std::fs::symlink_metadata(&db_path_abs)
            .map_err(|e| format!("Failed to stat database path {}: {}", db_path_abs.display(), e))?;
        if meta.file_type().is_symlink() {
            return Err(format!("Database path {} is a symbolic link; symlinks are not allowed", db_path_abs.display()));
        }
        let meta2 = std::fs::metadata(&db_path_abs)
            .map_err(|e| format!("Failed to stat database path {}: {}", db_path_abs.display(), e))?;
        if !meta2.is_file() {
            return Err(format!("Database path {} is not a regular file", db_path_abs.display()));
        }
    }

    if let Some(parent) = db_path_abs.parent() {
        if parent.exists() {
            let parent_meta = std::fs::symlink_metadata(parent)
                .map_err(|e| format!("Failed to stat database parent {}: {}", parent.display(), e))?;
            if parent_meta.file_type().is_symlink() {
                return Err(format!("Database parent directory {} is a symbolic link; symlinks are not allowed", parent.display()));
            }
        }

        fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create database directory {}: {}", parent.display(), e))?;
    }

    let mut open_readonly = false;
    if db_exists {
        match fs::OpenOptions::new().read(true).write(true).open(&db_path_abs).await {
            Ok(_) => {} 
            Err(e) => {
                let mut details = String::new();
                details.push_str(&format!("Initial async open error: {}", e));
                if let Some(code) = e.raw_os_error() {
                    details.push_str(&format!("OS error code: {}", code));
                }

                match std::fs::metadata(&db_path_abs) {
                    Ok(md) => {
                        details.push_str(&format!("File exists; readonly attribute: {}", md.permissions().readonly()));
                    }
                    Err(me) => {
                        details.push_str(&format!("Failed to stat file metadata: {}", me));
                    }
                }

                if let Some(parent) = db_path_abs.parent() {
                    match std::fs::metadata(parent) {
                        Ok(pmd) => {
                            details.push_str(&format!("Parent exists; readonly attribute: {}", pmd.permissions().readonly()));
                        }
                        Err(pe) => {
                            details.push_str(&format!("Failed to stat parent metadata: {}", pe));
                        }
                    }
                }

                match std::fs::OpenOptions::new().read(true).write(true).open(&db_path_abs) {
                    Ok(_) => {
                        details.push_str("Sync open unexpectedly succeeded while async open failed\n");
                    }
                    Err(se) => {
                        details.push_str(&format!("Sync open error: {}", se));
                        if let Some(code) = se.raw_os_error() {
                            details.push_str(&format!("Sync OS error code: {}", code));
                        }
                    }
                }

                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    match fs::OpenOptions::new().read(true).open(&db_path_abs).await {
                        Ok(_) => {
                            open_readonly = true;

                            details.push_str("Read-only fallback succeeded.\n");
                        }
                        Err(e2) => {
                            details.push_str(&format!("Read-only fallback error: {}", e2));
                            return Err(format!("Failed to open database file {}: {}. Diagnostics:\n{}Check file permissions and ensure the application has access to the file.", db_path_abs.display(), e2, details));
                        }
                    }
                } else {
                    return Err(format!("Failed to open database file {}: {}. Diagnostics:\n{}Check file permissions and ensure the application has access to the file.", db_path_abs.display(), e, details));
                }
            }
        }
    }

    let path_str = db_path_abs.to_string_lossy();

    let mut opts = SqliteConnectOptions::new()
        .filename(path_str.as_ref())
        .create_if_missing(!db_exists);

    if open_readonly {
        opts = opts.read_only(true);
    }

    let maybe_key_bytes: Option<Zeroizing<Vec<u8>>> = password.map(|p| Zeroizing::new(p.to_vec()));

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .after_connect(move |conn, _meta| {
            let kb = maybe_key_bytes.clone();
            Box::pin(async move {
                if let Some(key_bytes) = kb {
                    let param_try = sqlx::query("PRAGMA key = ?;")
                        .bind(key_bytes.as_slice())
                        .execute(&mut *conn)
                        .await;

                    if let Err(e) = param_try {
                        let msg = e.to_string();
                        if msg.contains("near \"?\": syntax error") || msg.contains("syntax error") {
                            let hex_key: String = key_bytes.iter().map(|b| format!("{:02x}", b)).collect();
                            let pragma_unquoted = format!("PRAGMA key = x'{}';", hex_key);
                            let unq_try = sqlx::query(&pragma_unquoted).execute(&mut *conn).await;
                            if unq_try.is_err() {
                                let pragma_quoted = format!("PRAGMA key = \"x'{}'\";", hex_key);
                                sqlx::query(&pragma_quoted)
                                    .execute(&mut *conn)
                                    .await?;
                            }
                        } else {
                            return Err(e);
                        }
                    }

                    let res = sqlx::query("PRAGMA integrity_check;").execute(&mut *conn).await;
                    if let Err(e) = res {
                        let msg = e.to_string();
                        if msg.contains("file is not a database") {
                            return Err(sqlx::Error::Configuration("Unlock failed: The password may be incorrect, or the database file is not encrypted.".into()));
                        }
                        return Err(e);
                    }
                }
                sqlx::query("PRAGMA foreign_keys = ON;")
                    .execute(&mut *conn)
                    .await
                    .map(|_| ())?;
                Ok(())
            })
        })
        .connect_with(opts)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    if !open_readonly {
        sqlx::migrate!()
            .run(&pool)
            .await
            .map_err(|e| {
                let mut msg = format!("Failed to run embedded migrations: {}", e);
                if let Some(h) = &file_header {
                    msg.push_str(&format!("; file header: {}", h));
                }
                msg
            })?;
    } else {
        // Do nothing
    }

    Ok(pool)
}