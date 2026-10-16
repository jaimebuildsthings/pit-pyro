// Handheld tire temperature scanner: pistol-grip enclosure, v0.2 (phase 2 prototype)
//
// Units: mm. Model frame: +X points at the tire, +Z is up, Y runs across the
// grip. The shell splits at Y = 0 into a left and a right half. Each half
// prints on its flat outer face, open side up, with no supports.
//
// Export:  openscad --export-format binstl -D 'part="left"'      -o stl/enclosure-left.stl      enclosure.scad
//          openscad --export-format binstl -D 'part="right"'     -o stl/enclosure-right.stl     enclosure.scad
//          openscad --export-format binstl -D 'part="usb_cover"' -o stl/usb-cover.stl           enclosure.scad
//          openscad --export-format binstl -D 'part="trigger"'   -o stl/trigger.stl             enclosure.scad
// Parts:   assembly (closed shell), cutaway (left half + stand-in parts),
//          left, right, usb_cover, trigger (each laid flat, ready to slice), check
//          (renders nothing but contact faces when every part fits).
//
// Licensed under CERN-OHL-P-2.0, see ../LICENSE.

part = "assembly"; // [assembly, cutaway, left, right, usb_cover, trigger, check]
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
// carrier width (Y), height (Z), thickness; 8 x 8 holes. Keep the outer row of holes empty.
car = [20.3, 20.3, 1.6];
car_back = 10;        // room behind the carrier for the capacitors, joints and cable
mlx_can_d = 9.2;
mlx_can_h = 5.0;      // lens face above the carrier
// Unexpected Maker FeatherS3[D]: length (X), width (Y), thickness
fth = [52.3, 22.9, 1.6];
fth_top_h = 5;        // tallest part above the PCB (battery JST, 6.6 mm overall)
fth_bot_h = 1.5;      // parts and trimmed leads under the PCB
fth_edge = 0.8;       // how much of each long edge the side rails hold
usb_y = 0;            // centre of the USB-C port, across the board
// Adafruit 326 0.96" OLED (STEMMA QT): width (Y, along the header), depth (X), PCB thickness
oled_pcb = [29.2, 26.7, 1.6];
oled_glass_t = 1.6;   // glass + tape above the PCB
oled_bot_h = 3;       // STEMMA QT sockets under the PCB
oled_win = [24, 13];  // visible window (Y, X), active area 21.7 x 10.9 plus margin
oled_win_dx = -2;     // window centre offset from the module centre, toward the front (the active area sits toward the header)
// Adafruit 5188 DS3231 clock: length (along the grip), width, thickness incl. coin cell and sockets
rtc = [25.4, 17.8, 8];
rtc_plug = 6;         // room past each end for a STEMMA QT plug
// Adafruit 258 LiPo, 1200 mAh: width, length, thickness incl. swelling
bat = [34, 62, 5.5];
usb_c = [9.0, 7.3, 3.2];      // receptacle width, depth, height
laser_d = 6;                  // Adafruit 1054 laser tube
laser_l = 14;
buzzer_d = 12.2;              // Adafruit 160 piezo (PS1240)
buzzer_h = 7;
// Lever microswitch (Adafruit 819, KW11-3Z roller): length, height, thickness
sw = [19.8, 10.2, 6.4];
sw_hole_d = 2.3;
sw_hole_pitch = 9.5;
sw_hole_h = 2.9;              // hole centres from the terminal edge
sw_lever = 5;                 // trigger-to-body gap at rest; the roller lever is preloaded inside it
// M3 screws into heat-set inserts
screw_d = 3.4;
screw_head_d = 6.2;
screw_head_h = 3.0;
insert_d = 4.0;
insert_depth = 8;
boss_d = 7.5;

/* [Layout] */
head_l = 82;
head_h = 40;
grip_x0 = 4;          // rear edge of the grip where it meets the head
grip_d = 47;          // grip front-to-back (deep enough for edge_r around the battery)
grip_l = 88;          // grip length below the head: four fingers of a medium to large hand
grip_angle = 16;      // rake, butt toward the rear
swell = 3;            // palm swell: how far the backstrap bulges...
swell_v = [12, 80];   // ...between these points down the grip
flare = [3, 2];       // butt flare, rear and front
flare_l = 16;         // flare length up the grip
sens_z = 14;          // sensor axis height
laser_z = 31;         // aiming dot height at the front face
aim_dist = 250;       // laser crosses the sensor axis this far out
fth_z = 12;           // underside of the Feather PCB
// Trigger: a wide paddle filling an opening at the top of the grip's front,
// hinged at its top inside the grip (like a handheld IR thermometer's).
trig_pivot = [grip_d - 7, -7]; // hinge, grip-local [u, v], just under the head
trig_top = -8;        // top of the opening (v)
trig_bot = 16.8;      // bottom of the paddle's face (v)
trig_back = grip_d - 9; // back face of the paddle, inside the grip (u); it presses the switch
trig_proud = [3, 7];  // how far the face stands proud of the grip, top and bottom
trig_t = 18;          // paddle width (Y), within the grip's flat front; clears the battery
trig_travel = 2.5;    // swing at the bottom of the back face, up to the stop
nut_af = 5.5;         // M3 nut across flats, for the hinge screw
buzzer_xz = [12, 27];
oled_x = 28.5;        // rear edge of the OLED module (header end)
rtc_v = 30;           // clock board top edge, along the grip
bat_v = 2.5;          // battery top edge, along the grip; it sits beside the switch
usb_cover_fit = 0.15; // per side; raise for a looser cover

$fa = 4;
$fs = 0.4;

// ---- derived ------------------------------------------------------------

W = oled_pcb[0] + 2 * clear + 2 * wall;       // overall width, set by the OLED
Wi = W / 2 - wall;                            // inner half-width
oled_z = head_h - wall - oled_glass_t - oled_pcb[2]; // underside of the OLED PCB
oled_win_x = oled_x + oled_pcb[1] / 2 + oled_win_dx;
fth_port_z = fth_z + fth[2] + usb_c[2] / 2;
front_in = grip_d - wall;                     // inner face of the grip's front wall (u)
trig_inner_bot = trig_bot + 2.3;              // bottom of the paddle's inside part; the shoulder below the opening is the rest stop
trig_angle = atan(trig_travel / (trig_inner_bot - trig_pivot[1]));
sw_top = trig_back - sw_lever;                // lever side of the switch body (u)
stop_u = trig_back - trig_travel;             // full-pull stop face (u)
sw_v = 1.5;                                   // switch body top edge (v)
pivot_xz = gp(trig_pivot[0], trig_pivot[1]);
mlx_x = head_l - 1 - mlx_can_h - car[2];      // back of the carrier; lens face 1 mm behind the front
fth_x = wall + 1.8;                           // USB end of the Feather
laser_tilt = atan((laser_z - sens_z) / aim_dist);
laser_block = [laser_l + 2, 12];              // length, height
rtc_u = (grip_d - rtc[1]) / 2;                // clock board rear edge (u)

// Grip-local (u across the grip rear to front, v down the grip) to model [x, z].
function gp(u, v) = [grip_x0 + u * cos(grip_angle) - v * sin(grip_angle),
                     -u * sin(grip_angle) - v * cos(grip_angle)];

boss_xz = [[58.5, 7.8], [59.5, 32], [24, 31], gp(10, -3), gp(grip_d / 2, grip_l - 13)];

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
        square([head_l, head_h]);
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

// Opening in the rear wall for the Feather's USB-C, sized for a plug overmold (Y, Z).
usb_open = [12.5, 7];
module usb_opening(grow = 0, depth = wall + 2) translate([-1, usb_y, fth_port_z]) rotate([0, 90, 0])
    linear_extrude(depth) rotate(90) rounded_rect(usb_open + [2 * grow, 2 * grow], 1.5 + grow);

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
fth_at = [fth_x, fth_z];

// ---- inside -------------------------------------------------------------

module features() {
    // Laser carrier.
    translate([head_l - wall - laser_block[0], -Wi - 0.01, laser_z - laser_block[1] / 2])
        cube([laser_block[0] + 0.1, 2 * Wi + 0.02, head_h]);
    // Screw bosses.
    for (p = boss_xz) at_xz(p) along_y(boss_d, -Wi - 0.01, Wi + 0.01);
    // Rails for the sensor carrier and the Feather.
    pcb_rails(car_at, car_size);
    pcb_rails(fth_at, fth, fth_edge);
    // Hinge pads: room for the nut pocket and screw head, and side guides for the trigger.
    at_xz(pivot_xz) for (s = [-1, 1])
        along_y(9, s > 0 ? trig_t / 2 + 0.5 : -Wi - 0.01, s > 0 ? Wi + 0.01 : -trig_t / 2 - 0.5);
    // Buzzer ring (right half).
    at_xz(buzzer_xz) difference() {
        along_y(buzzer_d + 2 * clear + 3, Wi - 5, Wi + 0.01);
        along_y(buzzer_d + 2 * clear, Wi - 6, Wi + 1);
    }
    in_grip() {
        // Microswitch cradle (right half; the battery takes the left). Its front
        // face is the trigger's full-pull stop; sw_pins() hold the switch.
        translate([sw_top - sw[1] - 1, sw[2] / 2 + 0.2, -(sw_v + sw[0] + 1.5)])
            cube([stop_u - sw_top + sw[1] + 1, Wi - sw[2] / 2 - 0.2 + 0.01, sw[0] + 0.3]);
        // Battery end stops (left half).
        for (v = [bat_v, bat_v + bat[1] + 2 * clear + 1.2])
            translate([wall - 0.01, -Wi - 0.01, -v]) cube([grip_d - 2 * wall + 0.02, 4, 1.2]);
        // Clock board corner stops (right half), leaving the ends free for the STEMMA QT plugs.
        for (v = [rtc_v - 1.2, rtc_v + rtc[0] + 2 * clear], u = [rtc_u - 2, rtc_u + rtc[1] - 1])
            translate([u, Wi - 4, -v - 1.2]) cube([3, 4.01, 1.2]);
    }
}

// Pins through the switch's two holes, from the right half's cradle across the split.
module sw_pins() in_grip() for (dv = [0, sw_hole_pitch])
    translate([sw_top - sw[1] + sw_hole_h, 0, -(sw_v + (sw[0] - sw_hole_pitch) / 2 + dv)])
        along_y(sw_hole_d - 0.3, -sw[2] / 2 - 0.3, sw[2] / 2 + 0.3);

module cutouts() {
    // Sensor aperture: open, no window. Glass and clear plastic block thermal IR.
    translate([0, 0, sens_z]) {
        along_x(mlx_can_d + 2 * clear + 0.2, head_l - wall - 1, head_l + 1);
        translate([head_l - 0.8, 0, 0]) rotate([0, 90, 0])
            cylinder(d1 = mlx_can_d + 0.8, d2 = mlx_can_d + 3, h = 1);
    }
    pcb_slots(car_at, car_size);
    pcb_slots(fth_at, fth, fth_edge);
    // Laser bore, tilted so the dot lands on the sensor axis at aim_dist.
    translate([head_l, 0, laser_z]) rotate([0, laser_tilt, 0])
        along_x(laser_d + 0.3, -laser_block[0] - 1, 2);
    // Feather USB-C (charging, flashing, serial console), closed by the usb_cover part.
    usb_opening();
    // OLED: pocket, PCB slots in the side walls, and a window bevelled fore and aft.
    translate([oled_x - clear, -W / 2 + wall, oled_z - 1]) cube([oled_pcb[1] + 2 * clear, W - 2 * wall, head_h - wall - oled_z + 1]);
    for (s = [-1, 1]) translate([oled_x - clear, s > 0 ? W / 2 - wall - 0.01 : -W / 2 + wall - 0.9, oled_z - 0.2])
        cube([oled_pcb[1] + 2 * clear, 0.91, oled_pcb[2] + 0.4]);
    translate([oled_win_x, 0, head_h - wall - 0.01]) hull() {
        linear_extrude(0.01) square([oled_win[1], oled_win[0]], center = true);
        translate([0, 0, wall + 0.02]) linear_extrude(0.01) square([oled_win[1] + 2 * wall, oled_win[0] + 2], center = true);
    }
    // Buzzer sound holes (right half).
    at_xz(buzzer_xz) for (p = [[0, 0], [3, 0], [-3, 0], [0, 3], [0, -3]])
        translate([p[0], 0, p[1]]) along_y(1.6, Wi - 1, W / 2 + 1);
    // Trigger opening.
    in_grip() translate([front_in - 1, -trig_t / 2 - 0.5, -trig_bot - 1])
        cube([wall + 2, trig_t + 1, trig_bot + 1 - trig_top]);
}

// Places where the lip would hit a part or the right half's features.
module lip_keepouts() {
    translate([0, 0, sens_z]) along_x(mlx_can_d + 6, head_l - 10, head_l + 1);
    translate([mlx_x - 3, -6, -1]) cube([head_l - mlx_x + 4, 12, sens_z]);
    translate([head_l - laser_block[0] - 6, -5, laser_z - laser_block[1] / 2 - 2]) cube([laser_block[0] + 8, 10, 30]);
    translate([-1, -13, fth_z - 4]) cube([wall + 4, 26, 18]);
    translate([oled_x - 2, -5, oled_z - 4]) cube([oled_pcb[1] + 4, 10, 20]);
    in_grip() translate([sw_top - sw[1] - 2, -Wi, -(sw_v + sw[0] + 2)])
        cube([grip_d - sw_top + sw[1] + 3, 2 * Wi, sw_v + sw[0] + 2 - trig_top + 4]);
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

module trigger_placed(f = pull) in_grip()
    translate([trig_pivot[0], 0, -trig_pivot[1]]) rotate([0, f * trig_angle, 0])
        translate([-trig_pivot[0], 0, trig_pivot[1]]) slab(-trig_t / 2, trig_t / 2) trigger_2d();

// Press-fit cover for the USB opening; prints flange down. Pull it by the tab.
module usb_cover() {
    flange = usb_open + [5, 5];
    linear_extrude(1.2) rounded_rect(flange, 3);
    linear_extrude(1.2 + wall) rounded_rect(usb_open - 2 * [usb_cover_fit, usb_cover_fit], 1.5 - usb_cover_fit);
    translate([flange[0] / 2 - 1, -3, 0]) cube([5, 6, 1.2]);
}

module left() {
    difference() {
        union() { intersection() { shell(); half_space(-1); } lip(); }
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

module right() {
    difference() {
        union() { intersection() { shell(); half_space(1); } sw_pins(); }
        for (p = boss_xz) at_xz(p) along_y(insert_d, -0.1, insert_depth);
        // Hinge screw through the right wall, nut recessed in its outer face.
        at_xz(pivot_xz) {
            along_y(screw_d, 0, W / 2 + 1);
            translate([0, W / 2 - 2.6, 0]) rotate([-90, 0, 0]) rotate(30) cylinder(d = nut_af / cos(30) + 0.3, h = 3, $fn = 6);
            // Bridging layers over the nut pocket, as for the counterbores.
            translate([-screw_d / 2, W / 2 - 2.6 - 0.25 + 0.01, -(nut_af + 0.3) / 2]) cube([screw_d, 0.25, nut_af + 0.3]);
            translate([-screw_d / 2, W / 2 - 2.6 - 0.5 + 0.01, -screw_d / 2]) cube([screw_d, 0.5, screw_d]);
        }
    }
}

// ---- stand-in parts (for the assembly view and the fit check) -----------

module usb_cover_in_place() color("dimgray")
    translate([-1.2, usb_y, fth_port_z]) rotate([0, 90, 0]) rotate(90) usb_cover();

module parts() {
    // MLX90640 on its carrier, with room behind for the capacitors and cable.
    color("royalblue") {
        translate([mlx_x, -car[0] / 2, sens_z - car[1] / 2]) cube([car[2], car[0], car[1]]);
        translate([0, 0, sens_z]) along_x(mlx_can_d, mlx_x + car[2], mlx_x + car[2] + mlx_can_h);
        translate([mlx_x - car_back, -car[0] / 2 + 2, sens_z - car[1] / 2 + 2]) cube([car_back, car[0] - 4, car[1] - 4]);
    }
    // FeatherS3[D]: PCB, parts above and below, USB-C receptacle.
    color("seagreen") {
        translate([fth_x, -fth[1] / 2, fth_z]) cube(fth);
        translate([fth_x + 2, -fth[1] / 2 + 2.5, fth_z + fth[2]]) cube([fth[0] - 4, fth[1] - 5, fth_top_h]);
        translate([fth_x + 2, -fth[1] / 2 + 2.5, fth_z - fth_bot_h]) cube([fth[0] - 4, fth[1] - 5, fth_bot_h]);
        translate([fth_x - 0.6, usb_y - usb_c[0] / 2, fth_z + fth[2]]) cube([usb_c[1], usb_c[0], usb_c[2]]);
    }
    // Laser module.
    color("crimson") translate([head_l, 0, laser_z]) rotate([0, laser_tilt, 0])
        along_x(laser_d, -laser_l - 0.8, -0.8);
    // Buzzer.
    color("dimgray") at_xz(buzzer_xz) along_y(buzzer_d, Wi - buzzer_h, Wi);
    // OLED module: PCB, glass, sockets underneath.
    color("black") {
        translate([oled_x, -oled_pcb[0] / 2, oled_z]) cube([oled_pcb[1], oled_pcb[0], oled_pcb[2]]);
        translate([oled_x + oled_pcb[1] / 2 - 9.5, -26.6 / 2, oled_z + oled_pcb[2]]) cube([19, 26.6, oled_glass_t]);
        translate([oled_x + 2, -oled_pcb[0] / 2, oled_z - oled_bot_h]) cube([oled_pcb[1] - 4, oled_pcb[0], oled_bot_h]);
    }
    in_grip() {
        // Microswitch body (its lever sits in the gap to the trigger).
        color("orange") difference() {
            translate([sw_top - sw[1], -sw[2] / 2, -sw_v - sw[0]]) cube([sw[1], sw[2], sw[0]]);
            for (dv = [0, sw_hole_pitch]) translate([sw_top - sw[1] + sw_hole_h, 0, -(sw_v + (sw[0] - sw_hole_pitch) / 2 + dv)])
                along_y(sw_hole_d, -sw[2], sw[2]);
        }
        // Clock board against the right wall, with its STEMMA QT plugs at both ends.
        color("teal") {
            translate([rtc_u, Wi - 0.1 - rtc[2], -rtc_v - clear - rtc[0]]) cube([rtc[1], rtc[2], rtc[0]]);
            for (v = [rtc_v + clear - rtc_plug, rtc_v + clear + rtc[0]])
                translate([grip_d / 2 - 4, Wi - 0.1 - rtc[2], -v - rtc_plug]) cube([8, 5, rtc_plug]);
        }
        // Battery, against the left wall.
        color("gold") translate([(grip_d - bat[0]) / 2, -Wi + 0.3, -bat_v - clear - bat[1]])
            cube([bat[0], bat[2], bat[1]]);
    }
}

// ---- output -------------------------------------------------------------

// Halves come out lying on their flat outer faces, open side up.
if (part == "left") translate([0, 0, W / 2]) rotate([90, 0, 0]) left();
else if (part == "right") translate([0, 0, W / 2]) rotate([-90, 0, 0]) right();
else if (part == "usb_cover") usb_cover();
else if (part == "trigger") linear_extrude(trig_t) trigger_2d();
else if (part == "cutaway") {
    color("lightgray") render() left();
    parts();
    color("orange") trigger_placed();
    usb_cover_in_place();
}
else if (part == "check") {
    // Renders empty when every part clears the shell and the halves don't overlap.
    intersection() { union() { left(); right(); } parts(); }
    intersection() { left(); sw_pins(); }
    for (f = [0, 1]) intersection() { union() { left(); right(); parts(); } trigger_placed(f); }
    intersection() { left(); right(); }
}
else {
    // Closed and opaque; see "cutaway" for the inside. (Transparent shells
    // draw as a checkerboard in OpenSCAD's F5 preview.)
    color("lightgray") render() left();
    color("silver") render() right();
    color("dimgray") trigger_placed();
    usb_cover_in_place();
}
