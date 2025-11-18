use async_trait::async_trait;
use log::*;
use std::{path::PathBuf, time::Duration};
use tokio::{
    fs::read_to_string,
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    time::sleep,
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

    async fn start(&mut self, unix: &mut tokio::net::UnixStream) {
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

        while let Some(_line) = reader
            .next_line()
            .await
            .expect("Failed to read line from udevadm monitor")
        {
            //if line.contains("ACTION=change") {
            // info!("Backlight change event detected");
            sleep(Duration::from_millis(5)).await;
            let current_brightness = get_brightness(&path).await;
            debug!("current_brightness: {}, previous_brightness: {}", current_brightness, previous_brightness);
            if previous_brightness != current_brightness {
                debug!("Sending cool brightness: {}", current_brightness);
                let new_brightness = ((current_brightness.parse::<u16>().unwrap() * 100 / 255) as u8).to_string();
                self.send_unix(unix, new_brightness).await;
                previous_brightness = current_brightness;
            } else {
                debug!("Backlight brightness is the same");
            }
            //}
        }
    }
}

pub struct WarmBacklightListener;

#[async_trait]
impl SocketHandler for WarmBacklightListener {
    const SOCKET_NAME: &'static str = "backlight_warm";

    async fn start(&mut self, unix: &mut tokio::net::UnixStream) {
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

        while let Some(_line) = reader
            .next_line()
            .await
            .expect("Failed to read line from udevadm monitor")
        {
            //if line.contains("ACTION=change") {
            //info!("Backlight change event detected");
            sleep(Duration::from_millis(5)).await;
            let current_brightness = get_brightness(&path).await;
            if previous_brightness != current_brightness {
                debug!("Sending warm brightness: {}", current_brightness);
                let new_brightness = ((current_brightness.parse::<u16>().unwrap() * 100 / 255) as u8).to_string();
                self.send_unix(unix, new_brightness).await;
                previous_brightness = current_brightness;
            } else {
                debug!("Backlight brightness is the same");
            }
            //}
        }
    }
}
