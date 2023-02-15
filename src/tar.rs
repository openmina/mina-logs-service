use std::{path::Path, io::Result};

use tokio_tar::Builder;
use tokio::io::AsyncWrite;

pub async fn tar_files<W: AsyncWrite + Send + Unpin + 'static>(dir: &Path, write: W) -> Result<W> {
    let mut tar = Builder::new(write);
    tar.append_dir_all(".", dir).await?;
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

        let dir = tempdir::TempDir::new("dir")?;
        fs::write(dir.path().join("a.txt"), "file a".as_bytes()).await?;
        fs::write(dir.path().join("b.txt"), "file b".as_bytes()).await?;

        let tar = super::tar_files(dir.path(), Vec::new()).await?;

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
