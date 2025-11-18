use enums::Requests;
use log::{debug, error, info, warn};
use std::time::{Duration, Instant};
use tokio::{process::Command, time::sleep};

pub struct SettingsMenuListener {
    pub channel_rx: tokio::sync::broadcast::Receiver<Requests>,
    pub channel_tx: tokio::sync::broadcast::Sender<Requests>,
}

impl SettingsMenuListener {
    pub async fn start(&mut self) {
        info!("Starting SettingsMenuListener");
        let mut latest_call = Instant::now() - Duration::from_secs(60);
        loop {
            if let Ok(data) = self.channel_rx.recv().await {
                if data == Requests::SettingsMenu {
                    if Instant::now().duration_since(latest_call) > Duration::from_millis(150) {
                        debug!("It is a settings menu call");
                        let mut counter = 1;
                        let is_visible = Self::is_visible().await;
                        if !is_visible {
                            self.channel_tx.send(Requests::Notifications).ok();
                        }
                        let mut new_is_visible = is_visible;
                        while new_is_visible == is_visible {
                            if counter > 1 {
                                warn!("Failed to toggle window");
                                if counter > 7 {
                                    error!("Critical toggle window");
                                    break;
                                }
                            }
                            Self::window_manage(!is_visible).await;
                            sleep(Duration::from_millis(50 * counter)).await;
                            new_is_visible = Self::is_visible().await;
                            counter += 1;
                            latest_call = Instant::now();
                        }
                        while !self.channel_rx.is_empty() {
                            self.channel_rx.recv().await.ok();
                        }
                    } else {
                        warn!("Ignoring call to settings menu, too quick!");
                    }
                }
            } else {
                error!("Failed to recv");
                sleep(Duration::from_secs(1)).await;
            }
        }
    }

    async fn is_visible() -> bool {
        let output = Command::new("eww")
            .args(&["--no-daemonize", "active-windows"])
            .output()
            .await
            .expect("failed to execute command");

        let result = String::from_utf8_lossy(&output.stdout);
        if result.contains("control_center") {
            return true;
        }
        false
    }

    async fn window_manage(state: bool) {
        if state {
            Self::open().await;
        } else {
            Self::close().await;
        }
    }

    async fn close() {
        let _output = Command::new("eww")
            .args(&["--no-daemonize", "close", "control_center"])
            .output()
            .await
            .expect("failed to execute command");
    }

    async fn open() {
        let _output = Command::new("eww")
            .args(&["--no-daemonize", "open", "control_center"])
            .output()
            .await
            .expect("failed to execute command");
    }
}
