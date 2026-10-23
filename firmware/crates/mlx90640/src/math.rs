//! Float helpers that `core` lacks on `no_std` targets.

/// Absolute value.
pub(crate) fn abs(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// 2 to the power `n`, exact for the small exponents the calibration uses.
pub(crate) fn pow2(n: u8) -> f32 {
    (1u64 << n) as f32
}

/// Square root by Newton's method, in f64 like the C library's `sqrt`.
/// Returns NaN for negative or NaN input.
pub(crate) fn sqrt(x: f64) -> f64 {
    if x.is_nan() || x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 || x.is_infinite() {
        return x;
    }
    // Halving the exponent bits gives a guess within a factor of ~2,
    // so a handful of iterations reaches full f64 precision.
    let mut y = f64::from_bits((x.to_bits() >> 1) + (1023u64 << 51));
    for _ in 0..8 {
        y = 0.5 * (y + x / y);
    }
    y
}

/// Fourth root, as Melexis's `sqrt(sqrt(x))`.
pub(crate) fn root4(x: f32) -> f32 {
    sqrt(sqrt(f64::from(x))) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqrt_matches_std() {
        for &x in &[1e-12, 0.25, 1.0, 2.0, 50.0, 12345.678, 8.4e6, 1e18] {
            let rel = (sqrt(x) - x.sqrt()).abs() / x.sqrt();
            assert!(rel < 1e-15, "sqrt({x}) off by {rel}");
        }
    }

    #[test]
    fn sqrt_edge_cases() {
        assert_eq!(sqrt(0.0), 0.0);
        assert!(sqrt(-1.0).is_nan());
        assert!(sqrt(f64::NAN).is_nan());
        assert_eq!(sqrt(f64::INFINITY), f64::INFINITY);
    }

    #[test]
    fn pow2_is_exact() {
        assert_eq!(pow2(0), 1.0);
        assert_eq!(pow2(14), 16384.0);
        assert_eq!(pow2(45), 35_184_372_088_832.0);
    }
}
