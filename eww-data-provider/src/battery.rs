use crate::{consts::BATTERY_DEVICE, listener::SocketHandler};
use std::path::PathBuf;
use tokio::{
    fs::read_to_string,
    io::AsyncWriteExt,
    time::{Duration, interval},
};
use log::*;

pub struct BatteryStateListener;

impl SocketHandler for BatteryStateListener {
    const SOCKET_NAME: &'static str = "battery_state";

    async fn start(&self, mut unix: tokio::net::UnixStream) {
        let mut path = PathBuf::from("/sys/class/power_supply/");
        path.push(BATTERY_DEVICE);
        path.push("status");

        let mut last_content: Option<String> = None;
        let mut interval = interval(Duration::from_millis(500));

        loop {
            interval.tick().await;
            let content = read_to_string(&path)
                .await
                .expect("Failed to read battery status");
            let content = content.trim().to_string();

            if last_content.as_ref() != Some(&content) {
                if !content.is_empty() {
                    debug!("Writing state: {}", content);
                    unix.write_all(format!("{}\n", content).as_bytes())
                        .await
                        .expect("Failed to write to socket");
                }
                last_content = Some(content);
            }
        }
    }
}

pub struct BatteryPercentListener;

impl SocketHandler for BatteryPercentListener {
    const SOCKET_NAME: &'static str = "battery_percent";

    async fn start(&self, mut unix: tokio::net::UnixStream) {
        let mut path = PathBuf::from("/sys/class/power_supply/");
        path.push(BATTERY_DEVICE);
        path.push("capacity");

        let mut last_content: Option<String> = None;
        let mut interval = interval(Duration::from_secs(5));

        loop {
            interval.tick().await;
            let content = read_to_string(&path)
                .await
                .expect("Failed to read battery capacity");
            let content = content.trim().to_string();

            if last_content.as_ref() != Some(&content) {
                if !content.is_empty() {
                    unix.write_all(content.as_bytes())
                        .await
                        .expect("Failed to write to socket");
                }
                last_content = Some(content);
            }
        }
    }
}
