pub mod backlight;
pub mod battery;
pub mod bluetooth;
pub mod dunst;
pub mod eink;
pub mod eink_listener;
// pub mod gamma;
pub mod gestures;
pub mod listener;
pub mod network;
pub mod player;
pub mod requests;
pub mod settingsmenu;
pub mod virtualkeyboard;
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

use crate::dunst::DunstListener;
use crate::eink_listener::EinkListener;
use crate::gestures::GesturesManager;
use crate::settingsmenu::SettingsMenuListener;
use crate::virtualkeyboard::VirtualKeyboardListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("none")).init();
    debug!("Starting eww-data-provider");

    let (tx, _rx) = broadcast::channel::<Requests>(16);
    let request_tx = tx.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) = requests::start_request_listener(request_tx.clone()).await {
                log::error!("Request listener failed: {}", e);
            }
        }
    });

    let mut dunst = DunstListener {
        channel: tx.subscribe(),
    };
    tokio::spawn(async move {
        let mut socket = dunst.open_socket().await;
        dunst.start(&mut socket).await;
    });

    let mut vkeyboard = VirtualKeyboardListener {
        channel: tx.subscribe(),
    };
    tokio::spawn(async move {
        vkeyboard.start().await;
    });

    /*
    let (gamma_channel_tx, gamma_channel_rx) = tokio::sync::mpsc::channel(10);
    let mut gamma = GammaListener {
        channel_rx: tx.subscribe(),
        child: None,
        internal_channel_rx: gamma_channel_rx,
        current_gamma: 0,
    };
    tokio::spawn(async move {
        gamma.start().await;
    });
    */

    let mut eink = EinkListener {
        channel_rx: tx.subscribe(),
        window_settings: true,
        screen_orientation: 90,
        // gamma_channel_tx: gamma_channel_tx,
    };
    tokio::spawn(async move {
        eink.start().await;
    });

    let mut settingsmenu = SettingsMenuListener {
        channel_rx: tx.subscribe(),
        channel_tx: tx.clone(),
    };
    tokio::spawn(async move {
        settingsmenu.start().await;
    });

    let channel = tokio::sync::mpsc::channel(10);
    let mut battery_state = BatteryStateListener {
        channel_tx: channel.0,
    };
    tokio::spawn(async move {
        let mut socket = battery_state.open_socket().await;
        battery_state.start(&mut socket).await;
    });

    let mut battery_percent = BatteryPercentListener {
        channel_rx: channel.1,
    };
    tokio::spawn(async move {
        let mut socket = battery_percent.open_socket().await;
        battery_percent.start(&mut socket).await;
    });

    let mut bluetooth_listener = BluetoothListener;
    tokio::spawn(async move {
        let mut socket = bluetooth_listener.open_socket().await;
        bluetooth_listener.start(&mut socket).await;
    });

    let mut backlight_listener = CoolBacklightListener;
    tokio::spawn(async move {
        let mut socket = backlight_listener.open_socket().await;
        backlight_listener.start(&mut socket).await;
    });

    let mut backlight_warm_listener = WarmBacklightListener;
    tokio::spawn(async move {
        let mut socket = backlight_warm_listener.open_socket().await;
        backlight_warm_listener.start(&mut socket).await;
    });

    let mut player_listener = PlayerListener;
    tokio::spawn(async move {
        let mut socket = player_listener.open_socket().await;
        player_listener.start(&mut socket).await;
    });

    let mut network_listener = NetworkListener;
    tokio::spawn(async move {
        let mut socket = network_listener.open_socket().await;
        network_listener.start(&mut socket).await;
    });

    let mut volume_listener = VolumeListener;
    tokio::spawn(async move {
        let mut socket = volume_listener.open_socket().await;
        volume_listener.start(&mut socket).await;
    });

    let mut gestures_manager = GesturesManager::default();
    tokio::spawn(async move {
        gestures_manager.start().await;
    });

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for event");
    Ok(())
}
