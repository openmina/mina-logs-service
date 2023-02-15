use std::{net::SocketAddr, path::PathBuf};

use async_ctrlc::CtrlC;
use clap::Parser;
use mina_logs_service::server;
use tokio::sync::oneshot;


#[derive(Debug, Parser)]
struct Cli {
    #[arg(short, long, default_value_t = SocketAddr::from(([127, 0, 0, 1], 0)))]
    address: SocketAddr,
    #[arg(short, long)]
    dir: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let server = server::serve(cli.dir).await;
    let (tx, rx) = oneshot::channel();
    let (addr, server) = server.bind_with_graceful_shutdown(cli.address, async {
        rx.await.ok();
    });
    println!("Server is ready at {addr}");

    let ctrlc = CtrlC::new()?;
    tokio::task::spawn(server);
    ctrlc.await;
    let _ = tx.send(());
    Ok(())
}
