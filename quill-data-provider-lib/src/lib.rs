use eframe::egui;
use enum2egui::{Gui, GuiInspect};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

// The mess of connected enums is so we know what affects when, so:
// - We can set only what's needed
// - We show only what can be changed

// Only matters when:
// DriverMode is Fast
// Normal, Y2 and Y1
// busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DitherMode y 0
#[derive(Copy, Clone, Debug, PartialEq, Gui, Default, Serialize, Deserialize)]
pub enum Dithering {
    #[default]
    Bayer, // 0
    BlueNoise16, // 1
    BlueNoise32, // 2
}

impl std::fmt::Display for Dithering {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DriverMode y 0
#[derive(Copy, Clone, Debug, PartialEq, Gui, Serialize, Deserialize)]
pub enum DriverMode {
    Normal(#[enum2egui(label = "Bit depth")] BitDepth), // 0
    Fast(#[enum2egui(label = "Dithering type")] Dithering), // 1
                                                        // Doesn't work for me
                                                        // Zero, // 8
}

impl std::fmt::Display for DriverMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for DriverMode {
    fn default() -> Self {
        DriverMode::Normal(BitDepth::Y2(
            Conversion::Thresholding,
            Redraw::DisableFastDrawing,
        ))
    }
}

// RenderHints
// Only matters in Normal mode
// busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DefaultHintHr s "Y1|T|R"
#[derive(Copy, Clone, Debug, PartialEq, Gui, Serialize, Deserialize)]
pub enum BitDepth {
    Y1(
        #[enum2egui(label = "Conversion")] Conversion,
        ThresholdLevel,
    ),
    Y2(
        #[enum2egui(label = "Conversion")] Conversion,
        #[enum2egui(label = "Fast redraw")] Redraw,
    ),
    Y4(#[enum2egui(label = "Fast redraw")] Redraw),
}

impl Default for BitDepth {
    fn default() -> Self {
        BitDepth::Y2(Conversion::default(), Redraw::default())
    }
}

impl std::fmt::Display for BitDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Gui, Serialize, Deserialize)]
pub enum Conversion {
    Thresholding,                                                // T, + level
    Dithering(#[enum2egui(label = "Dithering type")] Dithering), // D
}

impl Default for Conversion {
    fn default() -> Self {
        Conversion::Thresholding
    }
}

impl std::fmt::Display for Conversion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// So the configurator works well...
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Gui, Default, Serialize, Deserialize)]
pub enum ThresholdLevel {
    _2,
    _3,
    _4,
    _5,
    _6,
    #[default]
    _7,
    _8,
    _9,
    _10,
    _11,
    _12,
    _13,
    _14,
    _15,
}

impl ThresholdLevel {
    pub async fn set(&self) {
        /*
        if level < 2 || level > 15 {
            error!("Wrong treshold level");
        }
        */

        let level: u8 = self.to_u8();
        debug!("Writing Y1 threshold level: {}", level);
        if let Err(e) = tokio::fs::write(
            "/sys/module/rockchip_ebc_blit_neon/parameters/y4_threshold_y1",
            level.to_string(),
        )
        .await
        {
            error!("Failed to set threshold: {}", e);
        }
    }

    pub fn get_from_eww(level: u8) -> Self {
        let converted = 2 + ((level.saturating_sub(1) as f32 / 99.0) * 13.0).round() as u8;
        ThresholdLevel::try_from(converted).unwrap()
    }

    pub async fn set_eww_number(&self) {
        let level: u8 = self.to_u8();
        run_cmd(&format!(
            "eww --no-daemonize update thresholding_level_value_real={}",
            level
        ))
        .await;
    }

    /*
    pub async fn set_Thresholding_level(level: u8, show_gui: bool) {
        let converted = 2 + ((level.saturating_sub(1) as f32 / 99.0) * 13.0).round() as u8;
        Self::set_Thresholding_level_internal(converted).await;

        if show_gui {
            run_cmd(&format!(
                "eww --no-daemonize update thresholding_level_value_real={}",
                converted
            ))
            .await;
        }
    }
    */

    pub fn to_u8(&self) -> u8 {
        match self {
            ThresholdLevel::_2 => 2,
            ThresholdLevel::_3 => 3,
            ThresholdLevel::_4 => 4,
            ThresholdLevel::_5 => 5,
            ThresholdLevel::_6 => 6,
            ThresholdLevel::_7 => 7,
            ThresholdLevel::_8 => 8,
            ThresholdLevel::_9 => 9,
            ThresholdLevel::_10 => 10,
            ThresholdLevel::_11 => 11,
            ThresholdLevel::_12 => 12,
            ThresholdLevel::_13 => 13,
            ThresholdLevel::_14 => 14,
            ThresholdLevel::_15 => 15,
        }
    }
}

impl TryFrom<u8> for ThresholdLevel {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            2 => Ok(ThresholdLevel::_2),
            3 => Ok(ThresholdLevel::_3),
            4 => Ok(ThresholdLevel::_4),
            5 => Ok(ThresholdLevel::_5),
            6 => Ok(ThresholdLevel::_6),
            7 => Ok(ThresholdLevel::_7),
            8 => Ok(ThresholdLevel::_8),
            9 => Ok(ThresholdLevel::_9),
            10 => Ok(ThresholdLevel::_10),
            11 => Ok(ThresholdLevel::_11),
            12 => Ok(ThresholdLevel::_12),
            13 => Ok(ThresholdLevel::_13),
            14 => Ok(ThresholdLevel::_14),
            15 => Ok(ThresholdLevel::_15),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for ThresholdLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Gui, Default, Serialize, Deserialize)]
pub enum Redraw {
    FastDrawing(#[enum2egui(label = "")] RedrawOptions), // R
    #[default]
    DisableFastDrawing,                    // r
}

#[derive(Copy, Clone, Debug, PartialEq, Gui, Serialize, Deserialize)]
pub struct RedrawOptions {
    #[enum2egui(label = "\nRedraw delay (10-300 is reasonable)")]
    pub delay: u16,
}

impl RedrawOptions {
    pub async fn set(&self) {
        run_cmd(&format!("busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 RedrawDelay q {}", self.delay)).await;
    }
}

impl Default for RedrawOptions {
    fn default() -> Self {
        Self { delay: 25 }
    }
}

impl std::fmt::Display for Redraw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Impl
impl DriverMode {
    pub async fn set(&self) {
        let string = match self {
            DriverMode::Normal(_bit_depth) => "0",
            DriverMode::Fast(_dithering) => "1",
        };
        let line = format!(
            "busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DriverMode y {}",
            string
        );
        run_cmd(&line).await;
    }
}

impl Dithering {
    pub async fn set(&self) {
        let string = match self {
            Dithering::Bayer => "0",
            Dithering::BlueNoise16 => "1",
            Dithering::BlueNoise32 => "2",
        };
        let line = format!(
            "busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DitherMode y {}",
            string
        );
        run_cmd(&line).await;
    }
}

pub async fn run_cmd(line: &str) -> String {
    let parts: Vec<&str> = line.split_whitespace().collect();
    debug!("Running run_cmd as: {} {:?}", parts[0], &parts[1..]);
    let out = Command::new(parts[0])
        .args(&parts[1..])
        .output()
        .await
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[derive(Clone, Debug, PartialEq, Gui, Default, Serialize, Deserialize)]
pub struct EinkWindowSetting {
    pub app_id: String,
    pub settings: DriverMode,
}

static DEFAULT_WINDOW_SETTINGS: &str =
    include_str!("../../eink-window-settings/other/default/config.ron");
pub const WINDOW_SETTINGS_HOME_CONFIG_DIR: &str = "/.config/eink-window-settings/";
pub const WINDOW_SETTINGS_CONFIG_NAME: &str = "config.ron";

pub fn load_window_settings(path: String) -> Vec<EinkWindowSetting> {
    if let Some(parent) = std::path::Path::new(&path).parent() {
        // Not create_dir_all to avoid creating .config in encrypted home, when I mess things up
        std::fs::create_dir(parent)
            .unwrap_or_else(|e| panic!("Failed to create directories for {}: {}", path, e));
    }

    let contents = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => {
            eprintln!(
                "File {} not found or unreadable. Creating default settings.",
                path
            );
            std::fs::write(&path, DEFAULT_WINDOW_SETTINGS)
                .unwrap_or_else(|e| panic!("Failed to write default settings to {}: {}", path, e));
            DEFAULT_WINDOW_SETTINGS.to_string()
        }
    };

    match ron::from_str(&contents) {
        Ok(settings) => settings,
        Err(_) => {
            eprintln!(
                "Failed to parse settings from file. Rewriting with default settings at {}",
                path
            );
            std::fs::write(&path, DEFAULT_WINDOW_SETTINGS)
                .unwrap_or_else(|e| panic!("Failed to write default settings to {}: {}", path, e));
            ron::from_str(DEFAULT_WINDOW_SETTINGS)
                .unwrap_or_else(|e| panic!("Failed to parse default settings: {}", e))
        }
    }
}

pub const PINENOTE_ENABLE_SOCKET: &str = "/tmp/ps_quill_niri.sock";
