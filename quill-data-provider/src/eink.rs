use log::{debug, error};
use quill_data_provider_lib::{
    BitDepth, Conversion, Dithering, DriverMode, PINENOTE_ENABLE_SOCKET, Redraw, RedrawOptions,
    ThresholdLevel, run_cmd,
};
use std::str::FromStr;
use tokio::{io::AsyncWriteExt, net::UnixStream};

// enum ScreenOptions {
// busctl --user call org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 GlobalRefresh
// FullRefresh,
// ScreenMode(DriverMode),
// }

pub async fn refresh_screen() {
    run_cmd("busctl --user call org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 GlobalRefresh").await;
}

#[derive(Debug)]
pub struct EwwScreenConfig {
    pub window_settings: bool,
    pub screen_orientation: u16,
    driver_normal: bool,
    driver_fast: bool,
    dithering_bayer: bool,
    dithering_blue_noise16: bool,
    dithering_blue_noise32: bool,
    bitdepth_y1: bool,
    bitdepth_y2: bool,
    bitdepth_y4: bool,
    conv_thresholding: bool,
    thresholding_level_value: u8,
    conv_dithering: bool,
    redraw_fastdrawing: bool,
    redraw_level_value: u16,
    redraw_disablefastdrawing: bool,
}

fn parse_bool(state: &str, key: &str) -> bool {
    let res = state
        .lines()
        .find_map(|line| {
            let mut parts = line.splitn(2, ':');
            let k = parts.next()?.trim();
            let v = parts.next()?.trim();
            if k == key { Some(v == "true") } else { None }
        })
        .unwrap_or(false);

    debug!("For key {} the bool is: {}", key, res);
    res
}

fn parse_number<T: std::str::FromStr + std::fmt::Debug>(state: &str, key: &str, default: T) -> T {
    state
        .lines()
        .find_map(|line| {
            let mut parts = line.splitn(2, ':');
            let k = parts.next()?.trim();
            let v = parts.next()?.trim();
            if k == key { v.parse::<T>().ok() } else { None }
        })
        .unwrap_or_else(|| {
            error!("Key '{}' not found, using default {:?}", key, default);
            default
        })
}

impl EwwScreenConfig {
    pub async fn get_eww_screen_config(state: &str) -> Self {
        Self {
            window_settings: parse_bool(state, "per_window_settings"),
            screen_orientation: parse_number(state, "screen_orientation", 90),
            driver_normal: parse_bool(state, "driver_normal_mode"),
            driver_fast: parse_bool(state, "driver_fast_mode"),
            dithering_bayer: parse_bool(state, "dithering_bayer"),
            dithering_blue_noise16: parse_bool(state, "dithering_bluenoise16"),
            dithering_blue_noise32: parse_bool(state, "dithering_bluenoise32"),
            bitdepth_y1: parse_bool(state, "bitdepth_y1"),
            bitdepth_y2: parse_bool(state, "bitdepth_y2"),
            bitdepth_y4: parse_bool(state, "bitdepth_y4"),
            conv_thresholding: parse_bool(state, "conversion_thresholding"),
            thresholding_level_value: parse_number(state, "thresholding_level_value", 39),
            conv_dithering: parse_bool(state, "conversion_dithering"),
            redraw_fastdrawing: parse_bool(state, "redraw_fast_drawing"),
            redraw_level_value: parse_number(state, "redraw_level_value", 25),
            redraw_disablefastdrawing: parse_bool(state, "redraw_disabled"),
        }
    }

    pub async fn set_window_settings(&self) {
        match UnixStream::connect(PINENOTE_ENABLE_SOCKET).await {
            Ok(mut stream) => {
                let msg = if self.window_settings {
                    "true"
                } else {
                    "false"
                };

                if let Err(e) = stream.write_all(msg.as_bytes()).await {
                    error!("Failed to write to bridge socket: {}", e);
                }

                let _ = stream.shutdown().await;
            }
            Err(e) => {
                error!(
                    "Failed to connect to bridge socket at {}: {}",
                    PINENOTE_ENABLE_SOCKET, e
                );
            }
        }
    }
}

pub async fn eww_screen_config_to_enum(config: &EwwScreenConfig) -> DriverMode {
    fn get_dithering(config: &EwwScreenConfig) -> Dithering {
        if config.dithering_bayer {
            return Dithering::Bayer;
        }
        if config.dithering_blue_noise16 {
            return Dithering::BlueNoise16;
        }
        if config.dithering_blue_noise32 {
            return Dithering::BlueNoise32;
        }
        panic!("No dithering, what?");
    }
    let dithering = get_dithering(config);

    fn get_redraw(config: &EwwScreenConfig) -> Redraw {
        if config.redraw_fastdrawing {
            // map to 10-300
            let delay_drawing =
                ((config.redraw_level_value - 1) as f32 / 99.0 * 290.0 + 10.0).round() as u16;
            return Redraw::FastDrawing(RedrawOptions {
                delay: delay_drawing,
            });
        }
        if config.redraw_disablefastdrawing {
            return Redraw::DisableFastDrawing;
        }
        panic!("No redraw, what?");
    }
    let redraw = get_redraw(config);

    let get_conversion = |config: &EwwScreenConfig| -> Conversion {
        if config.conv_dithering {
            return Conversion::Dithering(dithering);
        }
        if config.conv_thresholding {
            return Conversion::Thresholding;
        }
        panic!("No conversion, what?");
    };
    let conversion = get_conversion(config);

    let get_bitdepth = |config: &EwwScreenConfig| -> BitDepth {
        if config.bitdepth_y1 {
            return BitDepth::Y1(
                conversion,
                ThresholdLevel::get_from_eww(config.thresholding_level_value),
            );
        }
        if config.bitdepth_y2 {
            return BitDepth::Y2(conversion, redraw);
        }
        if config.bitdepth_y4 {
            return BitDepth::Y4(redraw);
        }
        panic!("No bitdepth, what?");
    };
    let bitdepth = get_bitdepth(config);

    let get_mode = |config: &EwwScreenConfig| -> DriverMode {
        if config.driver_fast {
            return DriverMode::Fast(dithering);
        }
        if config.driver_normal {
            return DriverMode::Normal(bitdepth);
        }
        panic!("No mode, what?");
    };

    get_mode(config)
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PureBitDepth {
    Y1,
    Y2,
    Y4,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PureConversion {
    Thresholding,
    Dithering,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PureRedraw {
    FastDrawing,        // R
    DisableFastDrawing, // r
}

// Impl string functions for render hint enums
impl ToString for PureRedraw {
    fn to_string(&self) -> String {
        match self {
            PureRedraw::FastDrawing => "R".into(),
            PureRedraw::DisableFastDrawing => "r".into(),
        }
    }
}

impl FromStr for PureRedraw {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "R" => Ok(PureRedraw::FastDrawing),
            "r" => Ok(PureRedraw::DisableFastDrawing),
            _ => Err(()),
        }
    }
}

impl ToString for PureBitDepth {
    fn to_string(&self) -> String {
        match self {
            PureBitDepth::Y1 => "Y1".into(),
            PureBitDepth::Y2 => "Y2".into(),
            PureBitDepth::Y4 => "Y4".into(),
        }
    }
}

impl FromStr for PureBitDepth {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Y1" => Ok(PureBitDepth::Y1),
            "Y2" => Ok(PureBitDepth::Y2),
            "Y4" => Ok(PureBitDepth::Y4),
            _ => Err(()),
        }
    }
}

impl ToString for PureConversion {
    fn to_string(&self) -> String {
        match self {
            PureConversion::Thresholding => "T".into(),
            PureConversion::Dithering => "D".into(),
        }
    }
}

impl FromStr for PureConversion {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "T" => Ok(PureConversion::Thresholding),
            "D" => Ok(PureConversion::Dithering),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RenderHint {
    pub bit_depth: PureBitDepth,
    pub conversion: PureConversion,
    pub redraw: PureRedraw,
}

impl ToString for RenderHint {
    fn to_string(&self) -> String {
        format!(
            "{}|{}|{}",
            self.bit_depth.to_string(),
            self.conversion.to_string(),
            self.redraw.to_string()
        )
    }
}

impl FromStr for RenderHint {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        debug!("RenderHint FromStr received: {}", s);
        let mut parts = s.split('|');

        let bd = parts.next().ok_or(())?;
        let cv = parts.next().unwrap_or("T");
        let rw = parts.next().unwrap_or("r");

        let bit_depth = bd.parse()?;
        let conversion = cv.parse().unwrap_or(PureConversion::Thresholding);
        let redraw = rw.parse().unwrap_or(PureRedraw::FastDrawing);

        Ok(RenderHint {
            bit_depth,
            conversion,
            redraw,
        })
    }
}

impl RenderHint {
    pub async fn get_render_hint() -> Self {
        let line = run_cmd("busctl --user get-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DefaultHintHr").await;
        debug!("Received line hint: {}", line);
        let vec: Vec<&str> = line.trim().split(" ").collect();
        let final_str = vec[1].replace("\"", "");
        RenderHint::from_str(&final_str).unwrap()
    }

    pub async fn set(&self) {
        let line = format!(
            "busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DefaultHintHr s {}",
            self.to_string()
        );
        run_cmd(&line).await;
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct VisibleSettings {
    pub dithering: bool,
    pub bitdepth: bool,
    pub conversion: bool,
    pub thresholding_level: bool,
    pub redraw: bool,
    pub redraw_level: bool,
}

impl VisibleSettings {
    pub async fn set(&self, state: &str) {
        let mut updates = Vec::new();

        // debug!("State is: \n{}", state);
        if parse_bool(state, "dithering") != self.dithering {
            updates.push(format!("dithering={}", self.dithering));
        }
        if parse_bool(state, "bitdepth") != self.bitdepth {
            updates.push(format!("bitdepth={}", self.bitdepth));
        }
        if parse_bool(state, "conversion") != self.conversion {
            updates.push(format!("conversion={}", self.conversion));
        }
        if parse_bool(state, "redraw") != self.redraw {
            updates.push(format!("redraw={}", self.redraw));
        }
        // debug!("self.thresholding_level: {}", self.thresholding_level);
        if parse_bool(state, "thresholding_level") != self.thresholding_level {
            updates.push(format!("thresholding_level={}", self.thresholding_level));
        }
        if parse_bool(state, "redraw_level") != self.redraw_level {
            updates.push(format!("redraw_level={}", self.redraw_level));
        }

        if !updates.is_empty() {
            let cmd = format!("eww --no-daemonize update {}", updates.join(" "));
            debug!("Running eww update cmd: {}", cmd);
            run_cmd(&cmd).await;
        }
    }
}

pub async fn set_screen_settings(
    screen_settings: DriverMode,
    state: &str, // gamma_channel_tx: &mut tokio::sync::mpsc::Sender<GammaControl>,
) {
    let current_render_hint = RenderHint::get_render_hint().await;
    let mut render_hint = current_render_hint;
    let mut visible_settings = VisibleSettings::default();
    debug!("Got render hint which is: {:#?}", current_render_hint);
    screen_settings.set().await;
    match screen_settings {
        DriverMode::Normal(bit_depth) => {
            let mut maybe_conversion = None;
            let mut maybe_redraw = None;
            match bit_depth {
                BitDepth::Y1(conversion, _thresholding_level) => {
                    maybe_conversion = Some(conversion);
                    render_hint.bit_depth = PureBitDepth::Y1;
                    render_hint.redraw = PureRedraw::DisableFastDrawing // Doesn't make sense
                }
                BitDepth::Y2(conversion, redraw) => {
                    maybe_conversion = Some(conversion);
                    maybe_redraw = Some(redraw);
                    render_hint.bit_depth = PureBitDepth::Y2;
                }
                BitDepth::Y4(redraw) => {
                    maybe_redraw = Some(redraw);
                    render_hint.bit_depth = PureBitDepth::Y4;
                }
            }
            visible_settings.bitdepth = true;
            if let Some(conversion) = maybe_conversion {
                visible_settings.conversion = true;
                match conversion {
                    Conversion::Thresholding => {
                        render_hint.conversion = PureConversion::Thresholding;
                        // Only in Y1
                        match bit_depth {
                            BitDepth::Y1(_conversion, thresholding_level) => {
                                visible_settings.thresholding_level = true;
                                debug!("Setting thresholding value");
                                thresholding_level.set().await;
                                thresholding_level.set_eww_number().await;
                            }
                            _ => {}
                        }
                        /*
                        gamma_channel_tx
                            .send(GammaControl::PreviousValue)
                            .await
                            .ok();
                        */
                    }
                    Conversion::Dithering(dithering) => {
                        render_hint.conversion = PureConversion::Dithering;

                        dithering.set().await;
                        visible_settings.dithering = true;
                    }
                }
            }
            if let Some(redraw) = maybe_redraw {
                visible_settings.redraw = true;
                match redraw {
                    Redraw::FastDrawing(delay_drawing) => {
                        render_hint.redraw = PureRedraw::FastDrawing;

                        visible_settings.redraw_level = true;
                        delay_drawing.set().await;
                    }
                    Redraw::DisableFastDrawing => {
                        render_hint.redraw = PureRedraw::DisableFastDrawing
                    }
                }
            }
        }
        DriverMode::Fast(dithering) => {
            dithering.set().await;
            visible_settings.dithering = true;
        }
    }

    if !visible_settings.thresholding_level {
        debug!("Setting default treshold level");
        let thresholding_level = ThresholdLevel::default();
        thresholding_level.set().await;
        thresholding_level.set_eww_number().await; // So when it's visible again, it's the good number

        /*
        gamma_channel_tx
            .send(GammaControl::Force(DEFAULT_GAMMA))
            .await
            .ok();
         */
    }
    if render_hint != current_render_hint {
        debug!("Render hint changed! It's now: {:#?}", render_hint);
        render_hint.set().await;
    }

    visible_settings.set(state).await;
    // refresh_screen().await;

    debug!("Set screen settings finished!");
}
