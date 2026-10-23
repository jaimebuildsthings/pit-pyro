//! Supply voltage, ambient temperature and per-pixel object temperatures:
//! Melexis's `MLX90640_GetVdd`, `MLX90640_GetTa`, `MLX90640_CalculateTo` and
//! `MLX90640_BadPixelsCorrection`.

use crate::calibration::Calibration;
use crate::math::{abs, pow2, root4};
use crate::{COLUMNS, PIXELS, Subpage};

/// Kelvin at 0 °C.
const KELVIN: f32 = 273.15;

impl Calibration {
    /// Supply voltage measured by the sensor, in volts.
    pub fn vdd(&self, sub: &Subpage) -> f32 {
        let resolution_ram = ((sub.control() & 0x0C00) >> 10) as u8;
        let correction = pow2(self.resolution_ee) / pow2(resolution_ram);
        (correction * f32::from(sub.words[810] as i16) - f32::from(self.vdd25)) / f32::from(self.k_vdd) + 3.3
    }

    /// The sensor's own temperature (Ta), in °C. Runs a few degrees above the
    /// air around it, since the chip warms itself.
    pub fn ambient(&self, sub: &Subpage) -> f32 {
        let vdd = self.vdd(sub);
        let ptat = f32::from(sub.words[800] as i16);
        let ptat_art = ptat / (ptat * self.alpha_ptat + f32::from(sub.words[768] as i16)) * pow2(18);
        let ta = ptat_art / (1.0 + self.kv_ptat * (vdd - 3.3)) - f32::from(self.v_ptat25);
        ta / self.kt_ptat + 25.0
    }

    /// Object temperatures in °C for this subpage's pixels, written into
    /// `out`; the other subpage's pixels are left as they were.
    ///
    /// `emissivity` is the surface's (tire rubber ≈ 0.95). `reflected` is the
    /// temperature of what the surface reflects; Melexis suggests the sensor's
    /// [`ambient`](Self::ambient) minus 8 °C in open air.
    pub fn temperatures(&self, sub: &Subpage, emissivity: f32, reflected: f32, out: &mut [f32; PIXELS]) {
        let words = &sub.words;
        let number = sub.number();
        let vdd = self.vdd(sub);
        let ta = self.ambient(sub);

        let pow4 = |t: f32| {
            let k = t + KELVIN;
            (k * k) * (k * k)
        };
        let ta4 = pow4(ta);
        let tr4 = pow4(reflected);
        let ta_tr = tr4 - (tr4 - ta4) / emissivity;

        let kta_scale = pow2(self.kta_scale);
        let kv_scale = pow2(self.kv_scale);
        let alpha_scale = pow2(self.alpha_scale);

        let ks_to = &self.ks_to;
        let ct = self.ct.map(f32::from);
        let alpha_corr_r = {
            let r2 = 1.0 + ks_to[1] * ct[2];
            [1.0 / (1.0 + ks_to[0] * 40.0), 1.0, r2, r2 * (1.0 + ks_to[2] * (ct[3] - ct[2]))]
        };

        let gain = f32::from(self.gain_ee) / f32::from(words[778] as i16);
        let chess = sub.chess_mode();
        let calibrated_mode = chess == self.chess_calibrated;

        let cp_drift = (1.0 + self.cp_kta * (ta - 25.0)) * (1.0 + self.cp_kv * (vdd - 3.3));
        let mut ir_cp = [f32::from(words[776] as i16) * gain, f32::from(words[808] as i16) * gain];
        ir_cp[0] -= f32::from(self.cp_offset[0]) * cp_drift;
        if calibrated_mode {
            ir_cp[1] -= f32::from(self.cp_offset[1]) * cp_drift;
        } else {
            ir_cp[1] -= (f32::from(self.cp_offset[1]) + self.il_chess_c[0]) * cp_drift;
        }
        let ir_cp = ir_cp[usize::from(number & 1)];

        for (p, out) in out.iter_mut().enumerate() {
            let il_pattern = (p / 32 - (p / 64) * 2) as i32;
            let chess_pattern = il_pattern ^ (p - (p / 2) * 2) as i32;
            let pattern = if chess { chess_pattern } else { il_pattern };
            if pattern != i32::from(number) {
                continue;
            }

            let mut ir = f32::from(words[p] as i16) * gain;
            let kta = f32::from(self.kta[p]) / kta_scale;
            let kv = f32::from(self.kv[p]) / kv_scale;
            ir -= f32::from(self.offset[p]) * (1.0 + kta * (ta - 25.0)) * (1.0 + kv * (vdd - 3.3));
            if !calibrated_mode {
                // Kept in Melexis's form so it reads the same as the C.
                #[allow(clippy::manual_div_ceil)]
                let conversion_pattern =
                    ((p + 2) / 4) as i32 - ((p + 3) / 4) as i32 + ((p + 1) / 4) as i32 - (p / 4) as i32;
                let conversion_pattern = conversion_pattern * (1 - 2 * il_pattern);
                ir += self.il_chess_c[2] * (2 * il_pattern - 1) as f32 - self.il_chess_c[1] * conversion_pattern as f32;
            }
            ir -= self.tgc * ir_cp;
            ir /= emissivity;

            let alpha = (0.000001 * f64::from(alpha_scale) / f64::from(self.alpha[p])) as f32;
            let alpha = alpha * (1.0 + self.ks_ta * (ta - 25.0));

            let sx = root4(alpha * alpha * alpha * (ir + alpha * ta_tr)) * ks_to[1];
            let to = root4(ir / (alpha * (1.0 - ks_to[1] * KELVIN) + sx) + ta_tr) - KELVIN;

            let range = if to < ct[1] {
                0
            } else if to < ct[2] {
                1
            } else if to < ct[3] {
                2
            } else {
                3
            };
            let denominator = alpha * alpha_corr_r[range] * (1.0 + ks_to[range] * (to - ct[range]));
            *out = root4(ir / denominator + ta_tr) - KELVIN;
        }
    }

    /// Replaces the factory-flagged bad pixels in a complete image with values
    /// from their neighbours, as Melexis's `MLX90640_BadPixelsCorrection`.
    /// `chess` is the measurement mode ([`Subpage::chess_mode`]).
    pub fn correct_bad_pixels(&self, to: &mut [f32; PIXELS], chess: bool) {
        for &bad in self.bad_pixels() {
            let p = usize::from(bad);
            let line = p / COLUMNS;
            let column = p % COLUMNS;
            to[p] = if chess {
                match (line, column) {
                    (0, 0) => to[33],
                    (0, 31) => to[62],
                    (0, _) => (to[p + 31] + to[p + 33]) / 2.0,
                    (23, 0) => to[705],
                    (23, 31) => to[734],
                    (23, _) => (to[p - 33] + to[p - 31]) / 2.0,
                    (_, 0) => (to[p - 31] + to[p + 33]) / 2.0,
                    (_, 31) => (to[p - 33] + to[p + 31]) / 2.0,
                    _ => median4([to[p - 33], to[p - 31], to[p + 31], to[p + 33]]),
                }
            } else {
                match column {
                    0 => to[p + 1],
                    1 | 30 => (to[p - 1] + to[p + 1]) / 2.0,
                    31 => to[p - 1],
                    _ if !self.is_bad(p - 2) && !self.is_bad(p + 2) => {
                        let right = to[p + 1] - to[p + 2];
                        let left = to[p - 1] - to[p - 2];
                        if abs(right) > abs(left) { to[p - 1] + left } else { to[p + 1] + right }
                    }
                    _ => (to[p - 1] + to[p + 1]) / 2.0,
                }
            };
        }
    }

    fn is_bad(&self, p: usize) -> bool {
        self.bad_pixels().iter().any(|&b| usize::from(b) == p)
    }
}

/// Median of four values: the mean of the middle two.
fn median4(mut v: [f32; 4]) -> f32 {
    for i in 0..3 {
        for j in i + 1..4 {
            if v[j] < v[i] {
                v.swap(i, j);
            }
        }
    }
    (v[1] + v[2]) / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_of_four() {
        assert_eq!(median4([4.0, 1.0, 3.0, 2.0]), 2.5);
        assert_eq!(median4([1.0, 1.0, 9.0, 9.0]), 5.0);
    }
}
