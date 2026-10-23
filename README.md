# pit-pyro

DIY handheld tire pyrometer for autocross and track days. A thermal camera (MLX90640) reads the tread's outer, middle and inner temperatures in one trigger pull; an ESP32-S3 shows them on a small screen and stores every reading, time-stamped, on the device. Scan four tires in under a minute, with no phone or network needed.

> **Status: desk test.** The enclosure (v0.4) is printed. The firmware reads the thermal sensor and shows its image in °C ([desk test](docs/desk-test.md)); the trigger, screen and run log are next. See [docs/plan.md](docs/plan.md) for the roadmap.

> [!WARNING]
> **The aiming laser can damage eyes.** Use the lowest-power module you can find and never point it at people. The LiPo battery can catch fire if it's damaged, shorted or overheated: don't leave the device in a hot car. In the pits, watch for moving cars and hot brakes while you scan. Build and use this at your own risk; see the license for the warranty disclaimer.

![Enclosure cutaway with the parts in place](hardware/enclosure/images/cutaway.png)

## What's here

| Path | Contents |
| --- | --- |
| [`docs/`](docs/) | [Plan](docs/plan.md) · [Parts](docs/bom.md) · [Desk test](docs/desk-test.md) |
| [`hardware/`](hardware/) | [Pistol-grip enclosure](hardware/enclosure/README.md): OpenSCAD model, printable STLs, renders |
| [`firmware/`](firmware/) | [Rust firmware](firmware/README.md) (`esp-hal`, Embassy): reads the MLX90640 and prints its image at the serial console |
| [`tools/`](tools/) | [Live thermal viewer](tools/README.md) for the browser |

## Quick start

The STLs in [`hardware/enclosure/stl/`](hardware/enclosure/stl/) are ready to slice. To change the model, open [`enclosure.scad`](hardware/enclosure/enclosure.scad) in [OpenSCAD](https://openscad.org), adjust the parameters to your measured parts, and re-export as described in the [enclosure README](hardware/enclosure/README.md#changing-the-model).

## License

- Code (`firmware/`, `tools/`): [Apache-2.0](LICENSE). The MLX90640 maths is ported from Melexis's Apache-2.0 reference driver
- Documentation (`docs/`, READMEs): [CC-BY-SA-4.0](LICENSE-docs)
- Hardware designs (`hardware/`): [CERN-OHL-P-2.0](hardware/LICENSE)

Contributions are welcome; see [CONTRIBUTING.md](CONTRIBUTING.md) and the [Code of Conduct](CODE_OF_CONDUCT.md).
