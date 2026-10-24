//! Checks the port against Melexis's own C driver.
//!
//! `data/desk-dump.txt` is the firmware's `dump` output from the desk-test
//! sensor: its EEPROM and two consecutive subpages. `data/desk-melexis.csv` is
//! what Melexis's `MLX90640_API.c`, compiled on the host, computes from that
//! dump with `MLX90640_ExtractParameters` and `MLX90640_CalculateTo` (both
//! subpages, emissivity 0.95, reflected temperature = Ta − 8 °C).

use mlx90640::{Calibration, EEPROM_WORDS, PIXELS, SUBPAGE_WORDS, Subpage};

const DUMP: &str = include_str!("data/desk-dump.txt");
const MELEXIS: &str = include_str!("data/desk-melexis.csv");

/// Reads the hex words after the line `name` until the next label.
fn block(name: &str) -> Vec<u16> {
    DUMP.lines()
        .skip_while(|l| l.trim() != name)
        .skip(1)
        .take_while(|l| l.starts_with(|c: char| c.is_ascii_hexdigit()) && !l.starts_with("end"))
        .flat_map(|l| l.split_whitespace().map(|w| u16::from_str_radix(w, 16).unwrap()))
        .collect()
}

fn eeprom() -> [u16; EEPROM_WORDS] {
    block("eeprom").try_into().unwrap()
}

fn subpage(n: u16) -> Subpage {
    let words: [u16; SUBPAGE_WORDS] = block(&format!("subpage {n}")).try_into().unwrap();
    let sub = Subpage { words };
    assert_eq!(sub.number(), n);
    sub
}

#[test]
fn matches_melexis_reference() {
    let cal = Calibration::from_eeprom(&eeprom()).unwrap();
    assert!(cal.bad_pixels().is_empty());

    let mut ours = [f32::NAN; PIXELS];
    for n in [0, 1] {
        let sub = subpage(n);
        let ta = cal.ambient(&sub);
        cal.temperatures(&sub, 0.95, ta - 8.0, &mut ours);
    }

    let theirs: Vec<f32> = MELEXIS.split([',', '\n']).filter(|s| !s.is_empty()).map(|s| s.parse().unwrap()).collect();
    assert_eq!(theirs.len(), PIXELS);
    let worst = ours.iter().zip(&theirs).map(|(a, b)| (a - b).abs()).fold(0.0, f32::max);
    assert!(worst < 0.01, "largest difference from Melexis: {worst} °C");
}

#[test]
fn supply_and_sensor_temperature_are_plausible() {
    let cal = Calibration::from_eeprom(&eeprom()).unwrap();
    let sub = subpage(0);
    // Melexis's driver gives 3.3206 V and 28.9754 °C for this subpage.
    assert!((cal.vdd(&sub) - 3.3206).abs() < 0.0005);
    assert!((cal.ambient(&sub) - 28.9754).abs() < 0.005);
}
