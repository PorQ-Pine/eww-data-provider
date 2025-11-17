pub mod listener;
pub mod consts;
pub mod battery;
pub mod bluetooth;
pub mod backlight;
pub mod dunst; // Add this line

use listener::SocketHandler;
use battery::{BatteryStateListener, BatteryPercentListener};
use bluetooth::BluetoothListener;
use backlight::CoolBacklightListener;
use backlight::WarmBacklightListener;
use log::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    debug!("Starting eww-data-provider");

    let battery_state = BatteryStateListener;
    tokio::spawn(async move {
        let mut socket = battery_state.open_socket().await;
        battery_state.start(&mut socket).await;
    });

    let battery_percent = BatteryPercentListener;
    tokio::spawn(async move {
        let mut socket = battery_percent.open_socket().await;
        battery_percent.start(&mut socket).await;
    });

    let bluetooth_listener = BluetoothListener;
    tokio::spawn(async move {
        let mut socket = bluetooth_listener.open_socket().await;
        bluetooth_listener.start(&mut socket).await;
    });

    let backlight_listener = CoolBacklightListener;
    tokio::spawn(async move {
        let mut socket = backlight_listener.open_socket().await;
        backlight_listener.start(&mut socket).await;
    });

    let backlight_warm_listener = WarmBacklightListener;
    tokio::spawn(async move {
        let mut socket = backlight_warm_listener.open_socket().await;
        backlight_warm_listener.start(&mut socket).await;
    });

    tokio::signal::ctrl_c().await.expect("failed to listen for event");
    Ok(())
}
