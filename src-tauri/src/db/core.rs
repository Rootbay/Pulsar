use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::env;
use std::path::{Path, PathBuf};
use zeroize::Zeroize;

fn resolve_db_path(db_path: &Path) -> Result<PathBuf, String> {
    let db_path_abs: PathBuf = if db_path.is_absolute() {
        db_path.to_path_buf()
    } else {
        env::current_dir()
            .map_err(|e| {
                eprintln!("Failed to get current dir: {e}");
                "Internal error: Could not resolve database path".to_string()
            })?
            .join(db_path)
    };

    Ok(db_path_abs)
}

fn build_connect_options(
    db_path_abs: &Path,
    password: Option<&[u8]>,
    create_if_missing: bool,
) -> SqliteConnectOptions {
    let path_str = db_path_abs.to_string_lossy();
    let mut opts = SqliteConnectOptions::new()
        .filename(path_str.as_ref())
        .create_if_missing(create_if_missing)
        .busy_timeout(std::time::Duration::from_secs(30));

    if let Some(key_bytes) = password {
        let mut hex_key = hex::encode(key_bytes);
        opts = opts.pragma("key", format!("\"x'{hex_key}'\""));
        hex_key.zeroize();
    }

    opts
}

fn build_pool_options() -> SqlitePoolOptions {
    SqlitePoolOptions::new()
        .max_connections(4)
        .acquire_timeout(std::time::Duration::from_secs(60))
        .after_connect(move |conn, _meta| {
            Box::pin(async move {
                sqlx::query("PRAGMA journal_mode = WAL")
                    .execute(&mut *conn)
                    .await?;

                sqlx::query("PRAGMA synchronous = NORMAL")
                    .execute(&mut *conn)
                    .await?;

                sqlx::query("PRAGMA mmap_size = 268435456")
                    .execute(&mut *conn)
                    .await?;

                sqlx::query("PRAGMA temp_store = MEMORY")
                    .execute(&mut *conn)
                    .await?;

                sqlx::query("PRAGMA cache_size = -2000")
                    .execute(&mut *conn)
                    .await?;

                sqlx::query("PRAGMA foreign_keys = ON")
                    .execute(&mut *conn)
                    .await?;

                sqlx::query("SELECT count(*) FROM sqlite_master")
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| {
                        let msg = e.to_string();
                        if msg.contains("file is not a database") || msg.contains("encrypted") {
                            sqlx::Error::Configuration(
                                "Vault unlock failed. The password may be incorrect or the database is corrupted.".into(),
                            )
                        } else {
                            e
                        }
                    })?;

                Ok(())
            })
        })
}

pub async fn init_db_lazy(
    db_path: &Path,
    password: Option<&[u8]>,
    create_if_missing: bool,
) -> Result<SqlitePool, String> {
    let db_path_abs = resolve_db_path(db_path)?;

    if let Some(parent) = db_path_abs.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            eprintln!(
                "Failed to create database directory {}: {e}",
                parent.display()
            );
            "Failed to access database directory".to_string()
        })?;
    }

    let opts = build_connect_options(db_path_abs.as_path(), password, create_if_missing);

    Ok(build_pool_options().connect_lazy_with(opts))
}
