use log::{error, info, warn};
use niri_ipc::Transform::*;
use niri_ipc::{Event, Response};
use niri_ipc::{Request, Transform, socket::Socket};
use std::time::Duration;
use tokio::{process::Command, sync::mpsc};

#[derive(Default)]
pub struct GesturesManager {
    child: Option<tokio::process::Child>,
}

const OUTPUT_NAME: &str = "DPI-1";

impl GesturesManager {
    pub async fn start(&mut self) {
        let mut socket = get_socket().await;

        let reply = socket.send(Request::EventStream).unwrap();
        if !matches!(reply, Ok(Response::Handled)) {
            error!("Failed to subscribe to Niri IPC EventStream.");
            return;
        }

        let (evt_tx, mut evt_rx) = mpsc::unbounded_channel();

        tokio::task::spawn_blocking(move || {
            let mut read_event = socket.read_events();
            while let Ok(event) = read_event() {
                let _ = evt_tx.send(event);
            }
        });
        let mut socket = get_socket().await;

        let mut width_s: i32 = 0;
        let mut height_s: i32 = 0;
        let mut transform_s: Transform = _90;

        if let Some((width, height, transform)) = get_screen_dimensions(&mut socket) {
            width_s = width;
            height_s = height;
            transform_s = transform;
        }
        loop {
            if let Some(child) = &mut self.child {
                tokio::select! {
                    Some(event) = evt_rx.recv() => {
                        if let Event::WindowLayoutsChanged {..} = event {
                            if let Some((width, height, transform)) = get_screen_dimensions(&mut socket) {
                                if width != width_s || height != height_s || transform != transform_s {
                                    width_s = width;
                                    height_s = height;
                                    transform_s = transform;
                                    self.restart_lisgd(width_s, height_s, transform_s).await;
                                }
                            }
                        }
                    }
                    _ = child.wait() => {
                        self.restart_lisgd(width_s, height_s, transform_s).await;
                    }
                }
            } else {
                warn!("There is no lisgd running");
                self.restart_lisgd(width_s, height_s, transform_s).await;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    async fn restart_lisgd(&mut self, width: i32, height: i32, transform: Transform) {
        if let Some(mut old_child) = self.child.take() {
            info!("Display layout change detected. Killing old lisgd instance...");
            let _ = old_child.kill().await;
        }

        let orientation = match transform {
            Transform::Normal => "0",
            Transform::_90 => "3",
            Transform::_180 => "2",
            Transform::_270 => "1",
            _ => "0",
        };

        let command_str = format!(
            r#"lisgd -d /dev/input/by-path/platform-fe5e0000.i2c-event \
                  -w {} -h {} -o {} \
                  -g "2,LR,*,M,R,niri msg action focus-column-left" \
                  -g "2,RL,*,M,R,niri msg action focus-column-right""#,
            width, height, orientation
        );

        info!(
            "Spawning lisgd ({}x{}, orientation flag: {})",
            width, height, orientation
        );

        match Command::new("sh").arg("-c").arg(&command_str).spawn() {
            Ok(child) => self.child = Some(child),
            Err(e) => error!("Failed to spawn lisgd: {}", e),
        }
    }
}

fn get_screen_dimensions(socket: &mut Socket) -> Option<(i32, i32, Transform)> {
    let outputs = try_fetch(socket, Request::Outputs, |r| match r {
        Response::Outputs(w) => Some(w),
        _ => None,
    })?;

    let output = match outputs.get(OUTPUT_NAME) {
        Some(o) => o,
        None => {
            error!("Error: Output '{}' not found.", OUTPUT_NAME);
            return None;
        }
    };

    match output.logical {
        Some(logical_mode) => Some((
            logical_mode.width as i32,
            logical_mode.height as i32,
            logical_mode.transform,
        )),
        None => {
            error!("No logical screen info");
            None
        }
    }
}

async fn get_socket() -> Socket {
    let base_run_dir = "/run/user";

    loop {
        // 1. Read the base /run/user directory
        if let Ok(mut user_dirs) = tokio::fs::read_dir(base_run_dir).await {
            // Iterate through user folders (e.g., /run/user/1000)
            while let Ok(Some(user_entry)) = user_dirs.next_entry().await {
                let user_path = user_entry.path();

                if user_path.is_dir() {
                    // 2. Read the contents of each user directory
                    if let Ok(mut sockets) = tokio::fs::read_dir(&user_path).await {
                        while let Ok(Some(socket_entry)) = sockets.next_entry().await {
                            let path = socket_entry.path();
                            let file_name = path.file_name().unwrap_or_default().to_string_lossy();

                            // 3. Check for niri-wayland prefix
                            if file_name.starts_with("niri.wayland-") {
                                // Try to connect; if it succeeds, return the socket
                                if let Ok(socket) = Socket::connect_to(&path) {
                                    return socket;
                                }
                            }
                        }
                    }
                }
            }
        }

        // 4. Wait a second before re-scanning the filesystem
        tokio::time::sleep(Duration::from_secs(1)).await;
        warn!("Waiting for niri socket...");
    }
}

fn try_fetch<T, F>(socket: &mut Socket, req: Request, extract: F) -> Option<T>
where
    F: FnOnce(Response) -> Option<T>,
{
    match socket.send(req).unwrap() {
        Ok(res) => extract(res).or_else(|| {
            error!("Error: Received unexpected response variant.");
            None
        }),
        Err(e) => {
            error!("Error: Failed to get reply: {:?}", e);
            None
        }
    }
}
