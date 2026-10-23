//! Calibration parameters from the sensor's EEPROM: Melexis's
//! `MLX90640_ExtractParameters` and the `Extract*` functions it calls, in the
//! same order and with the same integer scaling.

use crate::math::{abs, pow2};
use crate::{COLUMNS, EEPROM_WORDS, PIXELS, ROWS};

/// Most deviating pixels (broken or outliers) the maths can correct.
pub(crate) const MAX_BAD: usize = 4;

/// Why the EEPROM can't be used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalibrationError {
    /// More than four broken pixels.
    TooManyBrokenPixels,
    /// More than four outlier pixels.
    TooManyOutlierPixels,
    /// More than four broken and outlier pixels together.
    TooManyBadPixels,
    /// Two bad pixels touch, so neither can be filled in from its neighbours.
    AdjacentBadPixels,
}

/// The sensor's calibration, extracted once from its EEPROM. About 4.7 KB.
#[derive(Clone, Debug)]
pub struct Calibration {
    pub(crate) k_vdd: i16,
    pub(crate) vdd25: i16,
    pub(crate) kv_ptat: f32,
    pub(crate) kt_ptat: f32,
    pub(crate) v_ptat25: u16,
    pub(crate) alpha_ptat: f32,
    pub(crate) gain_ee: i16,
    pub(crate) tgc: f32,
    pub(crate) cp_kv: f32,
    pub(crate) cp_kta: f32,
    pub(crate) resolution_ee: u8,
    /// True if the sensor was calibrated in chess mode (`calibrationModeEE`).
    pub(crate) chess_calibrated: bool,
    pub(crate) ks_ta: f32,
    pub(crate) ks_to: [f32; 5],
    pub(crate) ct: [i16; 5],
    pub(crate) alpha: [u16; PIXELS],
    pub(crate) alpha_scale: u8,
    pub(crate) offset: [i16; PIXELS],
    pub(crate) kta: [i8; PIXELS],
    pub(crate) kta_scale: u8,
    pub(crate) kv: [i8; PIXELS],
    pub(crate) kv_scale: u8,
    pub(crate) cp_alpha: [f32; 2],
    pub(crate) cp_offset: [i16; 2],
    pub(crate) il_chess_c: [f32; 3],
    /// Broken and outlier pixels, both corrected the same way.
    pub(crate) bad: [u16; MAX_BAD],
    pub(crate) bad_count: usize,
}

fn ms_byte(w: u16) -> u16 {
    w >> 8
}
fn ls_byte(w: u16) -> u16 {
    w & 0x00FF
}
fn nibble1(w: u16) -> u16 {
    w & 0x000F
}
fn nibble2(w: u16) -> u16 {
    (w & 0x00F0) >> 4
}
fn nibble3(w: u16) -> u16 {
    (w & 0x0F00) >> 8
}
fn nibble4(w: u16) -> u16 {
    (w & 0xF000) >> 12
}

/// Reads `value` (already masked and shifted down) as a signed number `bits` wide.
fn signed(value: u16, bits: u32) -> i32 {
    let v = i32::from(value);
    if v >= 1 << (bits - 1) { v - (1 << bits) } else { v }
}

/// Rounds half away from zero, as Melexis's `(temp ± 0.5)` cast to an integer.
fn round_i8(x: f32) -> i8 {
    (if x < 0.0 { x - 0.5 } else { x + 0.5 }) as i8
}

/// `2*(row is odd) + (column is odd)` for pixel `p`: picks one of four
/// row/column-parity coefficients.
fn split(p: usize) -> usize {
    2 * (p / 32 - (p / 64) * 2) + p % 2
}

impl Calibration {
    /// Extracts the calibration from an EEPROM dump.
    pub fn from_eeprom(ee: &[u16; EEPROM_WORDS]) -> Result<Self, CalibrationError> {
        let mut c = Calibration {
            k_vdd: 0,
            vdd25: 0,
            kv_ptat: 0.0,
            kt_ptat: 0.0,
            v_ptat25: 0,
            alpha_ptat: 0.0,
            gain_ee: 0,
            tgc: 0.0,
            cp_kv: 0.0,
            cp_kta: 0.0,
            resolution_ee: 0,
            chess_calibrated: false,
            ks_ta: 0.0,
            ks_to: [0.0; 5],
            ct: [0; 5],
            alpha: [0; PIXELS],
            alpha_scale: 0,
            offset: [0; PIXELS],
            kta: [0; PIXELS],
            kta_scale: 0,
            kv: [0; PIXELS],
            kv_scale: 0,
            cp_alpha: [0.0; 2],
            cp_offset: [0; 2],
            il_chess_c: [0.0; 3],
            bad: [0; MAX_BAD],
            bad_count: 0,
        };
        c.extract_vdd(ee);
        c.extract_ptat(ee);
        c.gain_ee = ee[48] as i16;
        c.tgc = f32::from(ls_byte(ee[60]) as u8 as i8) / 32.0;
        c.resolution_ee = ((ee[56] & 0x3000) >> 12) as u8;
        c.ks_ta = f32::from(ms_byte(ee[60]) as u8 as i8) / 8192.0;
        c.extract_ks_to(ee);
        c.extract_cp(ee);
        c.extract_alpha(ee);
        c.extract_offset(ee);
        c.extract_kta(ee);
        c.extract_kv(ee);
        c.extract_cilc(ee);
        c.extract_deviating_pixels(ee)?;
        Ok(c)
    }

    /// Broken and outlier pixels, as pixel numbers (row × 32 + column).
    pub fn bad_pixels(&self) -> &[u16] {
        &self.bad[..self.bad_count]
    }

    fn extract_vdd(&mut self, ee: &[u16; EEPROM_WORDS]) {
        let k_vdd = ms_byte(ee[51]) as u8 as i8;
        let vdd25 = i32::from(ls_byte(ee[51]));
        self.k_vdd = 32 * i16::from(k_vdd);
        self.vdd25 = (((vdd25 - 256) << 5) - 8192) as i16;
    }

    fn extract_ptat(&mut self, ee: &[u16; EEPROM_WORDS]) {
        self.kv_ptat = signed((ee[50] & 0xFC00) >> 10, 6) as f32 / 4096.0;
        self.kt_ptat = signed(ee[50] & 0x03FF, 10) as f32 / 8.0;
        self.v_ptat25 = ee[49];
        // Melexis divides the masked nibble without shifting it down first.
        self.alpha_ptat = f32::from(ee[16] & 0xF000) / pow2(14) + 8.0;
    }

    fn extract_ks_to(&mut self, ee: &[u16; EEPROM_WORDS]) {
        let step = (((ee[63] & 0x3000) >> 12) * 10) as i16;
        self.ct[0] = -40;
        self.ct[1] = 0;
        self.ct[2] = nibble2(ee[63]) as i16 * step;
        self.ct[3] = self.ct[2] + nibble3(ee[63]) as i16 * step;
        self.ct[4] = 400;
        let scale = pow2((nibble1(ee[63]) + 8) as u8);
        self.ks_to[0] = f32::from(ls_byte(ee[61]) as u8 as i8) / scale;
        self.ks_to[1] = f32::from(ms_byte(ee[61]) as u8 as i8) / scale;
        self.ks_to[2] = f32::from(ls_byte(ee[62]) as u8 as i8) / scale;
        self.ks_to[3] = f32::from(ms_byte(ee[62]) as u8 as i8) / scale;
        self.ks_to[4] = -0.0002;
    }

    fn extract_cp(&mut self, ee: &[u16; EEPROM_WORDS]) {
        let alpha_scale = (nibble4(ee[32]) + 27) as u8;
        let offset0 = signed(ee[58] & 0x03FF, 10);
        let offset1 = signed((ee[58] & 0xFC00) >> 10, 6) + offset0;
        let alpha0 = signed(ee[57] & 0x03FF, 10) as f32 / pow2(alpha_scale);
        let alpha1 = (1.0 + signed((ee[57] & 0xFC00) >> 10, 6) as f32 / 128.0) * alpha0;
        let kta_scale1 = (nibble2(ee[56]) + 8) as u8;
        let kv_scale = nibble3(ee[56]) as u8;
        self.cp_kta = f32::from(ls_byte(ee[59]) as u8 as i8) / pow2(kta_scale1);
        self.cp_kv = f32::from(ms_byte(ee[59]) as u8 as i8) / pow2(kv_scale);
        self.cp_alpha = [alpha0, alpha1];
        self.cp_offset = [offset0 as i16, offset1 as i16];
    }

    /// Per-row and per-column corrections: 24 row nibbles from `ee[row_at..]`
    /// and 32 column nibbles from `ee[col_at..]`, each signed.
    fn row_column(ee: &[u16; EEPROM_WORDS], row_at: usize, col_at: usize) -> ([i32; ROWS], [i32; COLUMNS]) {
        let nibbles = |w: u16| [nibble1(w), nibble2(w), nibble3(w), nibble4(w)].map(|n| signed(n, 4));
        let mut rows = [0; ROWS];
        let mut cols = [0; COLUMNS];
        for i in 0..ROWS / 4 {
            rows[i * 4..i * 4 + 4].copy_from_slice(&nibbles(ee[row_at + i]));
        }
        for i in 0..COLUMNS / 4 {
            cols[i * 4..i * 4 + 4].copy_from_slice(&nibbles(ee[col_at + i]));
        }
        (rows, cols)
    }

    fn extract_alpha(&mut self, ee: &[u16; EEPROM_WORDS]) {
        let rem_scale = nibble1(ee[32]);
        let col_scale = nibble2(ee[32]);
        let row_scale = nibble3(ee[32]);
        let alpha_scale = (nibble4(ee[32]) + 30) as u8;
        let alpha_ref = i32::from(ee[33]);
        let (acc_row, acc_col) = Self::row_column(ee, 34, 40);

        let mut alpha = [0f32; PIXELS];
        for (i, row) in acc_row.iter().enumerate() {
            for (j, col) in acc_col.iter().enumerate() {
                let p = 32 * i + j;
                let own = signed((ee[64 + p] & 0x03F0) >> 4, 6) as f32 * (1u32 << rem_scale) as f32;
                let sum = (alpha_ref + (row << row_scale) + (col << col_scale)) as f32 + own;
                let a = sum / pow2(alpha_scale) - self.tgc * (self.cp_alpha[0] + self.cp_alpha[1]) / 2.0;
                alpha[p] = (0.000001f64 / f64::from(a)) as f32;
            }
        }
        let mut temp = alpha.iter().copied().fold(alpha[0], f32::max);
        let mut scale = 0u8;
        while temp < 32767.4 {
            temp *= 2.0;
            scale += 1;
        }
        for (out, a) in self.alpha.iter_mut().zip(alpha) {
            *out = (a * pow2(scale) + 0.5) as u16;
        }
        self.alpha_scale = scale;
    }

    fn extract_offset(&mut self, ee: &[u16; EEPROM_WORDS]) {
        let rem_scale = nibble1(ee[16]);
        let col_scale = nibble2(ee[16]);
        let row_scale = nibble3(ee[16]);
        let offset_ref = i32::from(ee[17] as i16);
        let (occ_row, occ_col) = Self::row_column(ee, 18, 24);
        for (i, row) in occ_row.iter().enumerate() {
            for (j, col) in occ_col.iter().enumerate() {
                let p = 32 * i + j;
                let own = signed((ee[64 + p] & 0xFC00) >> 10, 6) * (1 << rem_scale);
                let sum = offset_ref + (row << row_scale) + (col << col_scale) + own;
                self.offset[p] = sum as i16;
            }
        }
    }

    fn extract_kta(&mut self, ee: &[u16; EEPROM_WORDS]) {
        let kta_rc = [
            ms_byte(ee[54]) as u8 as i8,
            ms_byte(ee[55]) as u8 as i8,
            ls_byte(ee[54]) as u8 as i8,
            ls_byte(ee[55]) as u8 as i8,
        ];
        let scale1 = (nibble2(ee[56]) + 8) as u8;
        let scale2 = nibble1(ee[56]);
        let mut kta = [0f32; PIXELS];
        for (p, k) in kta.iter_mut().enumerate() {
            let own = signed((ee[64 + p] & 0x000E) >> 1, 3) as f32 * (1u32 << scale2) as f32;
            *k = (f32::from(kta_rc[split(p)]) + own) / pow2(scale1);
        }
        let (scale, values) = rescale(&kta);
        self.kta = values;
        self.kta_scale = scale;
    }

    fn extract_kv(&mut self, ee: &[u16; EEPROM_WORDS]) {
        let kv_t = [
            signed(nibble4(ee[52]), 4) as f32,
            signed(nibble2(ee[52]), 4) as f32,
            signed(nibble3(ee[52]), 4) as f32,
            signed(nibble1(ee[52]), 4) as f32,
        ];
        let scale = nibble3(ee[56]) as u8;
        let mut kv = [0f32; PIXELS];
        for (p, k) in kv.iter_mut().enumerate() {
            *k = kv_t[split(p)] / pow2(scale);
        }
        let (scale, values) = rescale(&kv);
        self.kv = values;
        self.kv_scale = scale;
    }

    fn extract_cilc(&mut self, ee: &[u16; EEPROM_WORDS]) {
        self.chess_calibrated = ee[10] & 0x0800 == 0;
        self.il_chess_c = [
            signed(ee[53] & 0x003F, 6) as f32 / 16.0,
            signed((ee[53] & 0x07C0) >> 6, 5) as f32 / 2.0,
            signed((ee[53] & 0xF800) >> 11, 5) as f32 / 8.0,
        ];
    }

    fn extract_deviating_pixels(&mut self, ee: &[u16; EEPROM_WORDS]) -> Result<(), CalibrationError> {
        let mut broken = [0u16; 5];
        let mut outlier = [0u16; 5];
        let (mut nb, mut no) = (0, 0);
        let mut p = 0;
        while p < PIXELS && nb < 5 && no < 5 {
            if ee[64 + p] == 0 {
                broken[nb] = p as u16;
                nb += 1;
            } else if ee[64 + p] & 1 != 0 {
                outlier[no] = p as u16;
                no += 1;
            }
            p += 1;
        }
        if nb > MAX_BAD {
            return Err(CalibrationError::TooManyBrokenPixels);
        }
        if no > MAX_BAD {
            return Err(CalibrationError::TooManyOutlierPixels);
        }
        if nb + no > MAX_BAD {
            return Err(CalibrationError::TooManyBadPixels);
        }
        let bad = broken[..nb].iter().chain(&outlier[..no]);
        for (i, a) in bad.clone().enumerate() {
            if bad.clone().skip(i + 1).any(|b| adjacent(*a, *b)) {
                return Err(CalibrationError::AdjacentBadPixels);
            }
        }
        for (slot, p) in self.bad.iter_mut().zip(bad) {
            *slot = *p;
        }
        self.bad_count = nb + no;
        Ok(())
    }
}

/// Melexis's `CheckAdjacentPixels`: within one row and one column of each other.
fn adjacent(a: u16, b: u16) -> bool {
    let (ra, ca) = (i32::from(a >> 5), i32::from(a & 31));
    let (rb, cb) = (i32::from(b >> 5), i32::from(b & 31));
    (ra - rb).abs() < 2 && (ca - cb).abs() < 2
}

/// Scales `values` up by the largest power of two that keeps them within
/// ±63.4 and rounds them to `i8`, as Melexis stores Kta and Kv.
fn rescale(values: &[f32; PIXELS]) -> (u8, [i8; PIXELS]) {
    let mut temp = values.iter().fold(abs(values[0]), |m, &v| m.max(abs(v)));
    let mut scale = 0u8;
    while temp < 63.4 {
        temp *= 2.0;
        scale += 1;
    }
    (scale, values.map(|v| round_i8(v * pow2(scale))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_fields() {
        assert_eq!(signed(0b111111, 6), -1);
        assert_eq!(signed(31, 6), 31);
        assert_eq!(signed(32, 6), -32);
        assert_eq!(signed(0x3FF, 10), -1);
        assert_eq!(signed(7, 4), 7);
        assert_eq!(signed(8, 4), -8);
    }

    #[test]
    fn rounding_matches_c() {
        assert_eq!(round_i8(2.5), 3);
        assert_eq!(round_i8(-2.5), -3);
        assert_eq!(round_i8(2.49), 2);
        assert_eq!(round_i8(-0.2), 0);
    }

    #[test]
    fn split_picks_row_and_column_parity() {
        assert_eq!(split(0), 0); // row 0, column 0
        assert_eq!(split(1), 1); // row 0, column 1
        assert_eq!(split(32), 2); // row 1, column 0
        assert_eq!(split(33), 3); // row 1, column 1
        assert_eq!(split(64), 0); // row 2, column 0
    }

    #[test]
    fn adjacency() {
        assert!(adjacent(0, 1));
        assert!(adjacent(0, 33));
        assert!(!adjacent(0, 2));
        assert!(!adjacent(0, 64));
        assert!(!adjacent(31, 32)); // end of one row, start of the next
    }
}
