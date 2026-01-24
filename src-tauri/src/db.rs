use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::path::{Path, PathBuf};
use tokio::fs;
use std::env;
use zeroize::Zeroizing;

pub async fn init_db(db_path: &Path, password: Option<&[u8]>) -> Result<SqlitePool, String> {
    let db_path_abs: PathBuf = if db_path.is_absolute() {
        db_path.to_path_buf()
    } else {
        env::current_dir()
            .map_err(|e| {
                eprintln!("Failed to get current dir: {}", e);
                "Internal error: Could not resolve database path".to_string()
            })? 
            .join(db_path)
    };

    if let Some(parent) = db_path_abs.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| {
                eprintln!("Failed to create database directory {}: {}", parent.display(), e);
                "Failed to access database directory".to_string()
            })?;
    }

    let path_str = db_path_abs.to_string_lossy();
    let opts = SqliteConnectOptions::new()
        .filename(path_str.as_ref())
        .create_if_missing(true);

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
                        if msg.contains("syntax error") {
                            let hex_key: String = key_bytes.iter().map(|b| format!("{:02x}", b)).collect();
                            let pragma_unquoted = format!("PRAGMA key = x'{}';", hex_key);
                            if let Err(e2) = sqlx::query(&pragma_unquoted).execute(&mut *conn).await {
                                eprintln!("Primary and fallback PRAGMA key failed. Error: {}", e2);
                                return Err(e2);
                            }
                        } else {
                            return Err(e);
                        }
                    }

                    let res = sqlx::query("PRAGMA integrity_check;").execute(&mut *conn).await;
                    if let Err(e) = res {
                        let msg = e.to_string();
                        if msg.contains("file is not a database") {
                            return Err(sqlx::Error::Configuration("Vault unlock failed. The password may be incorrect or the database is corrupted.".into()));
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
        .map_err(|e| {
            eprintln!("Database connection error: {}", e);
            "Failed to connect to the database. Ensure the file is not locked by another process.".to_string()
        })?;

    println!("Running database migrations...");
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(|e| {
            eprintln!("Database migration error: {}", e);
            "Internal error: Database migration failed".to_string()
        })?;
    println!("Database migrations completed successfully.");

    Ok(pool)
}

