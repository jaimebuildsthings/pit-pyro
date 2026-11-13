//! What the 128 x 64 OLED shows: the thermal image as a dithered heat map,
//! the hottest, mean and coldest temperatures beside it, and a 1-pixel frame
//! around the whole panel, to measure where the lit area sits in the case's
//! window.
//!
//! Frames are in the SSD1306's page layout, ready to send: 8 pages of 128
//! bytes, each byte a column of 8 pixels with the top one in bit 0.

#![cfg_attr(not(test), no_std)]

use core::fmt::{self, Write};

use mlx90640::{COLUMNS, PIXELS, ROWS};
use protocol::image::{self, Stats};

/// Panel width, pixels.
pub const WIDTH: usize = 128;
/// Panel height, pixels.
pub const HEIGHT: usize = 64;

/// Each sensor pixel is drawn as a square this many panel pixels wide.
const SCALE: usize = 2;
/// Top left of the heat map: 64 x 48, centred in the height.
const MAP_X: usize = 3;
const MAP_Y: usize = (HEIGHT - ROWS * SCALE) / 2;
/// The readings, right of the map: one line each for the hottest, the mean and
/// the coldest pixel.
const TEXT_X: usize = MAP_X + COLUMNS * SCALE + 4;
const TEXT_Y: [usize; 3] = [8, 27, 46];
/// Glyphs are 3 x 5, drawn at twice that size, with a gap after each.
const GLYPH: (usize, usize) = (3, 5);
const TEXT_SCALE: usize = 2;
const ADVANCE: usize = (GLYPH.0 + 1) * TEXT_SCALE;
/// Right edge of the widest line: a label, a space and a five-character reading.
const TEXT_RIGHT: usize = TEXT_X + 7 * ADVANCE - TEXT_SCALE;
// The layout stays inside the border, and the text clear of the map.
const _: () = assert!(TEXT_X > MAP_X + COLUMNS * SCALE + 1 && TEXT_RIGHT < WIDTH - 1);
const _: () = assert!(TEXT_Y[2] + GLYPH.1 * TEXT_SCALE < HEIGHT - 1 && MAP_X > 1 && MAP_Y > 1);

/// 4 x 4 ordered-dither thresholds, in sixteenths.
const BAYER: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

/// One frame for the panel, in its page layout.
#[derive(Clone, PartialEq, Eq)]
pub struct Frame(pub [u8; WIDTH * HEIGHT / 8]);

impl Frame {
    /// A dark panel with the 1-pixel frame around its edge.
    pub fn border() -> Self {
        let mut f = Frame([0; WIDTH * HEIGHT / 8]);
        for x in 0..WIDTH {
            f.set(x, 0);
            f.set(x, HEIGHT - 1);
        }
        for y in 0..HEIGHT {
            f.set(0, y);
            f.set(WIDTH - 1, y);
        }
        f
    }

    /// The thermal image and its readings, inside the border.
    pub fn thermal(image: &[f32; PIXELS]) -> Self {
        let mut f = Self::border();
        let stats = Stats::of(image);
        let (lo, hi) = image::scale(&stats);
        for y in 0..ROWS * SCALE {
            for x in 0..COLUMNS * SCALE {
                let t = image[(y / SCALE) * COLUMNS + x / SCALE];
                let level = ((t - lo) / (hi - lo) * 16.0).clamp(0.0, 16.0);
                let (px, py) = (MAP_X + x, MAP_Y + y);
                if level > f32::from(BAYER[py % 4][px % 4]) + 0.5 {
                    f.set(px, py);
                }
            }
        }
        for (y, (label, t)) in TEXT_Y.into_iter().zip([('H', stats.max), ('A', stats.mean), ('L', stats.min)]) {
            let mut line = Line::default();
            let _ = write!(line, "{label} {}", Reading(t));
            f.text(TEXT_X, y, line.as_bytes());
        }
        f
    }

    /// Whether a pixel is lit.
    pub fn get(&self, x: usize, y: usize) -> bool {
        self.0[(y / 8) * WIDTH + x] & (1 << (y % 8)) != 0
    }

    fn set(&mut self, x: usize, y: usize) {
        if x < WIDTH && y < HEIGHT {
            self.0[(y / 8) * WIDTH + x] |= 1 << (y % 8);
        }
    }

    fn text(&mut self, x: usize, y: usize, s: &[u8]) {
        for (i, &c) in s.iter().enumerate() {
            let rows = glyph(c);
            for (gy, bits) in rows.iter().enumerate() {
                for gx in 0..GLYPH.0 {
                    if bits & (0b100 >> gx) != 0 {
                        for dy in 0..TEXT_SCALE {
                            for dx in 0..TEXT_SCALE {
                                self.set(x + i * ADVANCE + gx * TEXT_SCALE + dx, y + gy * TEXT_SCALE + dy);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A temperature with one decimal, or dashes when it can't be shown.
struct Reading(f32);

impl fmt::Display for Reading {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_finite() && self.0.abs() < 999.95 { write!(f, "{:.1}", self.0) } else { f.write_str("---") }
    }
}

/// One line of text: a label, a space and a reading fit in 8 bytes.
#[derive(Default)]
struct Line {
    buf: [u8; 8],
    len: usize,
}

impl Line {
    fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl Write for Line {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.len + s.len();
        if end > self.buf.len() {
            return Err(fmt::Error);
        }
        self.buf[self.len..end].copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

/// 3 x 5 glyphs, one row per entry, the leftmost pixel in bit 2. Anything not
/// listed is blank.
fn glyph(c: u8) -> [u8; 5] {
    match c {
        b'0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        b'1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        b'2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        b'3' => [0b111, 0b001, 0b011, 0b001, 0b111],
        b'4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        b'5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        b'6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        b'7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        b'8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        b'9' => [0b111, 0b101, 0b111, 0b001, 0b111],
        b'.' => [0b000, 0b000, 0b000, 0b000, 0b010],
        b'-' => [0b000, 0b000, 0b111, 0b000, 0b000],
        b'H' => [0b101, 0b101, 0b111, 0b101, 0b101],
        b'A' => [0b010, 0b101, 0b111, 0b101, 0b101],
        b'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
        _ => [0; 5],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit_in(f: &Frame, x0: usize, y0: usize, w: usize, h: usize) -> usize {
        (y0..y0 + h).flat_map(|y| (x0..x0 + w).map(move |x| (x, y))).filter(|&(x, y)| f.get(x, y)).count()
    }

    #[test]
    fn border_is_the_panel_edge() {
        let f = Frame::border();
        for x in 0..WIDTH {
            assert!(f.get(x, 0) && f.get(x, HEIGHT - 1));
        }
        for y in 0..HEIGHT {
            assert!(f.get(0, y) && f.get(WIDTH - 1, y));
        }
        assert_eq!(lit_in(&f, 1, 1, WIDTH - 2, HEIGHT - 2), 0);
    }

    #[test]
    fn hot_pixel_is_solid_and_cold_is_dark() {
        let mut image = [20.0; PIXELS];
        image[0] = 40.0;
        let f = Frame::thermal(&image);
        assert_eq!(lit_in(&f, MAP_X, MAP_Y, SCALE, SCALE), SCALE * SCALE);
        assert_eq!(lit_in(&f, MAP_X + SCALE, MAP_Y + SCALE, 8, 8), 0);
    }

    #[test]
    fn flat_scene_is_half_grey() {
        let f = Frame::thermal(&[25.0; PIXELS]);
        let area = COLUMNS * SCALE * ROWS * SCALE;
        assert_eq!(lit_in(&f, MAP_X, MAP_Y, COLUMNS * SCALE, ROWS * SCALE), area / 2);
    }

    #[test]
    fn readings_fit_beside_the_map() {
        let right = TEXT_RIGHT;
        let mut image = [-12.3; PIXELS];
        image[5] = 123.4;
        let f = Frame::thermal(&image);
        assert!(lit_in(&f, TEXT_X, TEXT_Y[0], right - TEXT_X, GLYPH.1 * TEXT_SCALE) > 0);
        // Nothing between the text and the border.
        assert_eq!(lit_in(&f, right + 1, 1, WIDTH - 2 - right, HEIGHT - 2), 0);
    }

    #[test]
    fn unreadable_values_show_dashes() {
        let mut line = Line::default();
        write!(line, "H {}", Reading(f32::NAN)).unwrap();
        assert_eq!(line.as_bytes(), b"H ---");
        let mut line = Line::default();
        write!(line, "L {}", Reading(-0.04)).unwrap();
        assert_eq!(line.as_bytes(), b"L -0.0");
    }
}
