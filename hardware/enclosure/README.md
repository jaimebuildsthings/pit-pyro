# Enclosure (phase 2 prototype)

A pistol-grip shell for the handheld scanner in [docs/plan.md](../../docs/plan.md). It is a parametric [OpenSCAD](https://openscad.org) model, [`enclosure.scad`](enclosure.scad), split down the middle into two halves that print flat without supports and screw together.

![Cutaway: left half with the parts in place](images/cutaway.png)

| Front: aperture and aiming dot | Rear: screen and menu button |
| --- | --- |
| ![Front view](images/front.png) | ![Rear view](images/rear.png) |

About 78 × 145 × 35 mm overall. The head is short and tall (61 mm long at the bottom, 47 at the top, 49 tall) with square edges. Its rear face leans forward 16°, in line with the grip, so the screen faces up toward you while you aim down at a tire; the face overhangs the grip by 7 mm, so your hand's web stops under it. Version 0.2, with a long head and the screen on top, is kept on the `enclosure-v0.2` branch.

The grip is shaped for the hand:

- **Length:** 88 mm below the head (`grip_l`), for four fingers of a medium to large hand.
- **Section:** 47 × 35 mm, with its long edges rounded at 8 mm (`edge_r`), so it's close to an oval. The rounding finishes in a 45° bevel at each print bed, so the halves still print without supports, and the inside follows it so the wall stays 2.4 mm.
- **Palm swell:** the backstrap bulges 3 mm where your palm sits (`swell`, `swell_v`).
- **Flared butt:** the bottom widens 3 mm at the back and 2 mm at the front (`flare`, `flare_l`), so the device doesn't slide out of your hand.

The sensor looks out through an **open aperture**: there's no window, because glass and clear plastic block thermal IR.

## Layout

| Where | What |
| --- | --- |
| Head, front | MLX90640 (bare TO39) on a perfboard carrier, its lens 1 mm behind the front face; the carrier's edges slide into rails |
| Head, front top | Laser bore, tilted so the dot lands on the sensor axis 250 mm out (`aim_dist`) |
| Head, rear face | 0.96″ OLED behind a bevelled window, header at the top |
| Head, rear face, below the screen | Menu button: a 6 mm tactile button behind a printed cap, under the thumb. It sits in a holder on the left half, with its legs out the back past a backstop bar |
| Head, right wall | Piezo buzzer, with sound holes |
| Grip, front top | Printed trigger: an 18 mm wide paddle filling an opening at the top of the grip's front, just under the head, hinged at its top on an M3 screw. The bottom swings in about 2.5 mm and presses a roller-lever microswitch held on pins behind it. The switch lever returns it; a shoulder below the opening is the rest stop and the switch cradle the full-pull stop |
| Grip, left wall | 1200 mAh LiPo, from just under the head down, between two stops |
| Grip, middle | FeatherS3[D], standing on its edge with its parts facing the clock, between the battery and the clock. Below the battery, rails on the left half hold its long edges; two posts on the right half stop its top end, so a USB plug can't push it up the grip |
| Grip, butt | The Feather's USB-C (charging, firmware, serial console) behind a press-fit cover. It's the only port |
| Grip, right wall | DS3231 clock board, between four corner stops, with room for a STEMMA QT plug at each end |

## Wiring

The I²C parts chain on STEMMA QT cables: Feather → clock → screen, and the sensor carrier on the Feather's second port.

- **Sensor carrier:** a STEMMA QT to male-header cable (Adafruit 4209), its pins soldered into the perfboard.
- **Clock:** a plug-to-plug cable (Adafruit 4210) from the Feather.
- **Screen:** the board's STEMMA QT sockets point sideways, and the head has no room for a plug beside the screen. Instead, cut one plug off a 4210 cable (keep the full 100 mm, the clock is down in the grip) and solder its wires to the screen's header holes: **Vin** (red), **GND** (black), **Data** = SDA (blue), **Clk** = SCL (yellow). Plug the other end into the clock's second socket.
- **Menu button:** wire two diagonally opposite legs, so you get the switch whichever way round it goes in. One goes to GND, the other to a free GPIO that can wake the board (GPIO 1–21). Trim the other two legs.
- **Feather:** solder the switch, menu button, buzzer and laser wires from the top (the side facing the clock) and trim the leads flush underneath, because the board sits close to the battery and the rails hold its long edges (0.8 mm, `fth_edge`). The sensor's 150 mm cable reaches from the head down to the Feather.

## Measure your parts first

Most dimensions come from datasheets and listings. Measure your actual parts with calipers and update the `/* [Parts] */` block before printing. Width and clearance changes ripple through the layout, so re-run the check afterwards (see [Changing the model](#changing-the-model)).

| Parameter | Assumed | Most likely to be wrong |
| --- | --- | --- |
| `car`, `car_back`, `mlx_can_h` | 20.3 × 20.3 mm carrier (8 × 8 holes); 10 mm of parts behind it; lens face 5 mm above it | How high the sensor sits once soldered |
| `fth`, `fth_top_h`, `fth_bot_h`, `usb_y` | 52.3 × 22.9 mm, 6.6 mm overall at the battery plug, 1.5 mm underneath, USB-C centred | Where the USB-C sits across the board; how tall the parts are on each side, since the board sits between the battery and the clock |
| `oled_pcb`, `oled_win_ds` | Adafruit 326: 29.2 × 26.7 mm, active area 2 mm toward the header | Where the active area sits on the module |
| `rtc` | Adafruit 5188: 25.4 × 17.8 mm, 8 mm thick with the coin cell | Thickness |
| `bat` | Adafruit 258: 34 × 62 × 5 mm (5.5 allowing for swelling) | |
| `laser_d`, `laser_l` | Adafruit 1054: 6 mm tube, 14 mm long | Length |
| `buzzer_d`, `buzzer_h` | Adafruit 160 (PS1240): 12.2 mm, 7 mm tall | Height |
| `btn`, `btn_stem` | Adafruit 367: 6.2 mm square body, 3.5 mm tall, 3.5 mm stem 1.5 mm above it | Body height and stem height: together they set where the cap meets the stem |
| `sw`, `sw_hole_*`, `sw_lever` | Adafruit 819: 19.8 × 10.2 × 6.4 mm body, holes Ø2.3 at 9.5 mm, 2.9 mm from the terminal edge; roller 5 mm proud of the body | **The roller's free height** (`sw_lever`) and how far the switch travels before it clicks (`trig_travel`, 2.5 mm) |

## Printing

![Right half as it prints: outer face down, inserts, switch cradle, Feather stops and clock stops facing up](images/right-half.png)

| File | Notes |
| --- | --- |
| [`stl/enclosure-left.stl`](stl/enclosure-left.stl) | Counterbored screw holes, alignment lip, battery stops, Feather rails, menu button holder |
| [`stl/enclosure-right.stl`](stl/enclosure-right.stl) | Heat-set insert holes, switch cradle and pins, Feather stops, buzzer ring, clock stops |
| [`stl/usb-cover.stl`](stl/usb-cover.stl) | Press-fit cover for the USB opening |
| [`stl/trigger.stl`](stl/trigger.stl) | Trigger, printed on its side. If it rubs in its opening, sand its sides lightly |
| [`stl/button-cap.stl`](stl/button-cap.stl) | Menu button cap, printed flange down |

- The STLs are already oriented: flat outer face down, open side up. No supports, no brim needed.
- PETG or ASA is best, since a car at an event sits in the sun and PLA softens around 55 °C. PLA is fine for a first fit check.
- 0.2 mm layers, 3–4 perimeters, 20 % infill. Each half is about 78 × 145 mm.
- The screw counterbores and the hinge nut pocket have two stepped bridging layers built in, so they print without support. Drill or cut them out if your slicer leaves a skin.
- The switch pins (2 mm thick) and the Feather stops (3 × 3 mm) on the right half stand 20–25 mm off the bed. Print them slowly, or bump the infill so they come out solid.
- If the cover is too tight or too loose, change `usb_cover_fit` and reprint just the cover.

## Hardware

- 5 × M3 × 20 socket head screws, plus 1 × M3 × 30 and an M3 nut for the trigger hinge
- 5 × M3 heat-set inserts, about 4 mm OD × 5.7 mm long, pressed into the right half (`insert_d`, `insert_depth`)
- Foam tape for the battery and the clock board

## Assembly

1. Press the heat-set inserts into the right half.
2. Push the microswitch onto its pins in the right half, roller toward the trigger, with a drop of glue: nothing on the left half holds it on. Stick the clock board to the right wall between its stops, and the buzzer into its ring.
3. Drop the button cap into its hole in the left half's rear wall from the inside, flange inward, then slide the menu button into its slot behind it, legs pointing up and down. A dab of glue keeps the button in its slot while you work. Push the laser into its bore (a dab of glue holds it). Drop the sensor carrier into its rails and the OLED into its slots. Stick the battery to the left wall between its stops, then slide the Feather into its rails over it, parts up and USB-C into the opening in the butt.
4. Fit the trigger into its opening and wire everything (see [Wiring](#wiring)). Keep the I²C runs short.
5. Close the halves (the lip on the left half locates the right) and screw them together. The M3 × 30 goes through the trigger's hinge, into the nut recessed in the right side.

The OLED's header ends up at the rear, so the image will be upside down unless the firmware rotates it 180°. The SSD1306 can do that with a register setting.

## Changing the model

Open `enclosure.scad` in OpenSCAD and press **F5** to preview. Use the `part` setting (or the Customizer panel) to switch between the `assembly`, `cutaway`, `left`, `right`, `usb_cover`, `trigger` and `btn_cap` views; `pull` (0 to 1) swings the trigger. Then export:

```sh
openscad --export-format binstl -D 'part="left"'      -o stl/enclosure-left.stl  enclosure.scad
openscad --export-format binstl -D 'part="right"'     -o stl/enclosure-right.stl enclosure.scad
openscad --export-format binstl -D 'part="usb_cover"' -o stl/usb-cover.stl       enclosure.scad
openscad --export-format binstl -D 'part="trigger"'   -o stl/trigger.stl         enclosure.scad
openscad --export-format binstl -D 'part="btn_cap"'   -o stl/button-cap.stl      enclosure.scad
```

`part="check"` is the fit check: it renders where the stand-in parts overlap the shell and where the two halves overlap each other. A clean design leaves only zero-thickness contact faces, such as boards resting in their slots. Run it after any layout change.

To try a different grip length, change `grip_l` and re-run the check. The Feather sits at the butt and the trigger switch at the top, so below about 82 mm they meet. Below about 85 mm, a large hand's little finger hangs off the end anyway. `rear_tilt` sets how far the screen leans back, and `rear_x` the overhang over your hand.

## Open questions

- **Grip feel.** The length, section and swell are from typical hand sizes. Print one in PLA and hold it before printing the real one: `grip_l`, `grip_angle`, `swell` and `flare` are the knobs. `grip_d` and `edge_r` are tied to the battery's width: rounder edges need a deeper grip.
- **Screen angle.** The rear face leans 16°, in line with the grip. Hold the print at a tire and check that you can read the window; `rear_tilt` changes it.
- **A full grip.** Battery, Feather, clock and switch share the grip, with the wires from the head running down past them. Check there's room for the wiring before printing in PETG.
- **Menu button reach.** It's where the thumb lands when it reaches over the back of the head. If that's awkward, `btn_z` moves it up or down the rear face.
- **Trigger feel** depends on the microswitch's roller lever. If it's too light to return the trigger, add a small compression spring between the paddle and the cradle.
- **Weather.** The open aperture and sound holes let dust in. A thin IR-transparent film over the aperture (germanium, or a thin polyethylene film) would need testing before use.
