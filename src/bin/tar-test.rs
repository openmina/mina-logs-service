use std::{path::{PathBuf, Path}, fs::{File, self, FileType}};

use clap::Parser;
use tar::Builder;


#[derive(Debug, Parser)]
struct Cli {
    /// Resulting tar file
    #[arg(short, long)]
    file: PathBuf,

    /// Path to put into tar archive
    path: PathBuf,
}


fn main() -> anyhow::Result<()> {
    let log = mina_logs_service::log::log()?;

    let cli = Cli::parse();

    let file = File::create(cli.file)?;

    let mut tar = Builder::new(file);

    for entry in fs::read_dir(&cli.path)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                slog::error!(log, "Error getting entry: {err}");
                continue;
            },
        };

        let path = entry.path();
        let tar_path = match path.strip_prefix(&cli.path) {
            Ok(p) => p,
            Err(err) => {
                slog::error!(log, "Error stripping prefix {:?} from {:?}: {err}", cli.path, path);
                continue;
            }
        };

        match entry.file_type() {
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
        if let Err(err) = tar.append_path_with_name(&path, tar_path) {
            slog::error!(log, "Error adding {:?} to tar: {err}", path);
        }
    }
    tar.finish()?;

    Ok(())
}
