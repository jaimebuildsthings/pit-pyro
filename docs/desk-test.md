# Desk test

The first step of the electronics: the thermal sensor and the FeatherS3[D] on the desk, no enclosure. **Goal:** see what the sensor sees, in °C.

**Contents:** [Sensor carrier](#sensor-carrier) · [Check before power](#check-before-power) · [Flash](#flash) · [Look](#look)

## Sensor carrier

The MLX90640 is a bare TO39 can with four gold legs. Bend them as little as possible: they break off at the glass seal, and then the sensor is done.

- **Pins** (datasheet bottom view, tab up): SCL top left, SDA top right, GND bottom left, VDD bottom right. The tab sits between SDA and SCL.
- **Legs on perfboard:** they sit on a 5.84 mm circle, which doesn't match the 2.54 mm grid in any pattern. A 2 × 1-hole rectangle (two legs in neighbouring holes on each side) is the closest fit. Turn the can so the tab faces the cable end.
- **Clean perfboard pads first.** On our first board they wouldn't take solder at all. Rubbing them with 2000-grit sandpaper until they shine, then wiping with isopropyl alcohol, made the next board solder cleanly. A future carrier will skip the perfboard.
- **Wires:** cut the male pins off a STEMMA QT cable (Adafruit 4209) and solder its wires to the legs: red VDD, black GND, blue SDA, yellow SCL. Heat-shrink each joint.
- **Supply capacitors: don't skip them.** A 100 nF ceramic (marked "104") and a 10 µF electrolytic across VDD and GND, as close to the legs as you can, with the electrolytic's stripe (−) on GND. Melexis asks for them (datasheet section 14). Without them, our second sensor showed stripes every fourth column, 0.7–1.2 °C high; the third sensor, with them, showed none (see [Look](#look)). Soldering the two together first, off the sensor, keeps the heat away from it.
- **Pull-ups:** a 4.7 kΩ resistor (yellow, violet, red) from SDA to VDD and another from SCL to VDD. Port 2 on the Feather has none, and the sensor needs them.
- **Glue:** hot glue over the back holds it together. Keep it 1 mm in from the board's edges so the carrier still slides into the enclosure's rails, and as low as you can: ours stands about 12 mm off the board, which the head still fits.

Solder one leg at a time with a small amount of solder, and check the gap to the neighbouring leg before the next. On a first try, the two neighbouring legs bridged; desoldering broke a leg.

## Check before power

A VDD and GND swap kills the sensor (−0.3 V limit), so check before plugging in.

| Check | Meter | Expect |
| --- | --- | --- |
| Shorts | continuity, every pair of wires | no beep |
| Pull-ups | Ω, blue–red and yellow–red | ~4.7 kΩ each; blue–yellow ~9.4 kΩ |
| Which wire is GND | diode mode, + on black, COM on blue | one diode drop, ~0.6 V |
| | diode mode, + on red, COM on blue | clearly higher (~1.3 V on ours) |

The pin with the single diode drop to SDA is GND. With the meter on the other two pins, the reading drifts, because the meter's current partly powers the chip; don't read anything into that.

## Flash

Plug the cable into the Feather's **I2C2** socket (the STEMMA QT socket nearer the USB-C), connect the Feather to the computer and follow [firmware/README.md](../firmware/README.md#build-and-flash). Then:

```text
> status
pit-pyro 0.1.0 on ESP32-S3 (FeatherS3[D])
sensor: MLX90640 at 0x33 on STEMMA 2, 4 subpages/s
supply 3.32 V, sensor 29.1 deg C
bad pixels: []
> scan
STEMMA 1 (SDA IO8, SCL IO9):
  0x36 MAX17048 battery gauge (on the Feather)
STEMMA 2 (SDA IO16, SCL IO15):
  0x33 MLX90640 thermal camera
```

The sensor's own temperature runs a few degrees above the room; the chip warms itself.

## Look

`frame` prints a text map. For a picture, close the monitor and open [`tools/viewer.html`](../tools/README.md) in Chrome. A face against a room is a good first test: about 32–36 °C against 23 °C.

Hold the carrier by the board, not the can, and wait **4 minutes** after power-on before trusting the temperatures (Melexis's warm-up; the viewer's run timer shows "settling" until then).

On an even surface, the image should be flat to within a few tenths of a degree. If every fourth column reads high, check the supply capacitors first:

| Sensor | Capacitors | Every 4th column | Spread |
| --- | --- | --- | --- |
| Second | none, held by the can | +0.7 to +1.2 °C | 0.7–0.9 °C |
| Third | 100 nF + 10 µF, held by the board, 4 min warm-up | ±0.1 °C | 0.4 °C |

Next: the trigger, menu button, buzzer and laser on the Feather's pins, then the clock and screen on STEMMA QT port 1.
