"""Generate macOS icon assets from the vendored Lucide monitor-off SVG."""

from copy import deepcopy
from pathlib import Path
import subprocess
import xml.etree.ElementTree as ET


ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "assets"
SOURCE = ASSETS / "lucide" / "monitor-off.svg"
WORK = ROOT / "target" / "icons"
SVG_NS = "http://www.w3.org/2000/svg"


def run(*args):
    subprocess.run([str(arg) for arg in args], check=True)


def main():
    WORK.mkdir(parents=True, exist_ok=True)
    ET.register_namespace("", SVG_NS)

    # PDF keeps the menu bar template sharp at any display scale. macOS supplies
    # the appropriate color for light, dark, and highlighted menu bars.
    run("rsvg-convert", "--format", "pdf", "--output",
        ASSETS / "MonitorOffTemplate.pdf", SOURCE)

    app = ET.fromstring(f"""<svg xmlns="{SVG_NS}" width="1024" height="1024"
        viewBox="0 0 1024 1024">
      <defs>
        <linearGradient id="background" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stop-color="#334155" />
          <stop offset="1" stop-color="#111827" />
        </linearGradient>
      </defs>
      <rect x="64" y="64" width="896" height="896" rx="200"
        fill="url(#background)" />
    </svg>""")
    symbol = deepcopy(ET.parse(SOURCE).getroot())
    symbol.attrib.update(x="224", y="224", width="576", height="576", stroke="#ffffff")
    app.append(symbol)
    app_svg = ASSETS / "AppIcon.svg"
    ET.indent(app, space="  ")
    ET.ElementTree(app).write(app_svg, encoding="unicode", xml_declaration=False)
    with app_svg.open("a") as file:
        file.write("\n")

    # iconutil needs the complete set of standard and Retina representations.
    iconset = WORK / "AppIcon.iconset"
    iconset.mkdir(exist_ok=True)
    for points in (16, 32, 128, 256, 512):
        for scale in (1, 2):
            pixels = points * scale
            suffix = "@2x" if scale == 2 else ""
            filename = iconset / f"icon_{points}x{points}{suffix}.png"
            run("rsvg-convert", "--width", pixels, "--height", pixels,
                "--output", filename, app_svg)
    run("iconutil", "--convert", "icns", "--output", ASSETS / "AppIcon.icns", iconset)
    print("Generated MonitorOffTemplate.pdf, AppIcon.svg, and AppIcon.icns")


if __name__ == "__main__":
    main()
