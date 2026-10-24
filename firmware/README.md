# Firmware

Rust, no ESP-IDF: `esp-hal` and Embassy on the FeatherS3[D] (ESP32-S3).

| Path | Contents |
| --- | --- |
| `crates/mlx90640/` | Driver for the MLX90640 thermal array and its temperature maths, ported from Melexis's reference driver. Tested against Melexis's own C code on a dump from a real sensor |
| `crates/protocol/` | The console's command language, and thermal images as text (map and CSV) |
| `pyro/` | The ESP32-S3 program |

This is the desk-test stage: the firmware reads the sensor and prints what it sees. The trigger, menu, screen, clock and run log come next ([plan](../docs/plan.md#firmware)).

## Host tests (stable Rust, no hardware)

```sh
cd firmware
cargo test
```

## One-time setup for the ESP32-S3 build

```sh
cargo install espup espflash
espup install          # Espressif's Rust toolchain for Xtensa chips
```

In every new shell before building: `source ~/export-esp.sh`.

## Build and flash

Plug the Feather into the computer with USB-C (the console runs on the chip's own USB port), then from `firmware/pyro`:

```sh
cargo run --release
```

That flashes and opens the serial monitor. Type `help`.

To open the console later without flashing, run `espflash monitor` and press **Ctrl+R** once: the monitor stops the chip in its bootloader to identify it, and typing does nothing until the reset starts the firmware. If flashing can't connect, put the Feather in download mode (hold **BOOT**, tap **RST**, let go of **BOOT**), flash, then tap **RST** to run.

To see the image instead of text, close the monitor and open [`tools/viewer.html`](../tools/README.md).

## Pins

| Signal | FeatherS3[D] |
| --- | --- |
| Thermal sensor | STEMMA QT port 2 (silkscreen I2C2, the socket nearer the USB-C): SDA IO16, SCL IO15, powered by LDO2 |
| LDO2 enable (port 2's power, off in deep sleep) | IO39 |
| STEMMA QT port 1 (I2C1; the Feather's battery gauge is on it) | SDA IO8, SCL IO9 |
| Console | USB-C (USB Serial/JTAG) |

Port 2 has no pull-up resistors on the Feather and the bare sensor has none either, so the sensor carrier has two 4.7 kΩ resistors from SDA and SCL to 3.3 V. Pins planned for the rest of the build: trigger IO5, menu button IO6, buzzer IO12, laser IO14 (through a transistor).

## Console

```text
status                   firmware, sensor, refresh rate, supply, sensor temperature
scan                     list devices on both I2C buses
frame                    one thermal image as a text map
frame csv                one thermal image as 24 rows of 32 temperatures (deg C)
stream on|off            a text map for every new image
emissivity E             surface emissivity, 0.1-1.0 (default 0.95)
rate HZ                  subpages per second: 0.5 1 2 4 8 16 32 64 (default 4)
dump                     EEPROM and two raw subpages in hex
```

The sensor measures half its pixels at a time, in two chessboard *subpages*; a full image takes two. At the default 4 subpages a second, that's 2 images a second.

`dump` output is what the test in `crates/mlx90640/tests/` uses: to check the maths against another sensor, save a dump to `tests/data/` and run Melexis's C driver on it the same way.

## Stripes in the image

If every fourth column reads high on an even surface, it's in the sensor's raw data, not the maths: on the desk test it came from a sensor wired without its supply capacitors (and held by the can). With a 100 nF and a 10 µF across VDD and GND at the sensor, the stripes went from +0.7–1.2 °C to within ±0.1 °C. See [docs/desk-test.md](../docs/desk-test.md#look).
