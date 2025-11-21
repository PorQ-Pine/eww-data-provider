use log::debug;
use std::str::FromStr;
use tokio::process::Command;

// The mess of connected enums is so we know what affects when, so:
// - We can set only what's needed
// - We show only what can be changed

// Only matters when:
// DriverMode is Fast
// Normal, Y2 and Y1
// busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DitherMode y 0
#[derive(Copy, Clone, Debug)]
pub enum Dithering {
    Bayer,       // 0
    BlueNoise16, // 1
    BlueNoise32, // 2
}

// busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DriverMode y 0
#[derive(Copy, Clone, Debug)]
pub enum DriverMode {
    Normal(BitDepth), //0
    Fast(Dithering),  // 1
                      // Doesn't work for me
                      // Zero, // 8
}

// RenderHints
// Only matters in Normal mode
// busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DefaultHintHr s "Y1|T|R"
#[derive(Copy, Clone, Debug)]
pub enum BitDepth {
    Y1(Conversion),
    Y2(Conversion, Redraw),
    Y4(Redraw),
}

#[derive(Copy, Clone, Debug)]
pub enum Conversion {
    Tresholding,          // T
    Dithering(Dithering), // D
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Redraw {
    FastDrawing,        // R
    DisableFastDrawing, // r
}

// enum ScreenOptions {
// busctl --user call org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 GlobalRefresh
// FullRefresh,
// ScreenMode(DriverMode),
// }

async fn run_cmd(line: &str) -> String {
    let parts: Vec<&str> = line.split_whitespace().collect();
    debug!("Running run_cmd as: {} {:?}", parts[0], &parts[1..]);
    let out = Command::new(parts[0])
        .args(&parts[1..])
        .output()
        .await
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

pub async fn refresh_screen() {
    run_cmd("busctl --user call org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 GlobalRefresh").await;
}

#[derive(Debug)]
pub struct EwwScreenConfig {
    dithering_bayer: bool,
    dithering_blue_noise16: bool,
    dithering_blue_noise32: bool,
    driver_normal: bool,
    driver_fast: bool,
    bitdepth_y1: bool,
    bitdepth_y2: bool,
    bitdepth_y4: bool,
    conv_tresholding: bool,
    conv_dithering: bool,
    redraw_fastdrawing: bool,
    redraw_disablefastdrawing: bool,
}

fn parse_bool(state: &str, key: &str) -> bool {
    state
        .lines()
        .find_map(|line| {
            let mut parts = line.splitn(2, ':');
            let k = parts.next()?.trim();
            let v = parts.next()?.trim();
            if k == key { Some(v == "true") } else { None }
        })
        .unwrap_or(false)
}

pub async fn get_eww_screen_config() -> EwwScreenConfig {
    let state = &run_cmd("eww --no-daemonize state").await;
    EwwScreenConfig {
        dithering_bayer: parse_bool(state, "dithering_bayer"),
        dithering_blue_noise16: parse_bool(state, "dithering_bluenoise16"),
        dithering_blue_noise32: parse_bool(state, "dithering_bluenoise32"),
        driver_normal: parse_bool(state, "driver_normal_mode"),
        driver_fast: parse_bool(state, "driver_fast_mode"),
        bitdepth_y1: parse_bool(state, "bitdepth_y1"),
        bitdepth_y2: parse_bool(state, "bitdepth_y2"),
        bitdepth_y4: parse_bool(state, "bitdepth_y4"),
        conv_tresholding: parse_bool(state, "conversion_tresholding"),
        conv_dithering: parse_bool(state, "conversion_dithering"),
        redraw_fastdrawing: parse_bool(state, "redraw_fast_drawing"),
        redraw_disablefastdrawing: parse_bool(state, "redraw_disabled"),
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
            return Redraw::FastDrawing;
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
        if config.conv_tresholding {
            return Conversion::Tresholding;
        }
        panic!("No conversion, what?");
    };
    let conversion = get_conversion(config);

    let get_bitdepth = |config: &EwwScreenConfig| -> BitDepth {
        if config.bitdepth_y1 {
            return BitDepth::Y1(conversion);
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
    Tresholding,
    Dithering,
}

// Impl string functions for render hint enums

impl ToString for Redraw {
    fn to_string(&self) -> String {
        match self {
            Redraw::FastDrawing => "R".into(),
            Redraw::DisableFastDrawing => "r".into(),
        }
    }
}

impl FromStr for Redraw {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "R" => Ok(Redraw::FastDrawing),
            "r" => Ok(Redraw::DisableFastDrawing),
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
            PureConversion::Tresholding => "T".into(),
            PureConversion::Dithering => "D".into(),
        }
    }
}

impl FromStr for PureConversion {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "T" => Ok(PureConversion::Tresholding),
            "D" => Ok(PureConversion::Dithering),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RenderHint {
    pub bit_depth: PureBitDepth,
    pub conversion: PureConversion,
    pub redraw: Redraw,
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
        let conversion = cv.parse().unwrap_or(PureConversion::Tresholding);
        let redraw = rw.parse().unwrap_or(Redraw::DisableFastDrawing);

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
            "busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DefaultHintHr y {}",
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
    pub redraw: bool,
}

impl VisibleSettings {
    pub async fn set(&self) {
        let state = &run_cmd("eww --no-daemonize state").await;

        let mut updates = Vec::new();

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

        if !updates.is_empty() {
            let cmd = format!("eww --no-daemonize update {}", updates.join(" "));
            run_cmd(&cmd).await;
        }
    }
}

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

pub async fn set_screen_settings(screen_settings: DriverMode) {
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
                BitDepth::Y1(conversion) => {
                    maybe_conversion = Some(conversion);
                    render_hint.bit_depth = PureBitDepth::Y1;
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
                    Conversion::Tresholding => render_hint.conversion = PureConversion::Tresholding,
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
                    Redraw::FastDrawing => render_hint.redraw = Redraw::FastDrawing,
                    Redraw::DisableFastDrawing => render_hint.redraw = Redraw::DisableFastDrawing,
                }
            }
        }
        DriverMode::Fast(dithering) => {
            dithering.set().await;
            visible_settings.dithering = true;
        }
    }
    if render_hint != current_render_hint {
        debug!("Render hint changed! It's now: {:#?}", render_hint);
        render_hint.set().await;
    }

    visible_settings.set().await;
    refresh_screen().await;
}
