"""Builds the MLX90640 sensor carrier PCB (sensor-carrier.kicad_pcb).

Run with KiCad's own Python, which has the `pcbnew` module:

    /Applications/KiCad/KiCad.app/Contents/Frameworks/Python.framework/Versions/Current/bin/python3 generate.py

The board is generated rather than drawn so every position below can be
reviewed and changed in one place. Coordinates are mm from the board's centre,
seen from the front (the sensor's side), x to the right and y down. "Up" (y < 0)
is the side the sensor's tab points to.
"""

import math
import os

import pcbnew

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "sensor-carrier.kicad_pcb")
LIB = "/Applications/KiCad/KiCad.app/Contents/SharedSupport/footprints/"

# Board: 20 x 20 x 1.6 mm, the enclosure's carrier size (`car` in enclosure.scad).
# The head's rails grip the left and right edges by 1 mm, so copper and parts
# stay at least 1.5 mm in from those two edges.
SIZE = 20.0
CORNER = 0.8
CAN_KEEPOUT = 5.2  # the can is Ø9.4 and sits flush: no front copper or pads inside this radius

# MLX90640 (TO39): legs on a Ø5.84 mm circle (datasheet Figure 27), not in a
# square: two pairs 45° apart, facing each other. The tab sits between SDA and
# SCL (Figure 2, a bottom view). Seen from the front with the tab up, each pin
# is 22.5° either side of straight up or straight down.
PIN_R = 5.84 / 2


def on_circle(degrees_from_up):
    a = math.radians(degrees_from_up)
    return (round(PIN_R * math.sin(a), 4), round(-PIN_R * math.cos(a), 4))


SENSOR_PINS = {
    "1": ("SDA", *on_circle(-22.5)),   # top, left of the tab
    "4": ("SCL", *on_circle(22.5)),    # top, right of the tab
    "2": ("VDD", *on_circle(180 + 22.5)),  # bottom left
    "3": ("GND", *on_circle(180 - 22.5)),  # bottom right
}

# Cable holes along the top edge: the signals in the middle, straight above
# their pins, and power on the outside.
J1_Y = -8.2
J1_X = [-3.81, -1.27, 1.27, 3.81]
J1_NETS = ["VDD", "SDA", "SCL", "GND"]

# Pull-ups on the left and right, standing between the sensor's signal and VDD.
R_X = 7.3
R_TOP, R_BOTTOM = -3.81, 3.81

# Supply capacitors along the bottom edge.
C1_AT = (-2.5, 7.8)   # 100 nF, pads 2.5 mm apart
C2_AT = (2.6, 7.3)    # 10 µF electrolytic, up to Ø5 mm, pads 2.5 mm apart, + on the left
VDD_ROW_Y = 6.0       # front-side VDD trace under the can, left pull-up to right

TRACK = 0.3
SIG = 0.25


def mm(x, y):
    return pcbnew.VECTOR2I(pcbnew.FromMM(x), pcbnew.FromMM(y))


board = pcbnew.BOARD()
board.SetCopperLayerCount(2)
settings = board.GetDesignSettings()
settings.SetBoardThickness(pcbnew.FromMM(1.6))

nets = {}
for name in ["VDD", "GND", "SDA", "SCL"]:
    n = pcbnew.NETINFO_ITEM(board, name)
    board.Add(n)
    nets[name] = n


def outline():
    h = SIZE / 2
    r = CORNER
    segments = [
        ((-h + r, -h), (h - r, -h)), ((h, -h + r), (h, h - r)),
        ((h - r, h), (-h + r, h)), ((-h, h - r), (-h, -h + r)),
    ]
    for (x1, y1), (x2, y2) in segments:
        s = pcbnew.PCB_SHAPE(board)
        s.SetShape(pcbnew.SHAPE_T_SEGMENT)
        s.SetStart(mm(x1, y1))
        s.SetEnd(mm(x2, y2))
        s.SetLayer(pcbnew.Edge_Cuts)
        s.SetWidth(pcbnew.FromMM(0.1))
        board.Add(s)
    for cx, cy, a in [(h - r, -h + r, -90), (h - r, h - r, 0), (-h + r, h - r, 90), (-h + r, -h + r, 180)]:
        s = pcbnew.PCB_SHAPE(board)
        s.SetShape(pcbnew.SHAPE_T_ARC)
        s.SetCenter(mm(cx, cy))
        start = math.radians(a)
        s.SetStart(mm(cx + r * math.cos(start), cy + r * math.sin(start)))
        s.SetArcAngleAndEnd(pcbnew.EDA_ANGLE(90, pcbnew.DEGREES_T))
        s.SetLayer(pcbnew.Edge_Cuts)
        s.SetWidth(pcbnew.FromMM(0.1))
        board.Add(s)


def silk_text(text, x, y, layer, size=0.8, mirror=False, angle=0):
    t = pcbnew.PCB_TEXT(board)
    t.SetText(text)
    t.SetPosition(mm(x, y))
    t.SetTextAngleDegrees(angle)
    t.SetLayer(layer)
    t.SetTextSize(mm(size, size))
    t.SetTextThickness(pcbnew.FromMM(size * 0.15))
    t.SetMirrored(mirror)
    board.Add(t)


def silk_line(x1, y1, x2, y2, layer, width=0.15):
    s = pcbnew.PCB_SHAPE(board)
    s.SetShape(pcbnew.SHAPE_T_SEGMENT)
    s.SetStart(mm(x1, y1))
    s.SetEnd(mm(x2, y2))
    s.SetLayer(layer)
    s.SetWidth(pcbnew.FromMM(width))
    board.Add(s)


def sensor():
    """The MLX90640 footprint: four plated holes, the can's outline and its tab."""
    fp = pcbnew.FOOTPRINT(board)
    fp.SetReference("U1")
    fp.SetValue("MLX90640ESF-BAA")
    fp.SetFPID(pcbnew.LIB_ID("pit-pyro", "MLX90640_TO39"))
    fp.Reference().SetVisible(False)
    fp.Value().SetVisible(False)
    for number, (net, x, y) in SENSOR_PINS.items():
        p = pcbnew.PAD(fp)
        p.SetNumber(number)
        p.SetAttribute(pcbnew.PAD_ATTRIB_PTH)
        p.SetShape(pcbnew.PAD_SHAPE_CIRCLE)
        p.SetSize(pcbnew.F_Cu, mm(1.3, 1.3))
        p.SetDrillSize(mm(0.8, 0.8))
        p.SetLayerSet(pcbnew.PAD.PTHMask())
        p.SetPosition(mm(x, y))
        p.SetNet(nets[net])
        fp.Add(p)
    # The can's outline and the tab, on the front silkscreen.
    c = pcbnew.PCB_SHAPE(fp)
    c.SetShape(pcbnew.SHAPE_T_CIRCLE)
    c.SetCenter(mm(0, 0))
    c.SetEnd(mm(4.85, 0))
    c.SetLayer(pcbnew.F_SilkS)
    c.SetWidth(pcbnew.FromMM(0.15))
    fp.Add(c)
    tab = pcbnew.PCB_SHAPE(fp)
    tab.SetShape(pcbnew.SHAPE_T_POLY)
    tab.SetPolyPoints([mm(-0.6, -4.9), mm(0.6, -4.9), mm(0, -6.0)])
    tab.SetFilled(True)
    tab.SetLayer(pcbnew.F_SilkS)
    tab.SetWidth(pcbnew.FromMM(0.1))
    fp.Add(tab)
    fp.SetPosition(mm(0, 0))
    board.Add(fp)
    return fp


def library(lib, name, ref, value, x, y, angle=0, back=True, nets_by_pad=None):
    fp = pcbnew.FootprintLoad(LIB + lib + ".pretty", name)
    fp.SetReference(ref)
    fp.SetValue(value)
    fp.Value().SetVisible(False)
    fp.Reference().SetVisible(False)
    fp.SetOrientationDegrees(angle)
    fp.SetPosition(mm(x, y))
    board.Add(fp)
    if back:
        fp.SetLayerAndFlip(pcbnew.B_Cu)
    for p in fp.Pads():
        if nets_by_pad and p.GetNumber() in nets_by_pad:
            p.SetNet(nets[nets_by_pad[p.GetNumber()]])
    return fp


def cable_holes():
    """Four plated holes for the cable's wires, which go in from the back and
    are soldered on the front. No connector: a STEMMA QT socket is hard to
    hand-solder and its plug would stick out behind the sensor."""
    fp = pcbnew.FOOTPRINT(board)
    fp.SetReference("J1")
    fp.SetValue("Cable")
    fp.SetFPID(pcbnew.LIB_ID("pit-pyro", "Wire_Holes_1x04"))
    fp.Reference().SetVisible(False)
    fp.Value().SetVisible(False)
    for i, (x, net) in enumerate(zip(J1_X, J1_NETS)):
        p = pcbnew.PAD(fp)
        p.SetNumber(str(i + 1))
        p.SetAttribute(pcbnew.PAD_ATTRIB_PTH)
        p.SetShape(pcbnew.PAD_SHAPE_CIRCLE)
        p.SetSize(pcbnew.F_Cu, mm(1.8, 1.8))
        p.SetDrillSize(mm(1.0, 1.0))
        p.SetLayerSet(pcbnew.PAD.PTHMask())
        p.SetPosition(mm(x, J1_Y))
        p.SetNet(nets[net])
        fp.Add(p)
    board.Add(fp)


def track(points, net, layer, width=TRACK):
    for (x1, y1), (x2, y2) in zip(points, points[1:]):
        t = pcbnew.PCB_TRACK(board)
        t.SetStart(mm(x1, y1))
        t.SetEnd(mm(x2, y2))
        t.SetWidth(pcbnew.FromMM(width))
        t.SetLayer(layer)
        t.SetNet(nets[net])
        board.Add(t)


outline()
sensor()

# Pull-ups: vertical, bodies on the back. Pad 1 at the top.
library("Resistor_THT", "R_Axial_DIN0207_L6.3mm_D2.5mm_P7.62mm_Horizontal", "R1", "4.7k",
        -R_X, R_TOP, angle=-90, nets_by_pad={"1": "SDA", "2": "VDD"})
library("Resistor_THT", "R_Axial_DIN0207_L6.3mm_D2.5mm_P7.62mm_Horizontal", "R2", "4.7k",
        R_X, R_TOP, angle=-90, nets_by_pad={"1": "SCL", "2": "VDD"})
# Supply capacitors, bodies on the back.
# (Turned 180° because flipping to the back mirrors them: pad 2 ends up on the right.)
library("Capacitor_THT", "C_Disc_D3.0mm_W1.6mm_P2.50mm", "C1", "100nF",
        C1_AT[0], C1_AT[1], angle=180, nets_by_pad={"1": "VDD", "2": "GND"})
library("Capacitor_THT", "CP_Radial_D5.0mm_P2.50mm", "C2", "10uF",
        C2_AT[0], C2_AT[1], angle=180, nets_by_pad={"1": "VDD", "2": "GND"})
cable_holes()

B, F = pcbnew.B_Cu, pcbnew.F_Cu
sda, scl, vdd, gnd = (SENSOR_PINS[k][1:] for k in ["1", "4", "2", "3"])
c1_vdd, c1_gnd = C1_AT, (C1_AT[0] + 2.5, C1_AT[1])
c2_vdd, c2_gnd = C2_AT, (C2_AT[0] + 2.5, C2_AT[1])
j = dict(zip(J1_NETS, [(x, J1_Y) for x in J1_X]))

OUTER = 4.2  # x of the power traces that run up the sides, under the can
# Signals: straight up to their cable holes on the back, and from the holes to
# their pull-ups on the front, outside the can.
track([sda, j["SDA"]], "SDA", B, SIG)
track([scl, j["SCL"]], "SCL", B, SIG)
track([j["SDA"], (-R_X, R_TOP)], "SDA", F, SIG)
track([j["SCL"], (R_X, R_TOP)], "SCL", F, SIG)
# VDD up the left side and GND up the right side to their holes, on the back.
track([vdd, (-OUTER, 0.0), (-OUTER, -5.0), j["VDD"]], "VDD", B)
track([gnd, (OUTER, 0.0), (OUTER, -5.0), j["GND"]], "GND", B)
# VDD to the left pull-up on the back; along the bottom on the front to the
# right pull-up and the capacitors.
track([(-R_X, R_BOTTOM), (-OUTER, 0.0)], "VDD", B)
track([(-R_X, R_BOTTOM), (-R_X, 4.6), (-5.9, VDD_ROW_Y), (5.9, VDD_ROW_Y), (R_X, 4.6), (R_X, R_BOTTOM)], "VDD", F)
track([c1_vdd, (c1_vdd[0], VDD_ROW_Y)], "VDD", F)
track([c2_vdd, (c2_vdd[0], VDD_ROW_Y)], "VDD", F)
# GND down to the capacitors, on the back.
track([gnd, (c1_gnd[0], 4.6), c1_gnd], "GND", B)
track([c1_gnd, (c1_gnd[0], 9.0), (c2_gnd[0], 9.0), c2_gnd], "GND", B)

# Silkscreen (0.8 mm text, the usual minimum). Front: the tab and a name.
# Back: what goes where, with the STEMMA QT cable's wire colours.
FS, BS = pcbnew.F_SilkS, pcbnew.B_SilkS
silk_text("pit-pyro", -6.4, 9.0, FS, 0.8)
silk_text("v1", 0, 3.9, BS, 0.8, mirror=True)
for x, label in zip(J1_X, J1_NETS):
    silk_text(label, x, J1_Y + 1.6, BS, 0.8, mirror=True)
for x, colour in zip(J1_X, ["red", "blu", "yel", "blk"]):
    silk_text(colour, x, J1_Y + 2.7, BS, 0.8, mirror=True)
silk_text("4k7", -4.9, 0, BS, 0.8, mirror=True, angle=90)
silk_text("4k7", 4.9, 0, BS, 0.8, mirror=True, angle=90)
silk_text("100n", c1_vdd[0] + 1.25, 6.1, BS, 0.8, mirror=True)
silk_text("10u", c2_vdd[0] + 1.25, 4.0, BS, 0.8, mirror=True)

board.Save(OUT)
print("wrote", OUT)
