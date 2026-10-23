//! Driver for the Melexis MLX90640 32 × 24 thermal array, and the maths that
//! turns its raw readings into temperatures.
//!
//! The sensor measures half its pixels at a time, in two *subpages* that
//! alternate in a chessboard pattern. [`Mlx90640::read_subpage`] reads one;
//! [`Calibration::temperatures`] updates that subpage's pixels in a 768-pixel
//! image, so two calls in a row give a complete image.
//!
//! [`Calibration`] is built once from the sensor's EEPROM. Its maths is a port
//! of Melexis's reference driver (`MLX90640_API.c` in
//! <https://github.com/melexis/mlx90640-library>), Copyright (C) 2017
//! Melexis N.V., licensed under the Apache License, Version 2.0, and keeps its
//! structure so the two can be compared line by line.

#![cfg_attr(not(test), no_std)]

mod calibration;
mod math;
mod temperature;

pub use calibration::{Calibration, CalibrationError};

use embedded_hal::i2c::I2c;

/// Columns in the image.
pub const COLUMNS: usize = 32;
/// Rows in the image.
pub const ROWS: usize = 24;
/// Pixels in the image.
pub const PIXELS: usize = COLUMNS * ROWS;
/// 16-bit words of calibration data in the EEPROM.
pub const EEPROM_WORDS: usize = 832;
/// Words in a [`Subpage`]: 768 pixels, 64 auxiliary words, the control register
/// and the subpage number.
pub const SUBPAGE_WORDS: usize = PIXELS + AUX_WORDS + 2;

/// The sensor's factory I2C address.
pub const DEFAULT_ADDRESS: u8 = 0x33;

const AUX_WORDS: usize = 64;
const EEPROM_START: u16 = 0x2400;
const RAM_START: u16 = 0x0400;
const AUX_START: u16 = 0x0700;
const STATUS: u16 = 0x8000;
const CONTROL: u16 = 0x800D;
/// Written to the status register after each read: clears "new data" and keeps
/// RAM overwrite enabled, as Melexis's driver does.
const STATUS_CLEAR: u16 = 0x0030;
const STATUS_DATA_READY: u16 = 1 << 3;
const STATUS_SUBPAGE: u16 = 1;
const CONTROL_REFRESH_SHIFT: u16 = 7;
const CONTROL_REFRESH_MASK: u16 = 0b111 << CONTROL_REFRESH_SHIFT;
/// Largest number of words read in one I2C transfer.
const CHUNK_WORDS: usize = 64;
/// What the sensor returns for a pixel or auxiliary word it hasn't measured.
const INVALID: u16 = 0x7FFF;

/// How often the sensor measures a subpage. A full image takes two.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RefreshRate {
    /// 0.5 Hz.
    Hz0_5 = 0,
    /// 1 Hz.
    Hz1 = 1,
    /// 2 Hz, the factory default.
    Hz2 = 2,
    /// 4 Hz.
    Hz4 = 3,
    /// 8 Hz.
    Hz8 = 4,
    /// 16 Hz.
    Hz16 = 5,
    /// 32 Hz.
    Hz32 = 6,
    /// 64 Hz.
    Hz64 = 7,
}

impl RefreshRate {
    /// Subpages per second.
    pub const fn hz(self) -> f32 {
        match self {
            RefreshRate::Hz0_5 => 0.5,
            RefreshRate::Hz1 => 1.0,
            RefreshRate::Hz2 => 2.0,
            RefreshRate::Hz4 => 4.0,
            RefreshRate::Hz8 => 8.0,
            RefreshRate::Hz16 => 16.0,
            RefreshRate::Hz32 => 32.0,
            RefreshRate::Hz64 => 64.0,
        }
    }

    /// The rate for `hz` subpages per second, if the sensor supports it.
    pub fn from_hz(hz: f32) -> Option<Self> {
        [
            RefreshRate::Hz0_5,
            RefreshRate::Hz1,
            RefreshRate::Hz2,
            RefreshRate::Hz4,
            RefreshRate::Hz8,
            RefreshRate::Hz16,
            RefreshRate::Hz32,
            RefreshRate::Hz64,
        ]
        .into_iter()
        .find(|r| r.hz() == hz)
    }

    const fn from_bits(bits: u16) -> Self {
        match bits & 0b111 {
            0 => RefreshRate::Hz0_5,
            1 => RefreshRate::Hz1,
            2 => RefreshRate::Hz2,
            3 => RefreshRate::Hz4,
            4 => RefreshRate::Hz8,
            5 => RefreshRate::Hz16,
            6 => RefreshRate::Hz32,
            _ => RefreshRate::Hz64,
        }
    }
}

/// One subpage as read from the sensor, laid out like Melexis's `frameData`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Subpage {
    /// Pixels (0..768), auxiliary data (768..832), the control register (832)
    /// and the subpage number (833).
    pub words: [u16; SUBPAGE_WORDS],
}

impl Subpage {
    /// An all-zero subpage, for buffers.
    pub const fn new() -> Self {
        Self { words: [0; SUBPAGE_WORDS] }
    }

    /// Which subpage this is: 0 or 1.
    pub fn number(&self) -> u16 {
        self.words[833]
    }

    /// The control register at the time of the read.
    pub fn control(&self) -> u16 {
        self.words[832]
    }

    /// True if the sensor measured in chess mode (the factory default) rather
    /// than interleaved mode.
    pub fn chess_mode(&self) -> bool {
        self.control() & (1 << 12) != 0
    }

    /// Checks for words the sensor flags as not measured, as Melexis's
    /// `ValidateFrameData` and `ValidateAuxData` do.
    fn is_valid(&self) -> bool {
        let pixels_ok = (0..ROWS).all(|row| self.words[row * COLUMNS] != INVALID || (row % 2) as u16 != self.number());
        let aux = &self.words[PIXELS..PIXELS + AUX_WORDS];
        let checked = [0..1, 8..19, 20..23, 24..33, 40..51, 52..55, 56..64];
        let aux_ok = checked.into_iter().flatten().all(|i| aux[i] != INVALID);
        pixels_ok && aux_ok
    }
}

impl Default for Subpage {
    fn default() -> Self {
        Self::new()
    }
}

/// Driver errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error<E> {
    /// The I2C transfer failed (for example, no sensor at the address).
    I2c(E),
    /// The sensor returned words it flags as not measured.
    InvalidData,
}

impl<E> From<E> for Error<E> {
    fn from(e: E) -> Self {
        Error::I2c(e)
    }
}

/// An MLX90640 on an I2C bus.
pub struct Mlx90640<I> {
    i2c: I,
    address: u8,
}

impl<I: I2c> Mlx90640<I> {
    /// Takes the bus. Doesn't talk to the sensor.
    pub fn new(i2c: I, address: u8) -> Self {
        Self { i2c, address }
    }

    /// Gives the bus back.
    pub fn release(self) -> I {
        self.i2c
    }

    /// Borrows the bus, to talk to other devices on it.
    pub fn i2c_mut(&mut self) -> &mut I {
        &mut self.i2c
    }

    /// Reads the calibration data, to build a [`Calibration`] from.
    pub fn read_eeprom(&mut self, out: &mut [u16; EEPROM_WORDS]) -> Result<(), Error<I::Error>> {
        self.read_words(EEPROM_START, out)
    }

    /// The current refresh rate.
    pub fn refresh_rate(&mut self) -> Result<RefreshRate, Error<I::Error>> {
        let control = self.read_word(CONTROL)?;
        Ok(RefreshRate::from_bits(control >> CONTROL_REFRESH_SHIFT))
    }

    /// Sets the refresh rate. Lasts until the sensor loses power.
    pub fn set_refresh_rate(&mut self, rate: RefreshRate) -> Result<(), Error<I::Error>> {
        let control = self.read_word(CONTROL)?;
        let value = (control & !CONTROL_REFRESH_MASK) | ((rate as u16) << CONTROL_REFRESH_SHIFT);
        self.write_word(CONTROL, value)
    }

    /// Reads a subpage if a new one is ready; returns `Ok(false)` if not.
    pub fn read_subpage(&mut self, out: &mut Subpage) -> Result<bool, Error<I::Error>> {
        let status = self.read_word(STATUS)?;
        if status & STATUS_DATA_READY == 0 {
            return Ok(false);
        }
        self.write_word(STATUS, STATUS_CLEAR)?;
        let (pixels, rest) = out.words.split_at_mut(PIXELS);
        self.read_words(RAM_START, pixels)?;
        self.read_words(AUX_START, &mut rest[..AUX_WORDS])?;
        rest[AUX_WORDS] = self.read_word(CONTROL)?;
        rest[AUX_WORDS + 1] = status & STATUS_SUBPAGE;
        if out.is_valid() { Ok(true) } else { Err(Error::InvalidData) }
    }

    fn read_word(&mut self, register: u16) -> Result<u16, Error<I::Error>> {
        let mut word = [0];
        self.read_words(register, &mut word)?;
        Ok(word[0])
    }

    fn read_words(&mut self, start: u16, out: &mut [u16]) -> Result<(), Error<I::Error>> {
        let mut bytes = [0u8; CHUNK_WORDS * 2];
        for (n, chunk) in out.chunks_mut(CHUNK_WORDS).enumerate() {
            let register = start + (n * CHUNK_WORDS) as u16;
            let buf = &mut bytes[..chunk.len() * 2];
            self.i2c.write_read(self.address, &register.to_be_bytes(), buf)?;
            for (word, pair) in chunk.iter_mut().zip(buf.chunks_exact(2)) {
                *word = u16::from_be_bytes([pair[0], pair[1]]);
            }
        }
        Ok(())
    }

    fn write_word(&mut self, register: u16, value: u16) -> Result<(), Error<I::Error>> {
        let [r0, r1] = register.to_be_bytes();
        let [v0, v1] = value.to_be_bytes();
        self.i2c.write(self.address, &[r0, r1, v0, v1])?;
        Ok(())
    }
}
