//! The main loop: reads the thermal sensor, runs console commands and prints
//! images.

use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Instant, Timer};
use esp_hal::Blocking;
use esp_hal::gpio::Output;
use esp_hal::i2c::master::I2c;
use mlx90640::{Calibration, DEFAULT_ADDRESS, EEPROM_WORDS, Mlx90640, PIXELS, RefreshRate, Subpage};
use protocol::command::HELP;
use protocol::image::{self, COLUMNS, ROWS, Stats};
use protocol::{Command, ParseError};

use crate::console::LINES;
use crate::out::say;

/// How often to ask the sensor for a new subpage.
const POLL: Duration = Duration::from_millis(25);
/// How long to wait before looking for a missing sensor again.
const RETRY: Duration = Duration::from_secs(2);
/// Subpages per second; a full image takes two.
const RATE: RefreshRate = RefreshRate::Hz4;
/// Tire rubber.
const DEFAULT_EMISSIVITY: f32 = 0.95;
/// Melexis: in open air, the reflected temperature is about the sensor's own
/// temperature minus 8 °C.
const REFLECTED_BELOW_SENSOR: f32 = 8.0;
/// Both subpages, as bits.
const BOTH: u8 = 0b11;

/// What a command is waiting for.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Waiting {
    Nothing,
    Frame,
    Csv,
    Dump,
}

/// App state.
pub struct App {
    mlx: Mlx90640<I2c<'static, Blocking>>,
    stemma1: I2c<'static, Blocking>,
    /// Keeps LDO2, and with it STEMMA QT port 2, switched on.
    _ldo2: Output<'static>,
    cal: Option<Calibration>,
    next_try: Instant,
    emissivity: f32,
    stream: bool,
    waiting: Waiting,
    /// Subpages read since the last command that waits for fresh data.
    fresh: u8,
    /// The latest raw subpage of each number.
    raw: [Subpage; 2],
    /// Temperatures, each subpage's pixels updated as it arrives.
    image: [f32; PIXELS],
    /// The last complete image, bad pixels filled in.
    shown: [f32; PIXELS],
    sensor_temp: f32,
}

impl App {
    /// Takes the sensor's bus (STEMMA QT port 2), the other STEMMA QT bus for
    /// `scan`, and the pin that powers port 2.
    pub fn new(sensor_bus: I2c<'static, Blocking>, stemma1: I2c<'static, Blocking>, ldo2: Output<'static>) -> Self {
        Self {
            mlx: Mlx90640::new(sensor_bus, DEFAULT_ADDRESS),
            stemma1,
            _ldo2: ldo2,
            cal: None,
            next_try: Instant::now(),
            emissivity: DEFAULT_EMISSIVITY,
            stream: false,
            waiting: Waiting::Nothing,
            fresh: 0,
            raw: [Subpage::new(), Subpage::new()],
            image: [0.0; PIXELS],
            shown: [0.0; PIXELS],
            sensor_temp: 0.0,
        }
    }

    /// Runs forever.
    pub async fn run(&mut self) -> ! {
        say!("\r\npit-pyro {} ready. Type `help`.", env!("CARGO_PKG_VERSION"));
        loop {
            match select(LINES.receive(), Timer::after(POLL)).await {
                Either::First(line) => self.command(line.as_str()).await,
                Either::Second(()) => self.poll().await,
            }
        }
    }

    async fn poll(&mut self) {
        if self.cal.is_none() {
            if Instant::now() >= self.next_try {
                self.connect().await;
            }
            return;
        }
        let mut sub = Subpage::new();
        match self.mlx.read_subpage(&mut sub) {
            Ok(true) => self.subpage(sub).await,
            Ok(false) => {}
            Err(mlx90640::Error::InvalidData) => say!("sensor: incomplete data, skipped"),
            Err(mlx90640::Error::I2c(_)) => {
                say!("sensor: lost on STEMMA 2; retrying");
                self.cal = None;
                self.next_try = Instant::now() + RETRY;
            }
        }
    }

    /// Reads the calibration and sets the refresh rate.
    async fn connect(&mut self) {
        self.next_try = Instant::now() + RETRY;
        let mut ee = [0u16; EEPROM_WORDS];
        if self.mlx.read_eeprom(&mut ee).is_err() {
            return;
        }
        match Calibration::from_eeprom(&ee) {
            Ok(cal) => {
                if self.mlx.set_refresh_rate(RATE).is_err() {
                    return;
                }
                say!("sensor: MLX90640 found, {} bad pixel(s)", cal.bad_pixels().len());
                self.cal = Some(cal);
                self.fresh = 0;
            }
            Err(e) => say!("sensor: unusable calibration: {e:?}"),
        }
    }

    async fn subpage(&mut self, sub: Subpage) {
        let Some(cal) = &self.cal else { return };
        let number = usize::from(sub.number() & 1);
        self.sensor_temp = cal.ambient(&sub);
        let reflected = self.sensor_temp - REFLECTED_BELOW_SENSOR;
        cal.temperatures(&sub, self.emissivity, reflected, &mut self.image);
        let chess = sub.chess_mode();
        self.raw[number] = sub;
        self.fresh |= 1 << number;
        if self.fresh != BOTH {
            return;
        }
        // A complete image: both subpages since the last request.
        self.shown = self.image;
        cal.correct_bad_pixels(&mut self.shown, chess);
        match self.waiting {
            Waiting::Frame => self.print_map().await,
            Waiting::Csv => self.print_csv().await,
            Waiting::Dump => self.print_dump().await,
            Waiting::Nothing if self.stream => self.print_map().await,
            Waiting::Nothing => {}
        }
        self.waiting = Waiting::Nothing;
        self.fresh = 0;
    }

    async fn command(&mut self, line: &str) {
        let cmd = match Command::parse(line) {
            Ok(c) => c,
            Err(ParseError::Empty) => return,
            Err(ParseError::Unknown) => return say!("unknown command; type `help`"),
            Err(ParseError::Missing(what)) => return say!("missing {what}"),
            Err(ParseError::BadArgument) => return say!("bad argument; type `help`"),
            Err(ParseError::TooManyArguments) => return say!("too many arguments"),
        };
        match cmd {
            Command::Help => {
                for line in HELP.lines() {
                    say!("{line}");
                }
            }
            Command::Status => self.status().await,
            Command::Scan => self.scan().await,
            Command::Frame | Command::FrameCsv | Command::Dump => {
                if self.cal.is_none() {
                    return say!("no sensor on STEMMA 2");
                }
                self.waiting = match cmd {
                    Command::Frame => Waiting::Frame,
                    Command::FrameCsv => Waiting::Csv,
                    _ => Waiting::Dump,
                };
                self.fresh = 0;
            }
            Command::Stream(on) => {
                self.stream = on;
                say!("stream {}", if on { "on" } else { "off" });
            }
            Command::Rate(hz) => {
                let Some(rate) = RefreshRate::from_hz(hz) else { return say!("unsupported rate") };
                if self.cal.is_none() {
                    return say!("no sensor on STEMMA 2");
                }
                match self.mlx.set_refresh_rate(rate) {
                    Ok(()) => say!("rate {hz} subpages/s ({} images/s)", hz / 2.0),
                    Err(_) => say!("sensor: couldn't set the rate"),
                }
                self.fresh = 0;
            }
            Command::Emissivity(e) => {
                self.emissivity = e;
                say!("emissivity {e:.2}");
            }
        }
    }

    async fn status(&mut self) {
        say!("pit-pyro {} on ESP32-S3 (FeatherS3[D])", env!("CARGO_PKG_VERSION"));
        match &self.cal {
            None => say!("sensor: not found on STEMMA 2 (SDA IO16, SCL IO15)"),
            Some(cal) => {
                let rate = self.mlx.refresh_rate().map(RefreshRate::hz).unwrap_or(0.0);
                let latest = &self.raw[0];
                say!("sensor: MLX90640 at 0x{DEFAULT_ADDRESS:02X} on STEMMA 2, {rate} subpages/s");
                say!("supply {:.2} V, sensor {:.1} deg C", cal.vdd(latest), self.sensor_temp);
                say!("bad pixels: {:?}", cal.bad_pixels());
            }
        }
        say!("emissivity {:.2}, stream {}", self.emissivity, if self.stream { "on" } else { "off" });
    }

    async fn scan(&mut self) {
        say!("STEMMA 1 (SDA IO8, SCL IO9):");
        scan_bus(&mut self.stemma1).await;
        say!("STEMMA 2 (SDA IO16, SCL IO15):");
        scan_bus(self.mlx.i2c_mut()).await;
    }

    async fn print_map(&self) {
        let stats = Stats::of(&self.shown);
        let (lo, hi) = image::scale(&stats);
        say!(
            "min {:.1}  max {:.1}  centre {:.1}  mean {:.1}  sensor {:.1}  (deg C)",
            stats.min,
            stats.max,
            stats.centre,
            stats.mean,
            self.sensor_temp
        );
        for row in 0..ROWS {
            let shades = image::map_row(&self.shown, row, lo, hi);
            say!("|{}|", core::str::from_utf8(&shades).unwrap_or(""));
        }
        say!("' ' {lo:.1} ... '@' {hi:.1} deg C");
    }

    async fn print_csv(&self) {
        for row in 0..ROWS {
            let mut buf = crate::out::Buf::new();
            let _ = image::csv_row(&self.shown, row, &mut buf);
            crate::out::line(buf.as_bytes()).await;
        }
    }

    async fn print_dump(&mut self) {
        let mut ee = [0u16; EEPROM_WORDS];
        if self.mlx.read_eeprom(&mut ee).is_err() {
            return say!("sensor: EEPROM read failed");
        }
        say!("eeprom");
        hex_lines(&ee).await;
        for sub in &self.raw {
            say!("subpage {}", sub.number());
            hex_lines(&sub.words).await;
        }
        say!("end");
    }
}

/// Lists the addresses that answer on a bus.
async fn scan_bus(bus: &mut I2c<'static, Blocking>) {
    let mut found = false;
    for addr in 0x08u8..0x78 {
        if bus.write(addr, &[]).is_ok() {
            let name = match addr {
                0x33 => "MLX90640 thermal camera",
                0x36 => "MAX17048 battery gauge (on the Feather)",
                0x3C | 0x3D => "SSD1306 screen",
                0x57 => "EEPROM (on some clock boards)",
                0x68 => "DS3231 clock",
                _ => "",
            };
            say!("  0x{addr:02X} {name}");
            found = true;
        }
    }
    if !found {
        say!("  nothing");
    }
}

/// Prints words as hex, 16 per line.
async fn hex_lines(words: &[u16]) {
    for chunk in words.chunks(16) {
        let mut buf = crate::out::Buf::new();
        for (i, w) in chunk.iter().enumerate() {
            let sep = if i == 0 { "" } else { " " };
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("{sep}{w:04X}"));
        }
        crate::out::line(buf.as_bytes()).await;
    }
}

// The map's width and the image's must agree.
const _: () = assert!(COLUMNS * ROWS == PIXELS);
