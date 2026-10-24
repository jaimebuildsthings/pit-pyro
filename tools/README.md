# Tools

## Thermal viewer

Live image from the sensor in the browser: [`viewer.html`](viewer.html).

1. Flash the firmware (see [firmware/README.md](../firmware/README.md)) and close any serial monitor.
2. Open `viewer.html` in **Chrome or Edge** (double-click it; no server needed). They support Web Serial; Firefox and Safari don't.
3. Click **Connect** and pick the Feather's USB JTAG/serial port.

It asks the Feather for a `frame csv` as soon as the last one arrives, about two images a second, and draws each with min, max and the centre reading (the circle, where the aiming dot points). A run timer counts from the first image and shows "settling" for the first 4 minutes, Melexis's warm-up before absolute temperatures can be trusted. **Rotate** and **Mirror** turn the picture; **fixed range** holds the colours to a set temperature range, so frames can be compared.

Nothing leaves your machine.
