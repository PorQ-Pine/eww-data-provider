use inotify::{EventMask, Inotify, WatchMask};
use std::path::PathBuf;
use tokio::{fs::read_to_string, io::AsyncWriteExt};
use crate::{consts::BATTERY_DEVICE, listener::SocketHandler};

pub struct BatteryStateListener;

impl SocketHandler for BatteryStateListener {
    const SOCKET_NAME: &'static str = "battery_state";

    async fn start(&self, mut unix: tokio::net::UnixStream) {
        let mut path = PathBuf::from("/sys/class/power_supply/");
        path.push(BATTERY_DEVICE);
        path.push("status");

        let mut inotify_instance = Inotify::init().expect("Failed to initialize inotify");
        inotify_instance
            .watches()
            .add(&path, WatchMask::MODIFY)
            .expect("Failed to add inotify watch");

        let mut buffer = [0; 1024];
        loop {
            let events = inotify_instance
                .read_events_blocking(&mut buffer)
                .expect("Failed to read inotify events");

            for event in events {
                if event.mask.contains(EventMask::MODIFY) {
                    let content = read_to_string(&path)
                        .await
                        .expect("Failed to read battery status");
                    let content = content.trim();
                    if !content.is_empty() {
                        unix.write_all(content.as_bytes())
                            .await
                            .expect("Failed to write to socket");
                    }
                }
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

        let mut inotify_instance = Inotify::init().expect("Failed to initialize inotify");
        inotify_instance
            .watches()
            .add(&path, WatchMask::MODIFY)
            .expect("Failed to add inotify watch");

        let mut buffer = [0; 1024];
        loop {
            let events = inotify_instance
                .read_events_blocking(&mut buffer)
                .expect("Failed to read inotify events");

            for event in events {
                if event.mask.contains(EventMask::MODIFY) {
                    let content = read_to_string(&path)
                        .await
                        .expect("Failed to read battery capacity");
                    let content = content.trim();
                    if !content.is_empty() {
                        unix.write_all(content.as_bytes())
                            .await
                            .expect("Failed to write to socket");
                    }
                }
            }
        }
    }
}
