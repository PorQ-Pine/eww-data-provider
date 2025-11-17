use std::time::Duration;

use async_trait::async_trait;
use log::*;
use tokio::{io::AsyncWriteExt, time::sleep};

#[async_trait]
pub trait SocketHandler {
    const SOCKET_NAME: &'static str;

    async fn open_socket(&self) -> tokio::net::UnixStream {
        let socket_path = format!("/tmp/eww_data/{}.socket", Self::SOCKET_NAME);
        let path = std::path::Path::new(&socket_path);

        loop {
            match tokio::net::UnixStream::connect(&path).await {
                Ok(stream) => {
                    info!("Successfully connected to socket: {}", socket_path);
                    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                    return stream;
                }
                Err(_e) => {
                    // debug!("Waiting for socket at {}: {}", socket_path, e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
            }
        }
    }

    async fn start(&self, unix: &mut tokio::net::UnixStream);

    async fn send_unix(&self, unix: &mut tokio::net::UnixStream, mut str: String) {
        // debug!("To {:?} sending: {}", unix.peer_addr(), str);
        str.push('\n');
        if let Err(e) = unix.write_all(str.as_bytes()).await {
            error!("Failed to write to socket: {}", e);
            *unix = self.open_socket().await;
            sleep(Duration::from_millis(100)).await;
            self.send_unix(unix, str).await;
        }
    }
}
