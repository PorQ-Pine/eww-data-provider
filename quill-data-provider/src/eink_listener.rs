use enums::Requests;
use log::{debug, error, info};
use quill_data_provider_lib::{BitDepth, Conversion, DriverMode, Redraw, run_cmd};
use std::time::Duration;
use tokio::time::sleep;

use crate::eink::{
    EwwScreenConfig, eww_screen_config_to_enum, refresh_screen, set_screen_settings,
};

pub struct EinkListener {
    pub channel_rx: tokio::sync::broadcast::Receiver<Requests>,
    pub window_settings: bool,
    pub screen_orientation: u16,
    // pub gamma_channel_tx: tokio::sync::mpsc::Sender<GammaControl>,
}

pub async fn default_set_screen_settings() {
    // Perfect defaults, middle ground between speed and look
    set_screen_settings(
        DriverMode::Normal(BitDepth::Y2(
            Conversion::Thresholding,
            Redraw::DisableFastDrawing,
        )),
        // &mut self.gamma_channel_tx,
        &run_cmd("eww --no-daemonize state").await,
    )
    .await;
}

impl EinkListener {
    pub async fn start(&mut self) {
        info!("Starting EinkListener");
        debug!("Setting initial settings");
        default_set_screen_settings().await;
        loop {
            if let Ok(data) = self.channel_rx.recv().await {
                match data {
                    Requests::ScreenRefresh => {
                        refresh_screen().await;
                    }
                    Requests::ScreenSettings => {
                        self.screen_settings_call(false).await;
                    }
                    Requests::SmallScreenSettings => {
                        self.screen_settings_call(true).await;
                    }
                    _ => {}
                }
            } else {
                error!("Failed to recv");
                sleep(Duration::from_secs(1)).await;
            }
        }
    }

    async fn screen_settings_call(&mut self, _quick: bool) {
        debug!("Got screen settings call");
        let state = &run_cmd("eww --no-daemonize state").await;
        let screen_settings = EwwScreenConfig::get_eww_screen_config(state).await;
        debug!("Screen settings: {:?}", screen_settings);
        // Always handle rotation
        if screen_settings.screen_orientation != self.screen_orientation {
            self.screen_orientation = screen_settings.screen_orientation;
            let rotation_str = match self.screen_orientation {
                0 => "normal",
                90 => "90",
                180 => "180",
                270 => "270",
                _ => {
                    error!("Invalid rotation number?");
                    return;
                }
            };
            run_cmd(&format!("niri msg output DPI-1 transform {}", rotation_str)).await;
            sleep(Duration::from_millis(500)).await;
            run_cmd("eww --no-daemonize close bar").await;
            sleep(Duration::from_millis(100)).await;
            run_cmd("eww --no-daemonize open bar").await;
            return;
        }
        if self.window_settings != screen_settings.window_settings {
            self.window_settings = screen_settings.window_settings;
            if self.window_settings {
                default_set_screen_settings().await;
            }
            screen_settings.set_window_settings().await;
        }
        if !self.window_settings {
            let enum_screen_settings = eww_screen_config_to_enum(&screen_settings).await;
            debug!("Enum screen settings: {:#?}", enum_screen_settings);
            set_screen_settings(
                enum_screen_settings,
                state, // &mut self.gamma_channel_tx
                       // quick
            )
            .await;
        }
    }
}
