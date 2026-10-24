#!/usr/bin/env python3
"""Builds the openOMSI Development Tools and packs them for one platform into dist/:

  openOMSI-Dev-Tools-<version>-<os>-<arch>.zip   the app
  oopc-<version>-<os>-<arch>.zip                 the command-line compiler

Every package is a .zip, on every system. The macOS one holds `openOMSI Dev Tools.app`, the
Windows and Linux ones a folder with the program, README and licence.

    python3 tools/package.py                          # for this computer
    python3 tools/package.py aarch64-apple-darwin     # for another target (rustup target add first)
    python3 tools/package.py --no-build <target>      # only pack what is already built
"""
import os, re, shutil, stat, subprocess, sys, zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DIST = ROOT / "dist"
APP_NAME = "openOMSI Dev Tools"


def version():
    text = (ROOT / "Cargo.toml").read_text()
    return re.search(r'^version = "([^"]+)"', text, re.M).group(1)


def host_target():
    out = subprocess.check_output(["rustc", "-vV"], text=True)
    return re.search(r"^host: (.+)$", out, re.M).group(1)


def label(target):
    arch = {"x86_64": "x64", "aarch64": "arm64"}.get(target.split("-")[0])
    if arch is None:
        sys.exit(f"unknown architecture in {target}")
    osname = "windows" if "windows" in target else "macos" if "apple-darwin" in target else "linux"
    return osname, arch


def zip_dir(src_root, entries, out):
    """Zips `entries` (paths under `src_root`, folders included whole) with their unix modes."""
    out.parent.mkdir(parents=True, exist_ok=True)
    if out.exists():
        out.unlink()
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
        for entry in entries:
            base = src_root / entry
            files = [base] if base.is_file() else sorted(p for p in base.rglob("*"))
            for p in files:
                arc = p.relative_to(src_root).as_posix()
                if p.is_dir():
                    z.writestr(zipfile.ZipInfo(arc + "/"), "")
                    continue
                info = zipfile.ZipInfo.from_file(p, arc)
                info.compress_type = zipfile.ZIP_DEFLATED
                z.writestr(info, p.read_bytes())
    return out


def release_dir(target):
    """Where cargo puts the programs: with no --target for this computer's own, else beside it."""
    return ROOT / "target" / "release" if target == host_target() else ROOT / "target" / target / "release"


def build(target):
    cmd = ["cargo", "build", "--release", "--locked", "-p", "openomsi-devtools", "-p", "oopc"]
    if target != host_target():
        cmd += ["--target", target]
    subprocess.check_call(cmd, cwd=ROOT)


INFO_PLIST = """<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>{name}</string>
  <key>CFBundleDisplayName</key><string>{name}</string>
  <key>CFBundleIdentifier</key><string>org.openomsi.devtools</string>
  <key>CFBundleVersion</key><string>{version}</string>
  <key>CFBundleShortVersionString</key><string>{version}</string>
  <key>CFBundleExecutable</key><string>openomsi-devtools</string>
  <key>CFBundleIconFile</key><string>devtools</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.developer-tools</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
"""

DESKTOP = """[Desktop Entry]
Type=Application
Name=openOMSI Dev Tools
Comment=Write, run, build and sign plugins for openOMSI
Exec=openomsi-devtools
Icon=openomsi-devtools
Terminal=false
Categories=Development;Game;
"""


def package_app(target, ver, osname, arch):
    release = release_dir(target)
    stage = DIST / "stage-app"
    shutil.rmtree(stage, ignore_errors=True)
    stage.mkdir(parents=True)
    out = DIST / f"openOMSI-Dev-Tools-{ver}-{osname}-{arch}.zip"
    if osname == "macos":
        app = stage / f"{APP_NAME}.app"
        (app / "Contents" / "MacOS").mkdir(parents=True)
        (app / "Contents" / "Resources").mkdir(parents=True)
        exe = app / "Contents" / "MacOS" / "openomsi-devtools"
        shutil.copy2(release / "openomsi-devtools", exe)
        exe.chmod(0o755)
        shutil.copy2(ROOT / "assets" / "icons" / "devtools.icns", app / "Contents" / "Resources" / "devtools.icns")
        (app / "Contents" / "Info.plist").write_text(INFO_PLIST.format(name=APP_NAME, version=ver))
        # (ad hoc signature: the program runs where it was built, and Gatekeeper's first-run
        # question is the ordinary one for an app from the internet)
        subprocess.call(["codesign", "--force", "--deep", "--sign", "-", str(app)], stderr=subprocess.DEVNULL)
        if out.exists():
            out.unlink()
        subprocess.check_call(["ditto", "-c", "-k", "--keepParent", str(app), str(out)])
    else:
        name = f"openOMSI-Dev-Tools-{ver}-{osname}-{arch}"
        folder = stage / name
        folder.mkdir()
        if osname == "windows":
            shutil.copy2(release / "openomsi-devtools.exe", folder / f"{APP_NAME}.exe")
        else:
            exe = folder / "openomsi-devtools"
            shutil.copy2(release / "openomsi-devtools", exe)
            subprocess.call(["strip", str(exe)], stderr=subprocess.DEVNULL)
            exe.chmod(0o755)
            shutil.copy2(ROOT / "assets" / "icons" / "devtools-256.png", folder / "openomsi-devtools.png")
            (folder / "openomsi-devtools.desktop").write_text(DESKTOP)
        shutil.copy2(ROOT / "README.md", folder / "README.md")
        shutil.copy2(ROOT / "LICENSE", folder / "LICENSE")
        zip_dir(stage, [name], out)
    shutil.rmtree(stage, ignore_errors=True)
    return out


def package_cli(target, ver, osname, arch):
    release = release_dir(target)
    stage = DIST / "stage-cli"
    shutil.rmtree(stage, ignore_errors=True)
    name = f"oopc-{ver}-{osname}-{arch}"
    folder = stage / name
    (folder / "docs").mkdir(parents=True)
    exe = "oopc.exe" if osname == "windows" else "oopc"
    shutil.copy2(release / exe, folder / exe)
    if osname != "windows":
        (folder / exe).chmod(0o755)
        subprocess.call(["strip", str(folder / exe)], stderr=subprocess.DEVNULL)
    shutil.copy2(ROOT / "LICENSE", folder / "LICENSE")
    shutil.copy2(ROOT / "docs" / "cli.md", folder / "docs" / "cli.md")
    shutil.copy2(ROOT / "docs" / "getting-started.md", folder / "docs" / "getting-started.md")
    out = zip_dir(stage, [name], DIST / f"{name}.zip")
    shutil.rmtree(stage, ignore_errors=True)
    return out


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    target = args[0] if args else host_target()
    osname, arch = label(target)
    ver = version()
    if "--no-build" not in sys.argv:
        build(target)
    DIST.mkdir(exist_ok=True)
    for f in (package_app(target, ver, osname, arch), package_cli(target, ver, osname, arch)):
        print(f)


if __name__ == "__main__":
    main()
