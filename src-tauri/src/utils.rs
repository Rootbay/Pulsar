use crate::error::{Error, Result};
use std::path::Path;
use tokio::fs;

pub async fn write_sensitive_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp_path = path.with_extension("tmp");
    if fs::try_exists(&tmp_path).await.unwrap_or(false) {
        let _ = fs::remove_file(&tmp_path).await;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp_path)
            .await?;
        tokio::io::AsyncWriteExt::write_all(&mut file, bytes).await?;
        file.sync_all().await?;
    }

    #[cfg(windows)]
    {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path)
            .await?;
        tokio::io::AsyncWriteExt::write_all(&mut file, bytes).await?;
        file.sync_all().await?;
    }

    #[cfg(not(any(unix, windows)))]
    {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path)
            .await?;
        tokio::io::AsyncWriteExt::write_all(&mut file, bytes).await?;
        file.sync_all().await?;
    }

    if let Err(err) = fs::rename(&tmp_path, path).await {
        if err.kind() == std::io::ErrorKind::Other
            || err.raw_os_error() == Some(17)
            || err.raw_os_error() == Some(18)
        {
            fs::copy(&tmp_path, path).await?;
            let _ = fs::remove_file(&tmp_path).await;
        } else {
            return Err(Error::Io(err));
        }
    }
    Ok(())
}
