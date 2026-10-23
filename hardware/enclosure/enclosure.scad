// Handheld tire temperature scanner: pistol-grip enclosure, v0.4 (phase 2 prototype)
//
// Units: mm. Model frame: +X points at the tire, +Z is up, Y runs across the
// grip. The shell splits at Y = 0 into a left and a right half. Each half
// prints on its flat outer face, open side up, with no supports.
//
// Export:  openscad --export-format binstl -D 'part="left"'      -o stl/enclosure-left.stl      enclosure.scad
//          openscad --export-format binstl -D 'part="right"'     -o stl/enclosure-right.stl     enclosure.scad
//          openscad --export-format binstl -D 'part="trigger"'   -o stl/trigger.stl             enclosure.scad
//          openscad --export-format binstl -D 'part="btn_cap"'   -o stl/button-cap.stl          enclosure.scad
//          openscad --export-format binstl -D 'part="test_trigger"' -o stl/test-trigger.stl   enclosure.scad
// Parts:   assembly (closed shell), cutaway (left half + stand-in parts),
//          left, right, trigger, btn_cap (each laid flat, ready to slice), check
//          (renders nothing but contact faces when every part fits);
//          test_trigger: a small test print of the trigger and switch.
//
// Licensed under CERN-OHL-P-2.0, see ../LICENSE.

part = "assembly"; // [assembly, cutaway, left, right, trigger, btn_cap, check, test_trigger]
pull = 0;          // [0:0.1:1] trigger position in the views: 0 at rest, 1 fully pulled

/* [Shell] */
wall = 2.4;
chamfer = 1.2;        // head: edge chamfer on the flat faces
edge_r = 8;           // grip: rounding of the long edges; turns into a 45° bevel at each print bed
clear = 0.3;          // fit clearance per side
corner_r = 5;
fillet_r = 8;         // grip-to-head blend
lip_t = 1.2;          // alignment lip on the left half
lip_h = 1.5;

/* [Parts: measure yours and adjust] */
// MLX90640ESF-BAA (bare TO39, SparkFun SEN-16429) on a perfboard carrier:
// carrier width (Y), height (Z), thickness: a 20 mm square cut from a 2 x 8 cm strip.
car = [20, 20, 1.6];
car_back = 10;        // room behind the carrier for the capacitors, joints and cable
mlx_can_d = 9.4;
mlx_can_h = 11.4;     // lens face above the carrier: the can sits flush, its 6 mm legs leave nothing to raise it
// Unexpected Maker FeatherS3[D]: length (down the grip), width (across it), thickness
fth = [51.4, 22.9, 1.1];
fth_top_h = 5.6;      // tallest part above the PCB (battery JST, 6.7 mm overall)
// Parts above the PCB, from the top (antenna) end: [length, height] zones. The rest is fth_top_h.
fth_zones = [[16, 2], [20.4, 4.4]]; // ESP32-S3 and u.FL (under 2 mm, from photos); then the STEMMA QT sockets and buttons (4.4, measured)
fth_bot_h = 1.5;      // parts and trimmed leads under the PCB
fth_edge = 0.8;       // how much of each long edge the side rails hold
usb_y = 0;            // centre of the USB-C port, across the board
// Adafruit 326 0.96" OLED (STEMMA QT): width (Y, along the header), height (up the rear face), PCB thickness
oled_pcb = [29.5, 31.7, 1.6];
oled_glass_t = 1.7;   // glass + tape above the PCB (3.3 mm from the PCB's back to the glass)
oled_bot_h = 3.1;     // STEMMA QT sockets under the PCB
oled_glass = [26.7, 19.25]; // glass, including the driver strip at its bottom...
oled_glass_top = 5;   // ...starting this far below the PCB's header edge
oled_hold = [4.2, 1.5]; // corner slots: how far they reach in from each side, and how much of the top and bottom edges they hold
oled_win = [24, 13];  // visible window (across, up the face), active area 21.7 x 10.9 plus margin
oled_win_ds = 3.6;    // window centre offset from the module centre, up the face (the active area sits toward the header, at the top; from the panel datasheet, check with the screen lit)
// Adafruit 5188 DS3231 clock: length (socket to socket, across the grip), width, thickness incl. coin cell and sockets
rtc = [25.85, 17.75, 8];
rtc_plug = 6;         // room past each end for a STEMMA QT plug
// Adafruit 258 LiPo, 1200 mAh: width, length, thickness incl. swelling
bat = [34.65, 61.6, 5.5];
usb_c = [9.0, 7.3, 3.2];      // receptacle width, depth, height
laser_d = 10.1;               // Adafruit 1054 laser tube
laser_l = 30.25;
buzzer_d = 11.8;              // Adafruit 160 piezo (PS1240)
buzzer_h = 6.6;
// Roller-lever microswitch (Adafruit 819, a standard-size V switch): length, height (lever side to back), thickness
sw = [28, 15.95, 9.85];
sw_holes = [[2.85, 2.75], [25.15, 13.05]]; // hole centres, from the terminal end and from the lever side
sw_hole_d = 3.2;
sw_roller = [9.65, 8.25, 4.5]; // roller: outer edge above the lever side at rest, centre past the far end, diameter
sw_roller_w = 7.75;           // roller width, across the switch
sw_hinge = 9;                 // lever hinge, from the terminal end (for the stand-in only)
sw_click = 2.7;               // roller travel to the click...
sw_bottom = 4.1;              // ...and to where the lever bottoms out
sw_term = 10;                 // room past the terminal end: the NO tab trimmed to 7 mm (NC cut off), plus the wire joint
sw_com = [5.5, 13, 5.5];      // COM tab on the back (the side opposite the lever): room it and its wire joint need off the back face, and its span from the terminal end
// 6 mm tactile button (Adafruit 367), the menu button: body square, body height,
// stem diameter and height above the body. Its legs come out of two opposite sides.
btn = [6.1, 2.7];
btn_stem = [3.5, 1.5];
// M3 screws into heat-set inserts
screw_d = 3.4;
screw_head_d = 6.2;
screw_head_h = 3.0;
insert_d = 4.0;
insert_depth = 8;
boss_d = 7.5;

/* [Layout] */
head_l = 58;          // front face, from the grip's rear edge at the top
head_h = 49;
rear_x = -3;          // bottom of the rear face: it overhangs the grip a little, so your hand's web stops under it
rear_tilt = 16;       // rear face leans forward at the top, so the screen faces up toward you while you aim down
grip_x0 = 4;          // rear edge of the grip where it meets the head
grip_d = 47;          // grip front-to-back (deep enough for edge_r around the battery)
grip_l = 88;          // grip length below the head: four fingers of a medium to large hand
grip_angle = 16;      // rake, butt toward the rear
swell = 3;            // palm swell: how far the backstrap bulges...
swell_v = [12, 80];   // ...between these points down the grip
flare = [3, 2];       // butt flare, rear and front
flare_l = 16;         // flare length up the grip
sens_z = 15;          // sensor axis height
laser_z = 39;         // aiming dot height at the front face
aim_dist = 250;       // laser crosses the sensor axis this far out
// Trigger: a wide paddle filling an opening at the top of the grip's front,
// hinged at its top inside the grip (like a handheld IR thermometer's).
trig_pivot = [grip_d - 7, -7]; // hinge, grip-local [u, v], just under the head
trig_top = -8;        // top of the opening (v)
trig_bot = 16.8;      // bottom of the paddle's face (v)
trig_back = grip_d - 7.5; // back face of the paddle, inside the grip (u); it presses the switch's roller
trig_proud = [3, 7];  // how far the face stands proud of the grip, top and bottom
trig_t = 14;          // paddle width (Y), centred; 2 mm clear of a swollen battery, and wide enough for the switch's roller
trig_pad = 3;         // the paddle's inside part reaches this much further right, under the switch's roller; the face stays centred
trig_ear = [4, 8];    // stop ear past that, on the right, inside the grip: width (Y), height (v). It meets a post on the right wall at full pull
// The switch stands upside down across the split (the Feather is beside it, left),
// terminals down, its roller against the paddle's back face.
sw_roll_v = 10.5;     // roller centre, down the grip: lower gives more roller travel for the same swing, but the switch's end must clear the Feather's STEMMA QT sockets
sw_preload = 0.5;     // the roller pushes the trigger forward against its rest stop by this much
sw_pull = 3.1;        // roller travel at full pull: 0.9 past the click, 0.5 short of bottoming out (not clicking is worse than bottoming)
sw_y = -0.45;         // switch's inner face, just left of the split: its roller lands 4.5 mm right of centre, on the paddle
sw_screw_l = 16;      // M3 screw through the switch and its post into a nut
sw_pin_l = 14;        // M3 screw from outside, through the other post, its shank the switch's locating pin
nut_af = 5.5;         // M3 nut across flats, for the hinge screw
buzzer_xz = [28, 20];
oled_s = 15.5;        // bottom edge of the OLED module, up the rear face
rtc_v = 57.5;         // clock board top edge, along the grip; it lies across the grip below the switch's wires
bat_v = 2.5;          // battery top edge, along the grip; it sits beside the switch
// Menu button: on the rear face below the screen, where the thumb reaches over.
btn_s = 8.5;          // centre, up the rear face
btn_cap_d = 7;        // printed cap: plunger through the rear wall...
btn_cap_proud = 1.5;  // ...standing this far out of it...
btn_flange = [10, 1.2]; // ...and a flange inside that keeps it in (diameter, thickness)

$fa = 4;
$fs = 0.4;

// ---- derived ------------------------------------------------------------

W = oled_pcb[0] + 2 * clear + 2 * wall;       // overall width, set by the OLED
Wi = W / 2 - wall;                            // inner half-width
oled_win_s = oled_s + oled_pcb[1] / 2 + oled_win_ds;
oled_k = wall + oled_glass_t;                 // front of the OLED PCB, in from the rear face
front_in = grip_d - wall;                     // inner face of the grip's front wall (u)
trig_open_bot = trig_bot + 1.9;               // bottom of the opening: room for the paddle's face to drop as it swings in
trig_inner_bot = trig_bot + 3.3;                // bottom of the paddle's inside part; the shoulder below the opening is the rest stop
trig_angle = atan(sw_pull / (sw_roll_v - trig_pivot[1]));
trig_travel = (trig_inner_bot - trig_pivot[1]) * tan(trig_angle); // swing at the bottom of the back face
assert(sw_pull > sw_click - sw_preload && sw_pull < sw_bottom - sw_preload, "sw_pull must pass the click and stop short of bottoming out");
sw_top = trig_back - (sw_roller[0] - sw_preload); // lever side of the switch body (u)
sw_far_v = sw_roll_v + sw_roller[1];          // switch's far end, up the grip (v)
sw_end_v = sw_far_v + sw[0];                  // terminal end (v)
function sw_hole(i) = [sw_top - sw_holes[i][1], sw_end_v - sw_holes[i][0]]; // [u, v]
pivot_xz = gp(trig_pivot[0], trig_pivot[1]);
mlx_x = head_l - 1 - mlx_can_h - car[2];      // back of the carrier; lens face 1 mm behind the front
// The Feather stands in the grip, between the battery (left) and the clock (right),
// components toward the clock, USB-C down through the butt.
usb_open = [12.5, 7];                         // opening in the butt for a plug overmold (across the grip, Y)
usb_yc = -usb_open[1] / 2;                    // opening runs from the split into the left half
fth_v1 = grip_l - wall - 1.8;                 // USB end of the board (v)
fth_v0 = fth_v1 - fth[0];
fth_u = (grip_d - fth[1]) / 2;                // rear long edge (u)
fth_yb = usb_yc - usb_c[2] / 2 - fth[2];      // underside of the PCB (y)
laser_tilt = atan((laser_z - sens_z) / aim_dist);
laser_block = [laser_l + 2, laser_d + 6];     // length, height
rtc_u = (grip_d - rtc[0]) / 2;                // clock board rear end (u)
rtc_y = Wi - 0.7 - rtc[2];                    // clock's inner face: 0.7 off the wall (foam tape), so its corners clear the grip's rounding
btn_tip_x = wall + btn_flange[1] + 0.3;       // stem tip, 0.3 behind the cap's flange
btn_body_x = btn_tip_x + btn_stem[1];         // front of the button body
btn_back_x = btn_body_x + btn[1];             // back of the body, against the backstop
btn_hold = [btn_back_x + clear + 2 - wall, btn_flange[0] / 2 + clear + 1.2]; // holder: depth (X), half-height (Z)

head_pts = [[rear_x, 0], [head_l, 0], [head_l, head_h], [rear_x + head_h * tan(rear_tilt), head_h]];

// Rear-face-local: children use x = depth in from the rear face, z = distance up it.
module in_rear() translate([rear_x, 0, 0]) rotate([0, rear_tilt, 0]) children();
function rp(k, s) = [rear_x + k * cos(rear_tilt) + s * sin(rear_tilt), -k * sin(rear_tilt) + s * cos(rear_tilt)];

// Grip-local (u across the grip rear to front, v down the grip) to model [x, z].
function gp(u, v) = [grip_x0 + u * cos(grip_angle) - v * sin(grip_angle),
                     -u * sin(grip_angle) - v * cos(grip_angle)];

boss_xz = [[49, 29.5], [19, 27], gp(10, -3), gp(8, grip_l - 9), gp(grip_d - 8, grip_l - 9)];

// Children use grip-local coordinates: x = u, z = -v.
module in_grip() translate([grip_x0, 0, 0]) rotate([0, grip_angle, 0]) children();

module at_xz(p) translate([p[0], 0, p[1]]) children();

module along_y(d, y0, y1) translate([0, y0, 0]) rotate([-90, 0, 0]) cylinder(d = d, h = y1 - y0);

module along_x(d, x0, x1) translate([x0, 0, 0]) rotate([0, 90, 0]) cylinder(d = d, h = x1 - x0);

// ---- outline ------------------------------------------------------------

// Side profile in (x, z).
module profile() {
    offset(r = corner_r) offset(delta = -corner_r)
    offset(r = -fillet_r) offset(delta = fillet_r)
    union() {
        polygon(head_pts);
        translate([grip_x0, 0]) rotate(-grip_angle) grip_2d();
    }
}

// Grip outline in grip-local [u, -v]: a palm swell on the backstrap, a flare at the butt.
module grip_2d() {
    sag = swell;
    len = swell_v[1] - swell_v[0];
    big_r = (len * len / 4 + sag * sag) / (2 * sag);
    translate([0, -grip_l]) square([grip_d, grip_l + head_h / 2]);
    intersection() {
        translate([big_r - sag, -(swell_v[0] + swell_v[1]) / 2]) circle(r = big_r, $fn = 720);
        translate([-sag - 1, -swell_v[1]]) square([sag + 1, len]);
    }
    polygon([[0, -(grip_l - flare_l)], [-flare[0], -grip_l], [0, -grip_l]]);
    polygon([[grip_d, -(grip_l - flare_l)], [grip_d + flare[1], -grip_l], [grip_d, -grip_l]]);
}

module slab(y0, y1) translate([0, y1, 0]) rotate([90, 0, 0]) linear_extrude(y1 - y0, convexity = 10) children();

// Edge profile: a round of edge_r, finished with 45° tangents at both outer
// faces so neither half needs support where its edges leave the bed.
module edge_kernel() rotate([90, 0, 0]) hull() {
    intersection() {
        sphere(r = edge_r, $fn = 32);
        cube([3 * edge_r, 3 * edge_r, sqrt(2) * edge_r], center = true);
    }
    for (s = [-1, 1]) translate([0, 0, s * (edge_r - 0.005)])
        cylinder(r = (sqrt(2) - 1) * edge_r, h = 0.01, center = true, $fn = 32);
}

// Square-edged head (chamfered), round-edged grip, split at the head's underside.
module body() {
    intersection() {
        minkowski() {
            slab(-W / 2 + chamfer, W / 2 - chamfer) offset(delta = -chamfer) profile();
            rotate([90, 0, 0]) for (s = [0, 1]) mirror([0, 0, s])
                cylinder(h = chamfer, r1 = chamfer, r2 = 0, $fn = 8);
        }
        translate([-500, -500, 0]) cube(1000);
    }
    intersection() {
        minkowski() {
            slab(-W / 2 + edge_r, W / 2 - edge_r) offset(delta = -edge_r) profile();
            edge_kernel();
        }
        translate([-500, -500, -1000]) cube(1000);
    }
}

// Square inside the head; in the grip (and the head's first 2 mm), rounded to follow the outside so the
// wall stays a full thickness along the rounded edges.
module cavity() {
    rc = edge_r - wall;
    slab(-Wi, Wi) intersection() { offset(r = -wall) profile(); translate([-500, 2]) square(1000); }
    intersection() {
        minkowski() {
            slab(-Wi + rc, Wi - rc) offset(delta = -rc) offset(r = -wall) profile();
            sphere(r = rc, $fn = 24);
        }
        translate([-500, -500, -998]) cube(1000);
    }
}

module rounded_rect(size, r) offset(r = r) square([size[0] - 2 * r, size[1] - 2 * r], center = true);

// Opening in the butt for the Feather's USB-C.
module usb_opening() in_grip() translate([grip_d / 2 + usb_y, usb_yc, -grip_l - 1])
    linear_extrude(wall + 2) rounded_rect(usb_open, 1.5);

// Side rails that hold a board's two edges, from the board's edge out to the
// side wall; pcb_slots() cuts the groove for the board into them.
// at = [x, z] of the board's lower rear corner, size = [length (X), width (Y), height (Z)].
module pcb_rails(at, size, grip = 1, pad = 1.5) for (s = [-1, 1])
    translate([at[0] - pad, s > 0 ? size[1] / 2 - grip : -Wi - 0.01, at[1] - pad])
        cube([size[0] + 2 * pad, Wi - size[1] / 2 + grip + 0.01, size[2] + 2 * pad]);

module pcb_slots(at, size, grip = 1) for (s = [-1, 1])
    translate([at[0] - clear, s > 0 ? size[1] / 2 - grip - 0.01 : -size[1] / 2 - clear, at[1] - clear])
        cube([size[0] + 2 * clear, grip + clear + 0.01, size[2] + 2 * clear]);

car_at = [mlx_x, sens_z - car[1] / 2];
car_size = [car[2], car[0], car[1]];

// Feather rails in the grip, below the battery: from the left wall to the board's two
// long edges, holding fth_edge of each; fth_slots() cuts the groove.
fth_rail_v = bat_v + bat[1] + 2 * clear + 1.2 + 0.5;
module fth_rails() in_grip() for (e = [0, 1])
    translate([e ? fth_u + fth[1] - fth_edge : fth_u - 2.5, -Wi - 0.01, -fth_v1 - 0.5])
        cube([2.5 + fth_edge, fth_yb + fth[2] + 1.2 + Wi + 0.01, fth_v1 + 0.5 - fth_rail_v]);
module fth_slots() in_grip() for (e = [0, 1])
    translate([e ? fth_u + fth[1] - fth_edge - 0.01 : fth_u - clear, fth_yb - clear, -fth_v1 - 0.5 - 0.01])
        cube([fth_edge + clear + 0.01, fth[2] + 2 * clear, fth_v1 + 0.5 - fth_rail_v + 0.02]);
// Stops at the board's top end, from the right half across the split, so a USB
// plug can't push the board up the grip.
module fth_stops() in_grip() for (u = [fth_u - 2, fth_u + fth[1] - 1]) // each catches 1 mm of the board's end
    translate([u, fth_yb - 0.5, -(fth_v0 - clear)]) cube([3, Wi - fth_yb + 0.5 + 0.01, 3]);

// ---- inside -------------------------------------------------------------

module features() {
    // Laser carrier.
    translate([head_l - wall - laser_block[0], -Wi - 0.01, laser_z - laser_block[1] / 2])
        cube([laser_block[0] + 0.1, 2 * Wi + 0.02, head_h]);
    // Screw bosses.
    for (p = boss_xz) at_xz(p) along_y(boss_d, -Wi - 0.01, Wi + 0.01);
    // Rails for the sensor carrier and the Feather.
    pcb_rails(car_at, car_size);
    fth_rails();
    // Hinge pads: room for the nut pocket and screw head, and side guides for the trigger.
    at_xz(pivot_xz) for (s = [-1, 1])
        along_y(9, s > 0 ? trig_t / 2 + 0.5 : -Wi - 0.01, s > 0 ? Wi + 0.01 : -trig_t / 2 - 0.5);
    // Buzzer ring (right half).
    at_xz(buzzer_xz) difference() {
        along_y(buzzer_d + 2 * clear + 3, Wi - 5, Wi + 0.01);
        along_y(buzzer_d + 2 * clear, Wi - 6, Wi + 1);
    }
    // Full-pull stop for the trigger's ear (right half).
    trig_stop();
    in_grip() {
        // Microswitch posts (right half), from the wall to the switch's outer face at
        // its two holes: a steel pin (an M3 from outside) through one, an M3 screw and nut through the other.
        for (i = [0, 1]) translate([sw_hole(i)[0], sw_y + sw[2], -sw_hole(i)[1]])
            rotate([-90, 0, 0]) cylinder(d = i == 0 ? 9 : 10, h = Wi - sw_y - sw[2] + 0.01);
        // Battery end stops (left half).
        for (v = [bat_v, bat_v + bat[1] + 2 * clear + 1.2])
            translate([wall - 0.01, -Wi - 0.01, -v]) cube([grip_d - 2 * wall + 0.02, 4, 1.2]);
        // Clock board stops along its long edges (right half), leaving the ends free for the STEMMA QT plugs.
        for (v = [rtc_v - 1.2, rtc_v + rtc[1] + 2 * clear], u = [rtc_u + 1, rtc_u + rtc[0] - 4])
            translate([u, rtc_y + 2, -v - 1.2]) cube([3, Wi - rtc_y - 2 + 0.01, 1.2]);
    }
}

// The locating pin's hole: an M3 screwed in from outside self-taps into the post, its
// head in a counterbore, its shank through the switch's hole near the terminals.
module sw_pin_hole() at_xz(gp(sw_hole(0)[0], sw_hole(0)[1])) {
    along_y(2.8, sw_y + sw[2] - 0.01, W / 2 + 1);
    along_y(screw_head_d, W / 2 - screw_head_h, W / 2 + 1);
    // Bridging layers over the counterbore.
    translate([-screw_head_d / 2, W / 2 - screw_head_h - 0.25 + 0.01, -1.4]) cube([screw_head_d, 0.25, 2.8]);
    translate([-1.4, W / 2 - screw_head_h - 0.5 + 0.01, -1.4]) cube([2.8, 0.5, 2.8]);
}

// Post on the right wall that the trigger's ear meets at full pull; its face
// is cut to the ear's angle there.
module trig_stop() difference() {
    in_grip() translate([trig_back - 8, trig_t / 2 + trig_pad + 0.7, -(trig_inner_bot + 1)])
        cube([8, Wi - trig_t / 2 - trig_pad - 0.7 + 0.01, trig_ear[1] + 2]);
    for (f = [0 : 0.1 : 1]) trigger_placed(f);
}

// Menu button holder, on the left half: a block on the rear wall, rising from the
// left wall past the split, with a slot open toward the right half. The button
// drops in from that side, legs up and down, against a backstop bar; the cap's
// flange sits in front of it.
module btn_holder() {
    intersection() { body(); in_rear() translate([wall - 0.01, -Wi - 0.01, btn_s - btn_hold[1]])
        cube([btn_hold[0] + 0.01, Wi + 0.02, 2 * btn_hold[1]]); }
    // Past the split it only touches the right half's walls.
    intersection() { cavity(); in_rear() translate([wall + 0.02, 0, btn_s - btn_hold[1]])
        cube([btn_hold[0] - 0.02, btn[0] / 2 + clear + 1.6, 2 * btn_hold[1]]); }
}

// Slots at the OLED's four corners: blocks on the side walls with a groove open toward
// the middle. The board slides sideways into the left half's two; the right half's two
// slide over its other corners as the halves close. No glue: open the case to take it out.
module oled_slots(side) intersection() { body(); in_rear() for (top = [0, 1]) {
    z0 = top ? oled_s + oled_pcb[1] - oled_hold[1] : oled_s - clear - 1.2;
    h = oled_hold[1] + clear + 1.2;
    y0 = side < 0 ? -Wi - 0.01 : oled_pcb[0] / 2 - oled_hold[0];
    difference() {
        translate([wall - 0.01, y0, z0]) cube([oled_k + oled_pcb[2] + clear + 1.5 - wall, oled_hold[0] + Wi - oled_pcb[0] / 2 + 0.01, h]);
        translate([oled_k - clear / 2, y0 - 1, top ? z0 - 0.01 : oled_s - clear]) cube([oled_pcb[2] + clear, oled_hold[0] + 3, oled_hold[1] + clear + 0.01]);
    }
} }

module btn_slot() {
    up = 20; // open toward +Y
    module slot(x0, x1, h, w) translate([x0, -w / 2, btn_s - h / 2]) cube([x1 - x0, w / 2 + up, h]);
    in_rear() {
    // Flange, with room for the cap's travel.
    translate([wall - 0.01, 0, btn_s]) hull() for (y = [0, up]) translate([0, y, 0])
        along_x(btn_flange[0] + 2 * clear, 0, btn_tip_x + 0.5 - wall + 0.01);
    // Stem.
    translate([btn_tip_x, 0, btn_s]) hull() for (y = [0, up]) translate([0, y, 0])
        along_x(btn_stem[0] + 1, 0, btn_body_x - btn_tip_x + 0.01);
    // Body, with room for the legs above and below it.
    slot(btn_body_x, btn_back_x + clear, btn[0] + 2, btn[0] + 2 * clear);
    // Legs past the backstop bar.
    for (s = [-1, 1]) translate([0, 0, s * 3.9])
        slot(btn_back_x, wall + btn_hold[0] + 1, 3, btn[0] + 2 * clear);
    }
}

module cutouts() {
    // Sensor aperture: open, no window. Glass and clear plastic block thermal IR.
    translate([0, 0, sens_z]) {
        along_x(mlx_can_d + 2 * clear + 0.2, head_l - wall - 1, head_l + 1);
        translate([head_l - 0.8, 0, 0]) rotate([0, 90, 0])
            cylinder(d1 = mlx_can_d + 0.8, d2 = mlx_can_d + 3, h = 1);
    }
    pcb_slots(car_at, car_size);
    fth_slots();
    // Laser bore, tilted so the dot lands on the sensor axis at aim_dist.
    translate([head_l, 0, laser_z]) rotate([0, laser_tilt, 0])
        along_x(laser_d + 0.6, -laser_block[0] - 1, 2); // printed holes come out small
    // Feather USB-C (charging, flashing, serial console), left open.
    usb_opening();
    in_rear() {
        // OLED: pocket, and a window bevelled in and out.
        translate([wall - 0.01, -Wi, oled_s - clear]) cube([oled_glass_t + oled_pcb[2] + oled_bot_h + 1, 2 * Wi, oled_pcb[1] + 2 * clear]);
        translate([wall + 0.01, 0, oled_win_s]) rotate([0, -90, 0]) hull() {
            linear_extrude(0.01) square([oled_win[1], oled_win[0]], center = true);
            translate([0, 0, wall + 0.02]) linear_extrude(0.01) square([oled_win[1] + 2 * wall, oled_win[0] + 2], center = true);
        }
        // Menu button cap bore.
        translate([0, 0, btn_s]) along_x(btn_cap_d + 2 * clear, -1, wall + 0.01);
    }
    // Buzzer sound holes (right half).
    at_xz(buzzer_xz) for (p = [[0, 0], [3, 0], [-3, 0], [0, 3], [0, -3]])
        translate([p[0], 0, p[1]]) along_y(1.6, Wi - 1, W / 2 + 1);
    // Trigger opening.
    in_grip() translate([front_in - 1, -trig_t / 2 - 0.5, -trig_open_bot])
        cube([wall + 2, trig_t + 1, trig_open_bot - trig_top]);
    // Switch hole in its post: the pin's twin, for the M3 screw.
    in_grip() translate([sw_hole(1)[0], 0, -sw_hole(1)[1]]) along_y(screw_d, sw_y + sw[2] - 0.01, W / 2 + 1);
}

// Places where the lip would hit a part or the right half's features.
module lip_keepouts() {
    translate([0, 0, sens_z]) along_x(mlx_can_d + 6, head_l - 10, head_l + 1);
    translate([mlx_x - 3, -6, -1]) cube([head_l - mlx_x + 4, 12, sens_z]);
    translate([head_l - laser_block[0] - 6, -5, laser_z - laser_block[1] / 2 - 2]) cube([laser_block[0] + 8, 10, 30]);
    in_rear() translate([-1, -Wi, btn_s - btn_hold[1] - 1])
        cube([wall + btn_hold[0] + 2, 2 * Wi, oled_s + oled_pcb[1] + 3 - (btn_s - btn_hold[1] - 1)]);
    in_grip() translate([fth_u - 3, usb_yc - usb_open[1] / 2 - 2, -grip_l - 1]) cube([fth[1] + 6, usb_open[1] + 4, wall + 5]);
    in_grip() translate([sw_top - sw[1] - 2, -Wi, -(sw_end_v + sw_term)])
        cube([grip_d - sw_top + sw[1] + 3, 2 * Wi, sw_end_v + sw_term - trig_top + 4]);
}

// The cavity's outline at the split, with the grip's rounding (an opening of
// edge_r - wall) applied everywhere, so the lip clears the rounded inside.
module inner_2d() offset(r = edge_r - wall) offset(delta = -(edge_r - wall)) offset(r = -wall) profile();

module lip() {
    difference() {
        union() {
            slab(-2, lip_h) difference() {
                offset(r = -0.25) inner_2d();
                offset(r = -(0.25 + lip_t)) inner_2d();
            }
            // Root of the lip, stepped back toward the wall so it prints without support.
            for (i = [0 : 4]) slab(-2 - 0.4 * (5 - i), i == 4 ? 0 : -2 - 0.4 * (4 - i)) difference() {
                offset(r = -wall + 0.1) profile();
                offset(r = -(0.25 + lip_t) * i / 4) inner_2d();
            }
        }
        lip_keepouts();
    }
}

module shell() {
    difference() {
        union() {
            difference() { body(); cavity(); }
            intersection() { features(); body(); }
        }
        cutouts();
    }
}

module half_space(s) translate([-300, s > 0 ? 0 : -100, -300]) cube([600, 100, 600]);

// Trigger, in grip-local (u, -v) at rest: a hub on the hinge, the paddle's inside
// part (its back face presses the switch lever; its front shoulder below the
// opening rests on the front wall), and the face that stands proud of the grip.
module trigger_2d() {
    difference() {
        union() {
            offset(r = 1) offset(delta = -1) union() {
                translate([trig_pivot[0], -trig_pivot[1]]) circle(r = 4);
                translate([trig_back, -trig_inner_bot]) square([front_in - 0.2 - trig_back, trig_inner_bot - trig_pivot[1]]);
            }
            hull() {
                translate([front_in - 1, -trig_bot]) square([1.1, trig_bot - trig_top - 0.5]);
                translate([grip_d + trig_proud[0] - 1.5, -trig_top - 2]) circle(r = 1.5);
                translate([grip_d + trig_proud[1] - 2, -trig_bot + 2]) circle(r = 2);
            }
        }
        translate([trig_pivot[0], -trig_pivot[1]]) circle(d = screw_d);
    }
}

// The ear: the bottom of the paddle's inside part, widened on the right inside the grip.
// It stops short of the front wall, clear of the grip's rounded inside edge.
module trig_ear_2d() translate([trig_back, -trig_inner_bot]) square([front_in - 2.5 - trig_back, trig_ear[1]]);
// The pad: the paddle's inside part, from below the hinge guides down to the bottom, behind the front wall.
module trig_pad_2d() translate([trig_back, -trig_inner_bot]) square([front_in - 2.5 - trig_back, trig_inner_bot - (sw_roll_v - 7)]);

module trigger_3d() {
    slab(-trig_t / 2, trig_t / 2) trigger_2d();
    slab(trig_t / 2 - 0.01, trig_t / 2 + trig_pad) trig_pad_2d();
    slab(trig_t / 2 - 0.01, trig_t / 2 + trig_pad + trig_ear[0]) trig_ear_2d();
}

module trigger_placed(f = pull) in_grip()
    translate([trig_pivot[0], 0, -trig_pivot[1]]) rotate([0, f * trig_angle, 0])
        translate([-trig_pivot[0], 0, trig_pivot[1]]) trigger_3d();

// Menu button cap; prints flange down.
module btn_cap() {
    cylinder(d = btn_flange[0], h = btn_flange[1]);
    cylinder(d = btn_cap_d, h = btn_flange[1] + wall + btn_cap_proud - 0.6);
    translate([0, 0, btn_flange[1] + wall + btn_cap_proud - 0.6])
        cylinder(d1 = btn_cap_d, d2 = btn_cap_d - 1.2, h = 0.6);
}

module btn_cap_in_place() color("dimgray") in_rear()
    translate([wall + btn_flange[1], 0, btn_s]) rotate([0, -90, 0]) btn_cap();

module left() {
    difference() {
        union() { intersection() { shell(); half_space(-1); } lip(); btn_holder(); oled_slots(-1); }
        btn_slot();
        for (p = concat(boss_xz, [pivot_xz])) at_xz(p) {
            along_y(screw_d, -W / 2 - 1, 0.1);
            along_y(screw_head_d, -W / 2 - 1, -W / 2 + screw_head_h);
            // Two bridging layers over the counterbore so it prints without support.
            translate([-screw_head_d / 2, -W / 2 + screw_head_h - 0.01, -screw_d / 2])
                cube([screw_head_d, 0.25, screw_d]);
            translate([-screw_d / 2, -W / 2 + screw_head_h - 0.01, -screw_d / 2])
                cube([screw_d, 0.5, screw_d]);
        }
    }
}

// Hex pocket for an M3 nut, `depth` in from the right half's outer face.
module nut_pocket(depth = 2.6) {
    translate([0, W / 2 - depth, 0]) rotate([-90, 0, 0]) rotate(30) cylinder(d = nut_af / cos(30) + 0.3, h = depth + 0.4, $fn = 6);
    // Bridging layers over the nut pocket, as for the counterbores.
    translate([-screw_d / 2, W / 2 - depth - 0.25 + 0.01, -(nut_af + 0.3) / 2]) cube([screw_d, 0.25, nut_af + 0.3]);
    translate([-screw_d / 2, W / 2 - depth - 0.5 + 0.01, -screw_d / 2]) cube([screw_d, 0.5, screw_d]);
}

module right() {
    difference() {
        union() { intersection() { shell(); half_space(1); } fth_stops(); oled_slots(1); }
        for (p = boss_xz) at_xz(p) along_y(insert_d, -0.1, insert_depth);
        // Hinge screw and the switch screw through the right wall, nuts recessed in its outer face.
        at_xz(pivot_xz) { along_y(screw_d, 0, W / 2 + 1); nut_pocket(); }
        // The switch's nut sits deeper, where the screw's tip ends.
        at_xz(gp(sw_hole(1)[0], sw_hole(1)[1])) nut_pocket(W / 2 - (sw_y + sw_screw_l - 2.6));
        sw_pin_hole();
    }
}

// ---- stand-in parts (for the assembly view and the fit check) -----------

module parts() {
    // MLX90640 on its carrier, with room behind for the capacitors and cable.
    color("royalblue") {
        translate([mlx_x, -car[0] / 2, sens_z - car[1] / 2]) cube([car[2], car[0], car[1]]);
        translate([0, 0, sens_z]) along_x(mlx_can_d, mlx_x + car[2], mlx_x + car[2] + mlx_can_h);
        translate([mlx_x - car_back, -car[0] / 2 + 2, sens_z - car[1] / 2 + 2]) cube([car_back, car[0] - 4, car[1] - 4]);
    }
    // FeatherS3[D] in the grip: PCB, parts on both sides, USB-C receptacle at the butt end.
    color("seagreen") in_grip() {
        translate([fth_u, fth_yb, -fth_v1]) cube([fth[1], fth[2], fth[0]]);
        zl = fth_zones[0][0] + fth_zones[1][0];
        translate([fth_u + 2.5, fth_yb + fth[2], -fth_v1 + 2]) cube([fth[1] - 5, fth_top_h, fth[0] - 2 - zl]);
        for (i = [0, 1]) let (v0 = i == 0 ? 0 : fth_zones[0][0])
            translate([fth_u + 2.5, fth_yb + fth[2], -fth_v0 - v0 - fth_zones[i][0]]) cube([fth[1] - 5, fth_zones[i][1], fth_zones[i][0]]);
        translate([fth_u + 2.5, fth_yb - fth_bot_h, -fth_v1 + 2]) cube([fth[1] - 5, fth_bot_h, fth[0] - 4]);
        translate([grip_d / 2 + usb_y - usb_c[0] / 2, fth_yb + fth[2], -fth_v1 - 0.6]) cube([usb_c[0], usb_c[2], usb_c[1]]);
    }
    // Laser module.
    color("crimson") translate([head_l, 0, laser_z]) rotate([0, laser_tilt, 0])
        along_x(laser_d, -laser_l - 0.8, -0.8);
    // Buzzer.
    color("dimgray") at_xz(buzzer_xz) along_y(buzzer_d, Wi - buzzer_h, Wi);
    // OLED module on the rear face: glass, PCB, sockets behind.
    color("black") in_rear() {
        translate([oled_k, -oled_pcb[0] / 2, oled_s]) cube([oled_pcb[2], oled_pcb[0], oled_pcb[1]]);
        translate([wall, -oled_glass[0] / 2, oled_s + oled_pcb[1] - oled_glass_top - oled_glass[1]]) cube([oled_glass_t, oled_glass[0], oled_glass[1]]);
        translate([oled_k + oled_pcb[2], -oled_pcb[0] / 2, oled_s + 2]) cube([oled_bot_h, oled_pcb[0], oled_pcb[1] - 4]);
    }
    in_grip() {
        // Microswitch body, upside down (terminals at the bottom), and room for its terminals and wires.
        color("orange") difference() {
            translate([sw_top - sw[1], sw_y, -sw_end_v]) cube([sw[1], sw[2], sw[0]]);
            for (i = [0, 1]) translate([sw_hole(i)[0], 0, -sw_hole(i)[1]]) along_y(sw_hole_d, sw_y - 1, sw_y + sw[2] + 1);
        }
        color("silver") translate([sw_top - sw[1] + 2, sw_y + 1, -sw_end_v - sw_term]) cube([sw[1] - 4, sw[2] - 2, sw_term]);
        color("silver") translate([sw_top - sw[1] - sw_com[0], sw_y + 1, -(sw_end_v - sw_com[2])]) cube([sw_com[0], sw[2] - 2, sw_com[1] - sw_com[2]]);
        // Its two screws: the clamp (head on the switch's inner face, nut in the post) and the steel pin (from outside).
        color("dimgray") {
            translate([sw_hole(1)[0], 0, -sw_hole(1)[1]]) { along_y(5.5, sw_y - 3, sw_y); along_y(2.9, sw_y, sw_y + sw_screw_l); }
            translate([sw_hole(0)[0], 0, -sw_hole(0)[1]]) { along_y(5.5, W / 2 - screw_head_h, W / 2 - 0.2); along_y(2.8, W / 2 - screw_head_h - sw_pin_l, W / 2 - screw_head_h); }
        }
        // Clock board across the grip, near the right wall, with its STEMMA QT plugs at both ends.
        color("teal") {
            translate([rtc_u, rtc_y, -rtc_v - clear - rtc[1]]) cube([rtc[0], rtc[2], rtc[1]]);
            for (u = [rtc_u - rtc_plug, rtc_u + rtc[0]])
                translate([u, rtc_y, -(rtc_v + clear + rtc[1] / 2 + 4)]) cube([rtc_plug, 5, 8]);
        }
        // Battery, against the left wall.
        color("gold") translate([(grip_d - bat[0]) / 2, -Wi + 0.45, -bat_v - clear - bat[1]])
            cube([bat[0], bat[2], bat[1]]);
    }
    // Menu button: body, stem, and its four legs bent straight back.
    color("white") in_rear() {
        translate([btn_body_x, -btn[0] / 2, btn_s - btn[0] / 2]) cube([btn[1], btn[0], btn[0]]);
        translate([0, 0, btn_s]) along_x(btn_stem[0], btn_tip_x, btn_body_x);
        for (s = [-1, 1], t = [-1, 1]) translate([btn_body_x + 1, t * 2.25 - 0.35, btn_s + s * 3.25 - 0.35])
            cube([btn[1] + 2.5, 0.7, 0.7]);
    }
}

// The switch's lever and roller, with the roller pushed in by the trigger at
// pull f (f = 0 includes the preload). A straight arm from the hinge to the roller.
module sw_lever(f = pull) color("silver") in_grip() {
    push = sw_preload + f * sw_pull * 1.03; // a little extra: the paddle's face tilts as it swings
    roll = [sw_top + sw_roller[0] - sw_roller[2] / 2 - push, sw_roll_v];
    hinge = [sw_top, sw_end_v - sw_hinge];
    translate([0, sw_y + sw[2] / 2, 0]) {
        hull() for (p = [hinge, roll + [-sw_roller[2] / 2 + 0.6, 0]])
            translate([p[0], -2, -p[1]]) cube([0.6, 4, 0.6]);
        translate([roll[0], 0, -roll[1]]) along_y(sw_roller[2], -sw_roller_w / 2, sw_roller_w / 2);
    }
}

// ---- output -------------------------------------------------------------

// Halves come out lying on their flat outer faces, open side up.
if (part == "left") translate([0, 0, W / 2]) rotate([90, 0, 0]) left();
else if (part == "right") translate([0, 0, W / 2]) rotate([-90, 0, 0]) right();
else if (part == "trigger") translate([0, 0, trig_t / 2]) rotate([90, 0, 0]) trigger_3d(); // on its side, ear up
else if (part == "btn_cap") btn_cap();
// Test print: the grip's top section of the right half (switch posts, trigger stop,
// hinge), to try with the trigger and the switch.
else if (part == "test_trigger") translate([0, 0, W / 2]) rotate([-90, 0, 0])
    intersection() { right(); in_grip() translate([-10, 0, -52]) cube([70, 30, 64]); }
else if (part == "cutaway") {
    color("lightgray") render() left();
    parts();
    sw_lever();
    color("orange") trigger_placed();
    btn_cap_in_place();
}
else if (part == "check") {
    // Renders empty when every part clears the shell and the halves don't overlap.
    intersection() { union() { left(); right(); } parts(); }
    intersection() { left(); fth_stops(); }
    for (f = [0, 1]) intersection() { union() { left(); right(); parts(); sw_lever(f); } trigger_placed(f); }
    intersection() { union() { left(); right(); parts(); } sw_lever(0); }
    intersection() { left(); right(); }
    intersection() { union() { left(); right(); parts(); } btn_cap_in_place(); }
}
else {
    // Closed and opaque; see "cutaway" for the inside. (Transparent shells
    // draw as a checkerboard in OpenSCAD's F5 preview.)
    color("lightgray") render() left();
    color("silver") render() right();
    color("dimgray") trigger_placed();
    btn_cap_in_place();
    sw_lever();
}
