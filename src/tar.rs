use std::{path::Path, io::Result};

use slog::Logger;
use tokio_tar::Builder;
use tokio::{io::AsyncWrite, fs};

pub async fn tar_files<W: AsyncWrite + Send + Unpin + 'static>(dir: &Path, write: W, log: Logger) -> Result<W> {
    let mut tar = Builder::new(write);

    let mut entries = fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let tar_path = match path.strip_prefix(dir) {
            Ok(p) => p,
            Err(err) => {
                slog::error!(log, "Error stripping prefix {:?} from {:?}: {err}", dir, path);
                continue;
            }
        };

        match entry.file_type().await {
            Ok(ft) if ft.is_file() => {}
            Ok(_) => {
                slog::warn!(log, "Ignoring non-file entry {:?}", path);
                continue;
            }
            Err(err) => {
                slog::error!(log, "Error getting file type for {:?}: {err}", path);
                continue;
            }
        }

        slog::info!(log, "Adding {:?} to tar", path);
        if let Err(err) = tar.append_path_with_name(&path, tar_path).await {
            slog::error!(log, "Error adding {:?} to tar: {err}", path);
        }
    }

    tar.finish().await?;
    tar.into_inner().await
}

#[cfg(test)]
mod tests {
    use std::pin::Pin;

    use tokio::io::AsyncReadExt;
    use tokio_stream::StreamExt;
    use tokio_tar::Archive;


    #[tokio::test]
    async fn tar_test() -> anyhow::Result<()> {
        use tokio::fs;

        let log = crate::log::log().unwrap();

        let dir = tempdir::TempDir::new("dir")?;
        fs::write(dir.path().join("a.txt"), "file a".as_bytes()).await?;
        fs::write(dir.path().join("b.txt"), "file b".as_bytes()).await?;

        let tar = super::tar_files(dir.path(), Vec::new(), log).await?;

        let mut slice = tar.as_slice();
        let mut archive = Archive::new(&mut slice);
        let mut entries = archive.entries()?;
        let mut pinned = Pin::new(&mut entries);
        let mut entries = Vec::new();
        while let Some(entry) = pinned.next().await {
            let mut entry = entry?;
            if entry.header().entry_type().is_file() {
                let path = entry.path()?.into_owned();
                let mut content = String::new();
                let _ = entry.read_to_string(&mut content).await?;
                entries.push((path, content));
            }
        }
        assert_eq!(entries.len(), 2);
        assert!(entries.contains(&("a.txt".into(), "file a".into())));
        assert!(entries.contains(&("b.txt".into(), "file b".into())));
        Ok(())
    }

}
