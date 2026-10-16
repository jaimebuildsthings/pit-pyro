# Contributing

Thanks for helping. This project is a prototype, so issues describing what you built, what you measured and what went wrong are as valuable as code.

## Ground rules

- **Dependencies:** every new crate, package or library needs a permissive, Apache-2.0-compatible license (MIT, Apache-2.0, BSD, ISC, Zlib). No GPL, LGPL or other copyleft code. Say which license it uses in the pull request.
- **Enclosure changes go through the parameters** in [`enclosure.scad`](hardware/enclosure/enclosure.scad). Commit the regenerated STLs and renders with the source change, so they always match.
- **Measurements:** when a change is based on a measured part, say what you measured and with what. When a change affects readings, include before/after temperatures against a reference thermometer.
- **Safety:** don't weaken the safety guidance in the docs, especially for the laser and the battery.

## Before you open a pull request

For enclosure changes, run the fit check and re-export the STLs (see [Changing the model](hardware/enclosure/README.md#changing-the-model)):

```sh
cd hardware/enclosure
openscad -D 'part="check"' -o /tmp/check.stl enclosure.scad
```

The check should leave only zero-thickness contact faces.

## Licensing of contributions

By contributing, you agree that your contribution is licensed under the license of the part of the repository it touches: Apache-2.0 for code, CC-BY-SA-4.0 for documentation, CERN-OHL-P-2.0 for hardware designs.
