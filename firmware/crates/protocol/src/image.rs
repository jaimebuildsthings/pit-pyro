//! Thermal images as text: a shaded map for the console, CSV rows for tools,
//! and the summary numbers printed with them.

use core::fmt;

use mlx90640::{COLUMNS, PIXELS, ROWS};

/// Shades from coldest to hottest.
const SHADES: &[u8; 10] = b" .:-=+*#%@";

/// The map always spans at least this many degrees, so a uniform scene shows
/// as flat rather than as amplified noise.
const MIN_SPAN: f32 = 2.0;

/// Summary of one image, in °C.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    /// Coldest pixel.
    pub min: f32,
    /// Hottest pixel.
    pub max: f32,
    /// Mean of all pixels.
    pub mean: f32,
    /// Mean of the four pixels in the middle, where the aiming dot points.
    pub centre: f32,
}

impl Stats {
    /// Summarises an image.
    pub fn of(image: &[f32; PIXELS]) -> Self {
        let (mut min, mut max, mut sum) = (image[0], image[0], 0.0);
        for &t in image {
            min = min.min(t);
            max = max.max(t);
            sum += t;
        }
        let mid = |r: usize, c: usize| image[r * COLUMNS + c];
        let (r, c) = (ROWS / 2, COLUMNS / 2);
        let centre = (mid(r - 1, c - 1) + mid(r - 1, c) + mid(r, c - 1) + mid(r, c)) / 4.0;
        Stats { min, max, mean: sum / PIXELS as f32, centre }
    }
}

/// The temperature range a map's shades cover: the image's own range,
/// widened to at least [`MIN_SPAN`] around its middle.
pub fn scale(stats: &Stats) -> (f32, f32) {
    let span = stats.max - stats.min;
    if span >= MIN_SPAN {
        (stats.min, stats.max)
    } else {
        let mid = (stats.min + stats.max) / 2.0;
        (mid - MIN_SPAN / 2.0, mid + MIN_SPAN / 2.0)
    }
}

/// One row of the map: a shade per pixel, coldest `' '` to hottest `'@'`,
/// over the range `lo..=hi`.
pub fn map_row(image: &[f32; PIXELS], row: usize, lo: f32, hi: f32) -> [u8; COLUMNS] {
    let mut out = [b' '; COLUMNS];
    let last = (SHADES.len() - 1) as f32;
    for (col, shade) in out.iter_mut().enumerate() {
        let t = image[row * COLUMNS + col];
        let x = ((t - lo) / (hi - lo) * last + 0.5).clamp(0.0, last);
        *shade = SHADES[x as usize];
    }
    out
}

/// One row as comma-separated temperatures with one decimal, for `{}`.
pub fn csv_row(image: &[f32; PIXELS], row: usize) -> impl fmt::Display + '_ {
    CsvRow(&image[row * COLUMNS..(row + 1) * COLUMNS])
}

struct CsvRow<'a>(&'a [f32]);

impl fmt::Display for CsvRow<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (col, t) in self.0.iter().enumerate() {
            if col > 0 {
                f.write_str(",")?;
            }
            write!(f, "{t:.1}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient() -> [f32; PIXELS] {
        core::array::from_fn(|p| 20.0 + (p % COLUMNS) as f32)
    }

    #[test]
    fn stats_of_a_gradient() {
        let s = Stats::of(&gradient());
        assert_eq!(s.min, 20.0);
        assert_eq!(s.max, 51.0);
        assert_eq!(s.mean, 35.5);
        assert_eq!(s.centre, 35.5); // columns 15 and 16
    }

    #[test]
    fn map_runs_cold_to_hot() {
        let image = gradient();
        let s = Stats::of(&image);
        let (lo, hi) = scale(&s);
        let row = map_row(&image, 0, lo, hi);
        assert_eq!(row[0], b' ');
        assert_eq!(row[31], b'@');
        assert!(
            row.windows(2).all(|w| SHADES.iter().position(|&c| c == w[0]) <= SHADES.iter().position(|&c| c == w[1]))
        );
    }

    #[test]
    fn flat_scene_is_flat() {
        let image = [25.0; PIXELS];
        let (lo, hi) = scale(&Stats::of(&image));
        assert_eq!((lo, hi), (24.0, 26.0));
        let row = map_row(&image, 3, lo, hi);
        assert!(row.iter().all(|&c| c == row[0]));
    }

    #[test]
    fn csv_has_32_values() {
        let image = gradient();
        let s = csv_row(&image, 0).to_string();
        assert_eq!(s.split(',').count(), 32);
        assert!(s.starts_with("20.0,21.0,"));
        assert!(s.ends_with(",51.0"));
    }
}
