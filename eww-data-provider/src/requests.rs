use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
    sync::broadcast,
};
use enums::Requests;
use serde_json;

const SOCKET_PATH: &str = "/tmp/eww_data/requests.socket";

pub async fn start_request_listener(tx: broadcast::Sender<Requests>) -> Result<()> {
    tokio::fs::create_dir_all("/tmp/eww_data").await?;

    if tokio::fs::metadata(SOCKET_PATH).await.is_ok() {
        tokio::fs::remove_file(SOCKET_PATH).await?;
    }

    let listener = UnixListener::bind(SOCKET_PATH)?;
    log::info!("Request listener started on {}", SOCKET_PATH);

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                log::debug!("New client connected to request socket");
                handle_client(stream, tx.clone()).await.ok();
            }
            Err(e) => {
                log::error!("Failed to accept request client: {}", e);
            }
        }
    }
}

async fn handle_client(mut stream: UnixStream, tx: broadcast::Sender<Requests>) -> Result<()> {
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;

    let request: Requests = serde_json::from_slice(&buffer)?;
    log::debug!("Received request: {:?}", request);

    tx.send(request)?;

    Ok(())
}
