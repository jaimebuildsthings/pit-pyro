//! The SSD1306 128 x 64 OLED on STEMMA QT port 1: switching it on and sending
//! it frames.

use ::screen::{Frame, WIDTH};
use esp_hal::Blocking;
use esp_hal::i2c::master::{Error, I2c};

/// The panel answers at one of these, set by a jumper on its board.
const ADDRESSES: [u8; 2] = [0x3D, 0x3C];

/// Adafruit's start-up sequence for a 128 x 64 panel on 3.3 V: internal charge
/// pump, horizontal addressing, the header at the top of the picture.
const INIT: &[u8] = &[
    0xAE, // display off
    0xD5, 0x80, // clock divider
    0xA8, 0x3F, // 64 rows
    0xD3, 0x00, // no vertical offset
    0x40, // start at row 0
    0x8D, 0x14, // charge pump on
    0x20, 0x00, // horizontal addressing
    0xA1, // columns mirrored
    0xC8, // rows scanned bottom up
    0xDA, 0x12, // row wiring of a 64-row panel
    0x81, 0xCF, // contrast
    0xD9, 0xF1, // pre-charge
    0xDB, 0x40, // VCOMH level
    0xA4, // show the frame, not all pixels lit
    0xA6, // not inverted
    0x2E, // no scrolling
    0xAF, // display on
];

/// A panel that answered and accepted its start-up sequence.
pub struct Screen {
    address: u8,
}

impl Screen {
    /// Looks for the panel and switches it on.
    pub fn find(bus: &mut I2c<'static, Blocking>) -> Option<Self> {
        let address = ADDRESSES.into_iter().find(|&a| bus.write(a, &[]).is_ok())?;
        let screen = Screen { address };
        screen.commands(bus, INIT).ok()?;
        Some(screen)
    }

    /// The panel's I2C address.
    pub fn address(&self) -> u8 {
        self.address
    }

    /// Sends a whole frame, one 128-byte page at a time.
    pub fn show(&self, bus: &mut I2c<'static, Blocking>, frame: &Frame) -> Result<(), Error> {
        self.commands(bus, &[0x21, 0, (WIDTH - 1) as u8, 0x22, 0, 7])?;
        let mut buf = [0u8; WIDTH + 1];
        buf[0] = 0x40; // data follows
        for page in frame.0.chunks(WIDTH) {
            buf[1..].copy_from_slice(page);
            bus.write(self.address, &buf)?;
        }
        Ok(())
    }

    fn commands(&self, bus: &mut I2c<'static, Blocking>, cmds: &[u8]) -> Result<(), Error> {
        let mut buf = [0u8; 32];
        buf[0] = 0x00; // commands follow
        buf[1..=cmds.len()].copy_from_slice(cmds);
        bus.write(self.address, &buf[..=cmds.len()])
    }
}
