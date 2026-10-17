# Bill of materials

Per unit. Prices are approximate, Oct 2026. Tools not included (soldering iron, multimeter, calipers). The part numbers are what was ordered; the enclosure is sized for them.

**Contents:** [Phase 1: bench](#phase-1-bench) · [Phase 2: handheld](#phase-2-handheld) · [Mechanical](#mechanical) · [Buying notes](#buying-notes)

## Phase 1: bench

About $80. Enough to read frames and zones on the serial console.

| Part | Qty | ~Price | Notes |
| --- | --- | --- | --- |
| Unexpected Maker FeatherS3[D] (Adafruit 6399) | 1 | $25 | ESP32-S3, 16 MB flash, 8 MB PSRAM, LiPo charger, USB Serial/JTAG, 2 STEMMA QT ports |
| MLX90640 sensor, **55° (MLX90640ESF-BAA)**, bare | 1 | $36 | SparkFun SEN-16429. Don't touch the lens; solder the leads quickly; 3.3 V only |
| Perfboard, ~20 × 20 mm | 1 | $3 | Sensor carrier: sensor, two capacitors, cable |
| Through-hole capacitor kit (SparkFun) | 1 | $12 | 100 nF (ceramic, X7R if possible) + 10 µF for the sensor supply. The 10 µF is likely electrolytic: stripe = GND |
| STEMMA QT / Qwiic cable (JST-SH 1.0 mm, 4-pin) to **male headers**, 150 mm (Adafruit 4209) | 1 | $1 | Sensor carrier to the Feather |
| STEMMA QT / Qwiic cable, plug both ends, **100 mm** (Adafruit 4210) | 2 | $2 | Feather to the clock; a second one with one plug cut off, soldered to the screen (its plug points sideways, with no room in the head) |
| Resistors, 2.2–10 kΩ | 2 | on hand | I²C pull-ups, only if nothing else on the bus has them (the OLED and clock boards usually do) |

## Phase 2: handheld

About $70 more, plus the mechanical parts below.

| Part | Qty | ~Price | Notes |
| --- | --- | --- | --- |
| LiPo, 1-cell 3.7 V, 1200 mAh, 34 × 62 × 5 mm (Adafruit 258) | 1 | $10 | JST-PH 2.0, built-in protection, Feather polarity. Others: max ~6 mm thick, 36 mm wide; **check plug polarity** |
| DS3231 clock, STEMMA QT (Adafruit 5188) | 1 | $14 | Keeps time while the board is asleep or the battery is flat |
| Coin cell, **CR1220** | 1 | $2 | For the Adafruit clock (not CR2032) |
| 0.96″ OLED, 128×64, SSD1306, white, STEMMA QT (Adafruit 326) | 1 | $18 | Monochrome: best contrast in daylight. Defaults to I²C |
| Microswitch with roller lever (Adafruit 819, KW11-3Z size) | 1 | $2 | Trigger switch, ~20 × 10 × 6.4 mm body; wire C and NO |
| Tactile button, 6 mm (Adafruit 367, 20 pack) | 1 | $3 | Menu button. Through-hole, 4 legs; any standard 6 × 6 mm tactile button fits |
| Laser diode, 5 mW 650 nm red, dot (Adafruit 1054) | 1 | $6 | Not the "line" version. Dim with PWM |
| Piezo buzzer PS1240 (Adafruit 160) | 1 | $2 | Passive; the firmware drives the tone |
| NPN transistor PN2222 (Adafruit 756, 10 pack) + 1 kΩ base resistor | 2 | $2 | Buzzer and laser drivers |
| Silicone hookup wire, 26–28 AWG | | $8 | |
| Heat-shrink, double-sided foam tape | | $5 | Battery, clock, buzzer |

## Mechanical

| Part | Qty | ~Price | Notes |
| --- | --- | --- | --- |
| M3 socket head screw kit, 6–30 mm | 1 | $10 | Enclosure, trigger hinge |
| M3 heat-set inserts, short (~4 mm OD × 5.7 mm) | ~10 | $10 | Ruthex or CNC Kitchen M3 short |
| M3 nuts | few | in kit | |
| Filament, PETG or ASA | 1 kg | $20 | PLA softens in a hot car; fine for a test fit |

## Buying notes

- **55°, not 110°.** The MLX90640 comes in two versions that look identical. The 55° (BAA) puts ~30 pixels across the tread at 25 cm; the 110° (BAB) only ~10. Adafruit's 4407 is 55° and 4469 is 110°.
- **Not the AMG8833.** Cheaper and easier to find, but it tops out at 80 °C and has only 8 × 8 pixels.
- **STEMMA QT = Qwiic** (small 1.0 mm JST-SH plug). Plain "STEMMA" is Adafruit's older, larger 2 mm connector; avoid "Qwiic to STEMMA" adapter cables. I²C needs the **4-pin** version; the 3-pin JST SH cables are for other things.
- **Through-hole, not SMD.** For hand-soldering on perfboard, capacitors and resistors need wire legs. A part described as "0603" or "0805" is surface-mount.
- **LiPo polarity.** Many JST-PH batteries are wired opposite to Adafruit/Feather convention; plugging one in backwards can kill the board.
- **I²C pull-ups.** With the power off, SDA and SCL should each read a few kΩ to 3.3 V. If not, add the resistors.
