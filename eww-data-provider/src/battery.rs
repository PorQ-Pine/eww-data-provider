use crate::listener::SocketHandler;
use async_trait::async_trait;
use log::*;
use std::{path::PathBuf, time::Duration};
use tokio::{
    fs::read_to_string,
    io::{AsyncBufReadExt, BufReader},
    process::Command, time::sleep,
};

pub const BATTERY_DEVICE: &'static str = "rk817-battery";

async fn get_battery_info(path: &PathBuf) -> String {
    read_to_string(path)
        .await
        .unwrap_or_else(|_| String::from("50"))
        .trim()
        .to_string()
}

pub struct BatteryStateListener;

#[async_trait]
impl SocketHandler for BatteryStateListener {
    const SOCKET_NAME: &'static str = "battery_state";

    async fn start(&mut self, unix: &mut tokio::net::UnixStream) {
        info!("Starting BatteryStateListener");
        let mut path = PathBuf::from("/sys/class/power_supply/");
        path.push(BATTERY_DEVICE);
        path.push("status");

        let mut previous_state = get_battery_info(&path).await;
        self.send_unix(unix, previous_state.clone()).await;

        let mut cmd = Command::new("udevadm")
            .arg("monitor")
            .arg("--subsystem-match=power_supply")
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
            // if line.contains("ACTION=change") {
            sleep(Duration::from_millis(100)).await;
            info!("Battery state change event detected");
            let current_state = get_battery_info(&path).await;
            if previous_state != current_state {
                self.send_unix(unix, current_state.clone()).await;
                previous_state = current_state;
            } else {
                debug!("Battery state is the same");
            }
            // }
        }
    }
}

pub struct BatteryPercentListener;

#[async_trait]
impl SocketHandler for BatteryPercentListener {
    const SOCKET_NAME: &'static str = "battery_percent";

    async fn start(&mut self, unix: &mut tokio::net::UnixStream) {
        info!("Starting BatteryPercentListener");
        let mut path = PathBuf::from("/sys/class/power_supply/");
        path.push(BATTERY_DEVICE);
        path.push("capacity");

        let mut previous_percent = get_battery_info(&path).await;
        self.send_unix(unix, previous_percent.clone()).await;

        let mut cmd = Command::new("udevadm")
            .arg("monitor")
            .arg("--subsystem-match=power_supply")
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
            // if line.contains("ACTION=change") {
            sleep(Duration::from_millis(250)).await;
            info!("Battery percent change event detected");
            let current_percent = get_battery_info(&path).await;
            if previous_percent != current_percent {
                self.send_unix(unix, current_percent.clone()).await;
                previous_percent = current_percent;
            } else {
                debug!("Battery percent is the same");
            }
            // }
        }
    }
}
