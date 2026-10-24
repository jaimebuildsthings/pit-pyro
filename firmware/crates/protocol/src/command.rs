//! The serial console's command language.

use mlx90640::RefreshRate;

/// A parsed console command.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    /// `help`: list commands.
    Help,
    /// `status`: firmware, sensor, refresh rate, supply voltage and sensor temperature.
    Status,
    /// `scan`: list the devices on both I2C buses.
    Scan,
    /// `frame`: print one thermal image as a text map with min, max and centre.
    Frame,
    /// `frame csv`: print one thermal image as 24 rows of 32 temperatures.
    FrameCsv,
    /// `stream on|off`: print a text map for every new image.
    Stream(bool),
    /// `emissivity E`: set the surface emissivity (0.1–1.0, default 0.95).
    Emissivity(f32),
    /// `dump`: print the EEPROM and two raw subpages in hex, for test fixtures.
    Dump,
    /// `rate HZ`: subpages per second (see [`RefreshRate`]). A full image
    /// takes two subpages.
    Rate(RefreshRate),
}

/// Why a line didn't parse.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseError {
    /// Blank line.
    Empty,
    /// Unknown command word.
    Unknown,
    /// A required argument is missing; names it.
    Missing(&'static str),
    /// An argument isn't a valid number or option.
    BadArgument,
    /// More arguments than the command takes.
    TooManyArguments,
}

/// Help text for `help`, one command per line.
pub const HELP: &str = "\
status                   firmware, sensor, refresh rate, supply, sensor temperature
scan                     list devices on both I2C buses
frame                    one thermal image as a text map
frame csv                one thermal image as 24 rows of 32 temperatures (deg C)
stream on|off            a text map for every new image
emissivity E             surface emissivity, 0.1-1.0 (default 0.95)
dump                     EEPROM and two raw subpages in hex
rate HZ                  subpages per second: 0.5 up to 64, doubling (default 4)
help                     this list";

fn lower(word: &str) -> &str {
    // Commands are ASCII; matching case-insensitively avoids an allocation.
    const WORDS: [&str; 9] = ["help", "?", "status", "scan", "frame", "stream", "emissivity", "dump", "rate"];
    WORDS.into_iter().find(|w| w.eq_ignore_ascii_case(word)).unwrap_or(word)
}

fn on_off<'a>(args: &mut impl Iterator<Item = &'a str>) -> Result<bool, ParseError> {
    match args.next() {
        None => Err(ParseError::Missing("on|off")),
        Some(a) if a.eq_ignore_ascii_case("on") => Ok(true),
        Some(a) if a.eq_ignore_ascii_case("off") => Ok(false),
        Some(_) => Err(ParseError::BadArgument),
    }
}

impl Command {
    /// Parses one console line.
    pub fn parse(line: &str) -> Result<Self, ParseError> {
        let mut args = line.split_ascii_whitespace();
        let word = args.next().ok_or(ParseError::Empty)?;
        let cmd = match lower(word) {
            "help" | "?" => Command::Help,
            "status" => Command::Status,
            "scan" => Command::Scan,
            "frame" => match args.next() {
                None => Command::Frame,
                Some(a) if a.eq_ignore_ascii_case("csv") => Command::FrameCsv,
                Some(_) => return Err(ParseError::BadArgument),
            },
            "stream" => Command::Stream(on_off(&mut args)?),
            "emissivity" => {
                let e: f32 = args
                    .next()
                    .ok_or(ParseError::Missing("emissivity"))?
                    .parse()
                    .map_err(|_| ParseError::BadArgument)?;
                if !(0.1..=1.0).contains(&e) {
                    return Err(ParseError::BadArgument);
                }
                Command::Emissivity(e)
            }
            "dump" => Command::Dump,
            "rate" => {
                let hz: f32 =
                    args.next().ok_or(ParseError::Missing("hz"))?.parse().map_err(|_| ParseError::BadArgument)?;
                Command::Rate(RefreshRate::from_hz(hz).ok_or(ParseError::BadArgument)?)
            }
            _ => return Err(ParseError::Unknown),
        };
        if args.next().is_some() {
            return Err(ParseError::TooManyArguments);
        }
        Ok(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_commands() {
        assert_eq!(Command::parse("help"), Ok(Command::Help));
        assert_eq!(Command::parse("?"), Ok(Command::Help));
        assert_eq!(Command::parse("  STATUS "), Ok(Command::Status));
        assert_eq!(Command::parse("scan"), Ok(Command::Scan));
        assert_eq!(Command::parse("frame"), Ok(Command::Frame));
        assert_eq!(Command::parse("frame CSV"), Ok(Command::FrameCsv));
        assert_eq!(Command::parse("stream on"), Ok(Command::Stream(true)));
        assert_eq!(Command::parse("stream Off"), Ok(Command::Stream(false)));
        assert_eq!(Command::parse("emissivity 0.95"), Ok(Command::Emissivity(0.95)));
        assert_eq!(Command::parse("dump"), Ok(Command::Dump));
        assert_eq!(Command::parse("rate 0.5"), Ok(Command::Rate(RefreshRate::Hz0_5)));
        assert_eq!(Command::parse("rate 8"), Ok(Command::Rate(RefreshRate::Hz8)));
    }

    #[test]
    fn rejects_bad_lines() {
        assert_eq!(Command::parse(""), Err(ParseError::Empty));
        assert_eq!(Command::parse("   "), Err(ParseError::Empty));
        assert_eq!(Command::parse("tare"), Err(ParseError::Unknown));
        assert_eq!(Command::parse("frame png"), Err(ParseError::BadArgument));
        assert_eq!(Command::parse("stream"), Err(ParseError::Missing("on|off")));
        assert_eq!(Command::parse("stream maybe"), Err(ParseError::BadArgument));
        assert_eq!(Command::parse("emissivity"), Err(ParseError::Missing("emissivity")));
        assert_eq!(Command::parse("emissivity 1.5"), Err(ParseError::BadArgument));
        assert_eq!(Command::parse("emissivity x"), Err(ParseError::BadArgument));
        assert_eq!(Command::parse("status now"), Err(ParseError::TooManyArguments));
        assert_eq!(Command::parse("rate 3"), Err(ParseError::BadArgument));
        assert_eq!(Command::parse("rate"), Err(ParseError::Missing("hz")));
    }
}
