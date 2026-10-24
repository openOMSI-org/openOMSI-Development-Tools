#!/usr/bin/env python3
"""Makes the app icon of openOMSI Development Tools.

The wordmark's outlines are the ones of the openOMSI icon (assets/icons/app/openomsi.svg of the
game, passed with --wordmark); the tools' badge, the `</>` under it, is drawn here. Writes the
SVG sources and, with --resvg <binary>, the PNG sizes, the Windows .ico and the macOS .icns
(iconutil, on a Mac) into assets/icons/.

    tools/make-icons.py --wordmark <openomsi.svg> --resvg <resvg>
"""
import argparse, os, re, struct, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "..", "assets", "icons")

ap = argparse.ArgumentParser()
ap.add_argument("--wordmark", required=True)
ap.add_argument("--resvg")
args = ap.parse_args()

src = open(args.wordmark).read()
paths = re.findall(r'<svg x="[^"]+" y="[^"]+" width="[^"]+" height="[^"]+" viewBox="[^"]+">\s*<path d="[^"]+" fill="[^"]+"/>\s*</svg>', src)
assert len(paths) == 2, "the wordmark file has two parts, 'open' and 'OMSI'"
word = "\n".join(paths)

BG = '<defs><linearGradient id="bg" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#2d3138"/><stop offset="1" stop-color="#14161a"/></linearGradient></defs>'

def art(tx, ty, k, bx, by, bk):
    """The wordmark scaled by k about (512, 507) and moved by (tx, ty); the badge at (bx, by)."""
    sw = 46
    badge = f'''<g transform="translate({bx} {by}) scale({bk})" fill="none" stroke-linecap="round" stroke-linejoin="round" stroke-width="{sw}">
<path d="M-118 -62L-190 0L-118 62" stroke="#ffffff"/>
<path d="M118 -62L190 0L118 62" stroke="#ffffff"/>
<path d="M28 -88L-28 88" stroke="#F78034"/>
</g>'''
    return f'<g transform="translate({tx} {ty}) translate(512 507) scale({k}) translate(-512 -507)">\n{word}\n</g>\n{badge}'

def svg(body, rect):
    return f'<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024">\n{BG}\n{rect}\n{body}\n</svg>\n'

# the Windows / Linux / web icon fills the square; the macOS one sits inside the grid Apple draws
flat = svg(art(0, -62, 0.84, 512, 752, 0.72), '<rect x="24" y="24" width="976" height="976" rx="200" fill="url(#bg)"/>')
# (the macOS art is the flat one shrunk about the middle of the canvas, 824/976 of its size)
mac_k = 824 / 976
mac_body = f'<g transform="translate(512 512) scale({mac_k}) translate(-512 -512)">\n{art(0, -62, 0.84, 512, 752, 0.72)}\n</g>'
mac = svg(mac_body, '<rect x="100" y="100" width="824" height="824" rx="185" fill="url(#bg)"/>')

os.makedirs(OUT, exist_ok=True)
open(os.path.join(OUT, "devtools.svg"), "w").write(flat)
open(os.path.join(OUT, "devtools-macos.svg"), "w").write(mac)
print("wrote the SVGs")
if not args.resvg:
    sys.exit(0)

def png(svgfile, size, out):
    subprocess.check_call([args.resvg, svgfile, out, "-w", str(size), "-h", str(size)])

flat_svg = os.path.join(OUT, "devtools.svg")
mac_svg = os.path.join(OUT, "devtools-macos.svg")
for s in (32, 128, 256, 512, 1024):
    png(flat_svg, s, os.path.join(OUT, f"devtools-{s}.png"))
png(mac_svg, 1024, os.path.join(OUT, "devtools-macos-1024.png"))

# .ico: PNG-compressed entries (Vista and later)
sizes = [16, 24, 32, 48, 64, 128, 256]
with tempfile.TemporaryDirectory() as t:
    blobs = []
    for s in sizes:
        p = os.path.join(t, f"{s}.png")
        png(flat_svg, s, p)
        blobs.append(open(p, "rb").read())
head = struct.pack("<HHH", 0, 1, len(sizes))
off = 6 + 16 * len(sizes)
ents = b""
for s, b in zip(sizes, blobs):
    ents += struct.pack("<BBBBHHII", s % 256, s % 256, 0, 0, 1, 32, len(b), off)
    off += len(b)
open(os.path.join(OUT, "devtools.ico"), "wb").write(head + ents + b"".join(blobs))

# .icns through iconutil (macOS only)
if sys.platform == "darwin":
    with tempfile.TemporaryDirectory() as t:
        iset = os.path.join(t, "devtools.iconset")
        os.makedirs(iset)
        for base in (16, 32, 128, 256, 512):
            png(mac_svg, base, os.path.join(iset, f"icon_{base}x{base}.png"))
            png(mac_svg, base * 2, os.path.join(iset, f"icon_{base}x{base}@2x.png"))
        subprocess.check_call(["iconutil", "-c", "icns", iset, "-o", os.path.join(OUT, "devtools.icns")])
print("wrote the PNGs, devtools.ico and devtools.icns")
