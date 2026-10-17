# Handheld Tire Temperature Scanner: Plan

Oct 14, 2026 · @jaimebuildsthings · Status: draft

**Contents:** [Goal](#goal) · [How it works](#how-it-works) · [Hardware](#hardware) · [Firmware](#firmware) · [Phases](#phases) · [Open questions](#open-questions)

## Goal

A cordless handheld that reads a tire's tread temperature **by zone** (outer, middle, inner) in one trigger pull, steps through all four corners, and **stores every reading on the device**, time-stamped, for review after the event and a revised alignment in [stringline.ing](https://stringline.ing).

**Designed for autocross and track days:** between runs or sessions there's no time to connect a phone or upload anything. Pull up, scan four tires, go. Everything you need at the car is on the device: its screen shows which corner is next and what it read.

It reads the surface with a thermal camera, so it works without touching the tire. Surface temperatures fall quickly once the car stops: scan within about a minute of finishing a run, always in the same order.

## How it works

```mermaid
flowchart TB
    tire["Tire tread"] -- "thermal IR" --> cam["MLX90640<br/>32×24 thermal array"]
    cam -- "I²C" --> esp["ESP32-S3"]
    trig["Trigger button"] --> esp
    menu["Menu button"] --> esp
    esp --> screen["Small screen<br/>next corner · last zones"]
    esp --> fb["Buzzer + aiming LED"]
    esp --> store[("Flash: time-stamped runs")]
    clock["Clock module<br/>(DS3231)"] --> esp
```

**At the car, between runs:**
1. **Pull** the trigger to wake the device. The screen shows `Run 4 · LF ▸`.
2. Point at the LF tread from about 25 cm, centered with the aiming dot, and **pull**. The device averages a few frames, finds the tire in the image and splits the tread into zones.
3. A **beep**, then the screen shows the zones (`LF 78 81 85`) and moves to `RF ▸`. Repeat for RF, LR, RR.
4. After RR, the screen shows a one-line summary per tire and stores the run. Done; no phone involved.

Every four reads make a run. **Hold** the trigger to turn off (deep sleep, with the peripherals' regulator switched off); the device also goes to sleep after a few idle minutes.

**The menu:** a small button on the back, under your thumb. Press it to open the menu and again to step through the items; the **trigger** selects. With no press for about 5 s, it goes back to scanning. Either button wakes the device.

| Item | Does |
| --- | --- |
| New session | Starts a fresh session (an event day); earlier sessions are kept |
| View | Steps through stored sessions and runs: each corner's zones, and single readings |
| Delete last reading | Throws away a bad scan, so you can redo that tire |
| Clear sessions | Erases every stored reading |
| Settings | Units (°F / °C), emissivity, set the clock |
| Sync | Wi-Fi, for getting runs off the device |
| Info | Battery, storage used, firmware version |
| Reset settings | Back to defaults; stored readings are kept |
| Sleep | Off now |

Anything that deletes shows "Hold trigger to confirm" and needs a 2 s hold, so a bump in the pits can't wipe a session.

**After the event:** the stored runs are reviewed for trends, and drive camber and pressure changes for the next alignment.

**Reading a tire:** a hotter inside edge means too much negative camber; a hotter middle means pressure too high; hotter edges mean pressure too low.

## Hardware

The full shopping list, with what to look for, is in [bom.md](bom.md).

| Part | Why | ~Cost |
| --- | --- | --- |
| MLX90640 sensor, **55° field of view (BAA)**, bare TO39 on a small perfboard carrier | Covers ~26 cm at 25 cm, so the whole tread with margin. Range −40 to 300 °C | $36 |
| **Unexpected Maker FeatherS3[D]** (ESP32-S3, 16 MB flash, 8 MB PSRAM) | LiPo charger, battery sense and USB Serial/JTAG on one USB-C; two STEMMA QT ports for the I²C parts; a second, switchable 3.3 V regulator to cut power to the peripherals. 16 MB flash holds thousands of runs | $25 |
| **Screen: 0.96″ OLED, 128×64 (SSD1306, I²C)** | Next corner and last reading; high contrast for daylight at the track; shares the I²C bus with the sensor and clock | $5 |
| **Clock module, DS3231** | Real timestamps on every run, kept on a coin cell while the device is off | $5 |
| 1-cell LiPo, ~1000 mAh, JST-PH, with protection circuit | Cordless, lasts an event day; charged by the board | $8 |
| Printed trigger + roller-lever microswitch (KW11-3Z size) | Pull = capture; also wakes the device from deep sleep. Hold = sleep. Selects in the menu | $1 |
| Menu button: 6 mm tactile button behind a printed cap, on the back of the head | Opens and steps through the menu; also wakes the device | $0.15 |
| Aiming aid: small laser dot module or bright LED | Center the sensor on the tread | $3–5 |
| Buzzer or vibration motor | "Captured" without looking | $2 |
| 3D-printed pistol-grip enclosure, **open aperture** in front of the lens, window for the screen | Glass and clear plastic block thermal IR | — |

Notes:
- **Laser safety:** use a low-power module and never point it at eyes.
- **Keep I²C short:** sensor, screen, clock and ESP32 all in one housing.
- **One USB-C:** the board's port, behind a cover in the grip's butt, does charging, firmware updates and the serial console.
- **Emissivity:** tire rubber is about 0.95; adjustable in the menu.

## Firmware

Same stack and conventions as corner-balancer: Rust, `esp-hal` + Embassy, host-tested `no_std` crates, Apache-2.0 code.

| Piece | Notes |
| --- | --- |
| MLX90640 driver | Reads the EEPROM calibration, then converts frames to °C. Melexis's reference C driver is Apache-2.0; check Rust crates first (license!) before porting |
| `zones` crate (`no_std`, host-tested) | Find the tire in the frame (warm band vs. background), split it into 3 (or 5) zones, and average each zone's pixels over several frames |
| Capture flow | Trigger → average 4–8 frames → zones → beep → show → store → next corner. Every four reads make a run. Hold or idle timeout = deep sleep; either button wakes it (ext1 wake on two pins) |
| **Run log** (host-tested record format) | Append-only log in a dedicated flash partition (like corner-balancer's `calib`, but larger). One record per corner reading: session, run number, corner, zones and clock time. "Delete last reading" appends a tombstone record; "Clear sessions" erases the partition. Survives power-off and firmware updates |
| Screen UI | Big text: run number, next corner, last zones. Battery level. Scanning needs no menu; the menu is a list stepped by the menu button and selected by the trigger, with a 5 s timeout back to scanning |
| Settings | Units, emissivity: a small two-slot crash-safe record, separate from the run log |
| Record format | Same JSON-lines style as the scales protocol, e.g. `{"v":1,"type":"tire","session":2,"run":4,"corner":"LF","zones_c":[78.4,81.2,84.9],"time":"2026-10-18T10:42:07"}` |

Reuse from corner-balancer: the console, the two-slot crash-safe flash record, the JSON frame style, and corner names (LF/RF/LR/RR).

## Phases

1. **Bench:** MLX90640 + ESP32-S3 on a desk. Serial console prints a 32×24 frame and zone temps. Check accuracy against a reference thermometer, a cup of hot water and a warm pan. *Gate:* zones within ±2 °C of a reference at 60–100 °C.
2. **Handheld prototype:** battery, trigger, menu button, screen, clock, buzzer, aiming aid, printed grip, run log. *Gate:* a full four-tire scan in under 60 s after a drive, repeatable within ±2 °C, all runs still stored with correct times after a power cycle.
3. **Later, maybe:** getting runs into stringline, a probe-pyrometer mode (K-type needle + MAX31856) for under-surface temps, or the on-car version (one sensor per wheel on the upright, ESP-NOW).

## Open questions

- 3 zones or 5? Pro pyrometers use 3; the array could do more.
- Rust MLX90640 driver: an existing crate (license?) or a port of Melexis's Apache-2.0 driver.
- Deep-sleep current: measure it, and check the battery still holds a charge after a week or two off.
