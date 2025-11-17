use async_trait::async_trait;
use log::*;
use std::path::PathBuf;
use tokio::{
    fs::read_to_string,
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

use crate::listener::SocketHandler;

const PATH_BASE: &str = "/sys/class/backlight";

async fn get_brightness(path: &PathBuf) -> String {
    read_to_string(path)
        .await
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "120".to_string())
}

pub struct CoolBacklightListener;

#[async_trait]
impl SocketHandler for CoolBacklightListener {
    const SOCKET_NAME: &'static str = "backlight_cool";

    async fn start(&self, unix: &mut tokio::net::UnixStream) {
        info!("Starting CoolBacklightListener");
        let mut path = PathBuf::from(PATH_BASE);
        path.push(Self::SOCKET_NAME);
        path.push("actual_brightness");

        let mut previous_brightness = get_brightness(&path).await;
        self.send_unix(unix, previous_brightness.clone()).await;

        let mut cmd = Command::new("udevadm")
            .arg("monitor")
            .arg("--subsystem-match=backlight")
            .arg("--property")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn udevadm monitor command");

        let stdout = cmd.stdout.take().expect("Failed to take stdout");
        let mut reader = BufReader::new(stdout).lines();

        while let Some(line) = reader
            .next_line()
            .await
            .expect("Failed to read line from udevadm monitor")
        {
            if line.contains("ACTION=change") {
                info!("Backlight change event detected");
                let current_brightness = get_brightness(&path).await;
                if previous_brightness != current_brightness {
                    self.send_unix(unix, current_brightness.clone()).await;
                    previous_brightness = current_brightness;
                } else {
                    debug!("Backlight brightness is the same");
                }
            }
        }
    }
}

pub struct WarmBacklightListener;

#[async_trait]
impl SocketHandler for WarmBacklightListener {
    const SOCKET_NAME: &'static str = "backlight_warm";

    async fn start(&self, unix: &mut tokio::net::UnixStream) {
        info!("Starting WarmBacklightListener");
        let mut path = PathBuf::from(PATH_BASE);
        path.push(Self::SOCKET_NAME);
        path.push("actual_brightness");

        let mut previous_brightness = get_brightness(&path).await;
        self.send_unix(unix, previous_brightness.clone()).await;

        let mut cmd = Command::new("udevadm")
            .arg("monitor")
            .arg("--subsystem-match=backlight")
            .arg("--property")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn udevadm monitor command");

        let stdout = cmd.stdout.take().expect("Failed to take stdout");
        let mut reader = BufReader::new(stdout).lines();

        while let Some(line) = reader
            .next_line()
            .await
            .expect("Failed to read line from udevadm monitor")
        {
            if line.contains("ACTION=change") {
                info!("Backlight change event detected");
                let current_brightness = get_brightness(&path).await;
                if previous_brightness != current_brightness {
                    self.send_unix(unix, current_brightness.clone()).await;
                    previous_brightness = current_brightness;
                } else {
                    debug!("Backlight brightness is the same");
                }
            }
        }
    }
}
