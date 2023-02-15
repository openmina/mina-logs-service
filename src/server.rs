use std::{convert::Infallible, path::PathBuf};

use warp::{
    http::{Response, StatusCode},
    reject::Reject,
    Filter, Server,
};

#[derive(Debug)]
enum TarError {
    IO(std::io::Error),
    Http(warp::http::Error),
}

impl From<std::io::Error> for TarError {
    fn from(source: std::io::Error) -> Self {
        TarError::IO(source)
    }
}

impl From<warp::http::Error> for TarError {
    fn from(source: warp::http::Error) -> Self {
        TarError::Http(source)
    }
}

impl Reject for TarError {}

async fn tar(dir: PathBuf) -> Result<Response<Vec<u8>>, warp::Rejection> {
    let body = super::tar::tar_files(&dir.as_path(), Vec::new())
        .await
        .map_err(|err| {
            eprintln!("error: {err}");
            warp::reject::custom(TarError::IO(err))
        })?;
    let response = Response::builder()
        .header("content-type", "application/x-tar")
        .body(body)
        .map_err(|err| {
            eprintln!("error: {err}");
            warp::reject::custom(TarError::Http(err))
        })?;
    Ok(response)
}

async fn handle_rejection(err: warp::Rejection) -> Result<Box<dyn warp::Reply>, Infallible> {
    let (code, message) = if let Some(TarError::IO(err)) = err.find() {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("tar error: {err}"),
        )
    } else if let Some(TarError::Http(err)) = err.find() {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("response error: {err}"),
        )
    } else {
        eprintln!("unhandled rejection: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "UNHANDLED_REJECTION".to_string(),
        )
    };

    let json = warp::reply::json(&serde_json::json!({
        "code": code.as_u16(),
        "message": message,
    }));

    Ok(Box::new(warp::reply::with_status(json, code)))
}

pub async fn serve(
    dir: PathBuf,
) -> Server<impl Filter<Extract = (impl warp::Reply,), Error = Infallible> + Clone> {
    let with_path = warp::any().map(move || dir.clone());
    let download = warp::path("download").and(with_path).and_then(tar);

    let paths = download.recover(handle_rejection);

    let server = warp::serve(paths);
    server
}

#[cfg(test)]
mod tests {
    use std::pin::Pin;

    use tokio::{io::AsyncReadExt, sync::oneshot};
    use tokio_stream::StreamExt;
    use tokio_tar::Archive;

    #[tokio::test]
    async fn test_server() -> anyhow::Result<()> {
        use tokio::fs;
        let dir = tempdir::TempDir::new("dir")?;
        fs::write(dir.path().join("a.txt"), "file a".as_bytes()).await?;
        fs::write(dir.path().join("b.txt"), "file b".as_bytes()).await?;

        let server = super::serve(dir.path().into()).await;
        let (tx, rx) = oneshot::channel();
        let (addr, server) = server.bind_with_graceful_shutdown(([127, 0, 0, 1], 0), async {
            rx.await.ok();
        });
        tokio::task::spawn(server);

        let bytes = reqwest::get(format!("http://{addr}/download"))
            .await?
            .bytes()
            .await?;

        let mut slice: &[u8] = &bytes;
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

        let _ = tx.send(());
        Ok(())
    }

    #[tokio::test]
    async fn incorrect_dir() -> anyhow::Result<()> {
        let dir = tempdir::TempDir::new("dir")?;
        let server = super::serve(dir.path().join("/nonexisting-path")).await;
        let (tx, rx) = oneshot::channel();
        let (addr, server) = server.bind_with_graceful_shutdown(([127, 0, 0, 1], 0), async {
            rx.await.ok();
        });
        tokio::task::spawn(server);

        let response = reqwest::get(format!("http://{addr}/download")).await?;
        println!("{response:#?}");

        let _ = tx.send(());
        Ok(())
    }
}
