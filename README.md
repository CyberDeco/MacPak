# MacPak

<p align="center">
    <img src="MacPak/macpak-gui/assets/icon.iconset/icon_512x512.png" alt="MacPak Icon" width="128">
</p>

---

macOS BG3 modding tool combining functionality from [Norbyte's ExportTool](https://github.com/Norbyte/lslib/releases) and [ShinyHobo's Baldur's Gate 3 Modder's Multitool](https://github.com/ShinyHobo/BG3-Modders-Multitool). Heavier processes, such as .pak and .lsf manipulation, are self-contained - an upgrade from [my previous Python version](https://github.com/CyberDeco/mac-pak).

---

## Features:

### General

Components:

- MacPak - the name of the project and eventual app
- MacLarian - the backend powerhouse behind MacPak (like the sports car... and Larian... and macOS...)
- macpak-cli - command-line interface, called by MacLarian and passed into MacPak
- macpak-gui - the desktop app

Status:

- [x] Working: pak operations (list contents, unpack, verify structure, and pack)
- [x] Working: .lsf <> .lsx <> .lxj conversion
- [ ] Currently fist fighting: converting granny2.dll to work on macOS
- [ ] Future: port over [previous Python GUI](https://github.com/CyberDeco/mac-pak/tree/main/gui_tour)

### Technical

1. Self-contained: no need to download/install/build any dependencies.
2. Automatically checks MacPak repo for updates on launch.

## What it does not do:

1. Edit meshes or textures. You still need to do those in Blender, Photoshop, etc.
2. Hook into the game. Meaning, it doesn't work with anything that would require [Norbyte's BG3SE](https://github.com/Norbyte/bg3se), but check out the [BG3SE-macOS](https://github.com/tdimino/bg3se-macos) project.
3. Edit/access game files. Maybe in the future, but not now.
4. Access or recreate the Official Larian Modding Toolkit. I looked into it and decided it's not worth the time and effort.
5. Work on Windows. Because, well, duh. You people already have those tools.
6. Work on Linux. I'm sure it can be adapted, but I won't be the one to do it.

## What I'm not sure about:

- If it works on Intel Macs: just got my hands on a 2019 Intel iMac, will be testing in the future.
- Efficiency on less powerful Apple silicon Macs: I built this using my 2019 MacBook Pro M2 Max with 32 GB RAM.
- If it works with older versions of BG3: I built this for [V4.1.1.6897358](https://bg3.wiki/wiki/Patch_Notes) and I'm not going to check against older versions.

## Will I ever do X, Y, Z super involved/difficult thing?

Maybe? But this haqs been a lot to put together, it's my first time doing anything like this, and I work full-time. Feel free to [make suggestions](https://github.com/CyberDeco/MacPak/issues/new/) and/or initiate pull requests, it's just not guaranteed that I'll implement them.

