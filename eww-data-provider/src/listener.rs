pub trait SocketHandler {
    const SOCKET_NAME: &'static str;

    async fn open_socket(&self) -> tokio::net::UnixStream {
        let socket_path = format!("/tmp/eww_data/{}.socket", Self::SOCKET_NAME);
        let path = std::path::Path::new(&socket_path);

        loop {
            match tokio::net::UnixStream::connect(&path).await {
                Ok(stream) => {
                    eprintln!("Successfully connected to socket: {}", socket_path);
                    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                    return stream;
                }
                Err(e) => {
                    eprintln!("Waiting for socket at {}: {}", socket_path, e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    }

    async fn start(&self, unix: tokio::net::UnixStream);
}
