use std::{path::{PathBuf, Path}};

use clap::Parser;
use tokio::fs::{File, self};
use tokio_tar::Builder;


#[derive(Debug, Parser)]
struct Cli {
    /// Resulting tar file
    #[arg(short, long)]
    file: PathBuf,

    /// Path to put into tar archive
    path: PathBuf,
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log = mina_logs_service::log::log()?;

    let cli = Cli::parse();

    let file = File::create(cli.file).await?;

    let mut tar = Builder::new(file);

    let mut entries = fs::read_dir(&cli.path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let tar_path = match path.strip_prefix(&cli.path) {
            Ok(p) => p,
            Err(err) => {
                slog::error!(log, "Error stripping prefix {:?} from {:?}: {err}", cli.path, path);
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

    Ok(())
}
