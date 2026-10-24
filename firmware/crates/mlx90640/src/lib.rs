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

// Portions ported from MLX90640_API.c, Copyright (C) 2017 Melexis N.V.,
// licensed under the Apache License, Version 2.0. Modified: translated from C
// to Rust for pit-pyro (subpage reads and validation in this file).

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

/// Positions of the words the maths uses in a [`Subpage`] (Melexis's
/// `frameData`): auxiliary data from the sensor's RAM, then two added by the read.
pub(crate) mod word {
    /// Ta_Vbe, the voltage behind the sensor's own temperature.
    pub const TA_VBE: usize = 768;
    /// Compensation pixel, subpage 0.
    pub const CP_SP0: usize = 776;
    /// The ADC gain measurement.
    pub const GAIN: usize = 778;
    /// Ta_PTAT, the proportional-to-temperature reference.
    pub const TA_PTAT: usize = 800;
    /// Compensation pixel, subpage 1.
    pub const CP_SP1: usize = 808;
    /// The supply voltage measurement.
    pub const VDD_PIX: usize = 810;
    /// Copy of the control register, added by the read.
    pub const CONTROL: usize = 832;
    /// The subpage number, added by the read.
    pub const NUMBER: usize = 833;
}
const EEPROM_START: u16 = 0x2400;
const RAM_START: u16 = 0x0400;
const AUX_START: u16 = 0x0700;
const STATUS: u16 = 0x8000;
const CONTROL: u16 = 0x800D;
/// Written to the status register after each read: clears "new data" and
/// turns RAM overwrite off (bit 4), so the sensor can't replace a subpage while
/// it's being read. Melexis's driver leaves overwrite on (0x0030); at the
/// fastest rates a 40 ms read could then mix two subpages.
const STATUS_CLEAR: u16 = 0x0020;
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
    /// Every rate, slowest first, in the order of their register values.
    pub const ALL: [RefreshRate; 8] = [
        RefreshRate::Hz0_5,
        RefreshRate::Hz1,
        RefreshRate::Hz2,
        RefreshRate::Hz4,
        RefreshRate::Hz8,
        RefreshRate::Hz16,
        RefreshRate::Hz32,
        RefreshRate::Hz64,
    ];

    /// Subpages per second: 0.5, doubling with each step.
    pub fn hz(self) -> f32 {
        f32::from(1u16 << (self as u8)) / 2.0
    }

    /// The rate for `hz` subpages per second, if the sensor supports it.
    pub fn from_hz(hz: f32) -> Option<Self> {
        Self::ALL.into_iter().find(|r| r.hz() == hz)
    }

    fn from_bits(bits: u16) -> Self {
        Self::ALL[usize::from(bits & 0b111)]
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
        self.words[word::NUMBER]
    }

    /// The control register at the time of the read.
    pub fn control(&self) -> u16 {
        self.words[word::CONTROL]
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
    ///
    /// Reads RAM first and only then clears "new data", with overwrite off, so
    /// the sensor holds the subpage until the read is done.
    pub fn read_subpage(&mut self, out: &mut Subpage) -> Result<bool, Error<I::Error>> {
        let status = self.read_word(STATUS)?;
        if status & STATUS_DATA_READY == 0 {
            return Ok(false);
        }
        let (pixels, rest) = out.words.split_at_mut(PIXELS);
        self.read_words(RAM_START, pixels)?;
        self.read_words(AUX_START, &mut rest[..AUX_WORDS])?;
        out.words[word::CONTROL] = self.read_word(CONTROL)?;
        out.words[word::NUMBER] = status & STATUS_SUBPAGE;
        self.write_word(STATUS, STATUS_CLEAR)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use core::convert::Infallible;
    use embedded_hal::i2c::{ErrorType, Operation};
    use std::collections::HashMap;

    /// A pretend sensor: 16-bit registers, and a log of what was read and written.
    #[derive(Default)]
    struct FakeSensor {
        registers: HashMap<u16, u16>,
        log: Vec<(char, u16)>,
    }

    impl ErrorType for FakeSensor {
        type Error = Infallible;
    }

    impl I2c for FakeSensor {
        fn transaction(&mut self, _address: u8, operations: &mut [Operation<'_>]) -> Result<(), Infallible> {
            let mut at = 0;
            for op in operations {
                match op {
                    Operation::Write(bytes) => {
                        at = u16::from_be_bytes([bytes[0], bytes[1]]);
                        if let [_, _, hi, lo] = bytes[..] {
                            self.registers.insert(at, u16::from_be_bytes([hi, lo]));
                            self.log.push(('w', at));
                        }
                    }
                    Operation::Read(buf) => {
                        self.log.push(('r', at));
                        for pair in buf.chunks_exact_mut(2) {
                            pair.copy_from_slice(&self.registers.get(&at).copied().unwrap_or(0).to_be_bytes());
                            at += 1;
                        }
                    }
                }
            }
            Ok(())
        }
    }

    #[test]
    fn refresh_rates() {
        assert_eq!(RefreshRate::ALL.map(RefreshRate::hz), [0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0]);
        assert_eq!(RefreshRate::from_hz(4.0), Some(RefreshRate::Hz4));
        assert_eq!(RefreshRate::from_hz(3.0), None);
        assert_eq!(RefreshRate::from_bits(3), RefreshRate::Hz4);
    }

    #[test]
    fn waits_for_new_data() {
        let mut mlx = Mlx90640::new(FakeSensor::default(), DEFAULT_ADDRESS);
        assert_eq!(mlx.read_subpage(&mut Subpage::new()), Ok(false));
        assert!(mlx.release().log.iter().all(|&(op, _)| op == 'r'));
    }

    #[test]
    fn clears_new_data_only_after_reading_ram() {
        let mut sensor = FakeSensor::default();
        sensor.registers.insert(STATUS, STATUS_DATA_READY | 1); // subpage 1 ready
        sensor.registers.insert(RAM_START, 0x1234);
        let mut mlx = Mlx90640::new(sensor, DEFAULT_ADDRESS);
        let mut sub = Subpage::new();
        assert_eq!(mlx.read_subpage(&mut sub), Ok(true));
        assert_eq!(sub.number(), 1);
        assert_eq!(sub.words[0], 0x1234);

        let sensor = mlx.release();
        let clear = sensor.log.iter().position(|&e| e == ('w', STATUS)).unwrap();
        let last_ram_read = sensor.log.iter().rposition(|&(op, at)| op == 'r' && at < STATUS).unwrap();
        assert!(clear > last_ram_read, "status cleared before the RAM was read");
        let written = sensor.registers[&STATUS];
        assert_eq!(written & STATUS_DATA_READY, 0);
        assert_eq!(written & (1 << 4), 0, "RAM overwrite must be off");
    }
}
