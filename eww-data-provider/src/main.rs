pub mod backlight;
pub mod battery;
pub mod bluetooth;
pub mod consts;
pub mod dunst;
pub mod listener;
pub mod network;
pub mod player;
pub mod requests;
pub mod volume;

use backlight::CoolBacklightListener;
use backlight::WarmBacklightListener;
use battery::{BatteryPercentListener, BatteryStateListener};
use bluetooth::BluetoothListener;
use enums::Requests;
use listener::SocketHandler;
use log::*;
use network::NetworkListener;
use player::PlayerListener;
use tokio::sync::broadcast;
use volume::VolumeListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    debug!("Starting eww-data-provider");

    let (tx, _rx) = broadcast::channel::<Requests>(16);

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

    let player_listener = PlayerListener;
    tokio::spawn(async move {
        let mut socket = player_listener.open_socket().await;
        player_listener.start(&mut socket).await;
    });

    let network_listener = NetworkListener;
    tokio::spawn(async move {
        let mut socket = network_listener.open_socket().await;
        network_listener.start(&mut socket).await;
    });

    let volume_listener = VolumeListener;
    tokio::spawn(async move {
        let mut socket = volume_listener.open_socket().await;
        volume_listener.start(&mut socket).await;
    });

    let request_tx = tx.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) = requests::start_request_listener(request_tx.clone()).await {
                log::error!("Request listener failed: {}", e);
            }
        }
    });

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for event");
    Ok(())
}
