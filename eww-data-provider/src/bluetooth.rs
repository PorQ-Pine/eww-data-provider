use serde::{Serialize, Deserialize};
use tokio::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct BluetoothStatus {
    on: bool,
    name: String,
    signal: String,
}

pub async fn get_bt() -> String {
    let mut status = BluetoothStatus {
        on: false,
        name: "".to_string(),
        signal: "".to_string(),
    };

    let output = Command::new("bluetoothctl")
        .arg("show")
        .output()
        .await
        .expect("Failed to execute bluetoothctl show");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let power_line = stdout.lines().find(|line| line.contains("Powered:"));

    if let Some(line) = power_line {
        if line.contains("yes") {
            status.on = true;
        }
    }

    if !status.on {
        return serde_json::to_string(&status).unwrap();
    }

    let devices_output = Command::new("bluetoothctl")
        .arg("devices")
        .output()
        .await
        .expect("Failed to execute bluetoothctl devices");

    let devices_stdout = String::from_utf8_lossy(&devices_output.stdout);
    let mut connected_mac: Option<String> = None;

    for line in devices_stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let mac = parts[1].to_string();
            let info_output = Command::new("bluetoothctl")
                .arg("info")
                .arg(&mac)
                .output()
                .await
                .expect("Failed to execute bluetoothctl info");

            let info_stdout = String::from_utf8_lossy(&info_output.stdout);
            if info_stdout.contains("Connected: yes") {
                connected_mac = Some(mac);
                break;
            }
        }
    }

    if let Some(mac) = connected_mac {
        let info_output = Command::new("bluetoothctl")
            .arg("info")
            .arg(&mac)
            .output()
            .await
            .expect("Failed to execute bluetoothctl info");

        let info_stdout = String::from_utf8_lossy(&info_output.stdout);

        if let Some(name_line) = info_stdout.lines().find(|line| line.contains("Name:")) {
            status.name = name_line
                .trim_start_matches("Name:")
                .trim()
                .trim_matches('"')
                .to_string();
        }

        if let Some(rssi_line) = info_stdout.lines().find(|line| line.contains("RSSI:")) {
            let parts: Vec<&str> = rssi_line.split_whitespace().collect();
            if parts.len() >= 2 {
                status.signal = parts[1].to_string();
            }
        }
    }

    serde_json::to_string(&status).unwrap()
}
