# MacPak - BG3 Modding Toolkit for macOS

<p align="center">
    <a href="#">
        <img src="macpak-gui/assets/icons/icon.iconset/icon_512x512.png" alt="MacPak Icon" width="128">
    </a>
</p>

---

macOS BG3 modding tool combining functionality from [Norbyte's ExportTool](https://github.com/Norbyte/lslib/releases) and [ShinyHobo's Baldur's Gate 3 Modder's Multitool](https://github.com/ShinyHobo/BG3-Modders-Multitool), along with a whole mess of unique features as well.

---

## Features:

### General

Components:

- MacPak - the frontend name of the project and eventual app, calls MacLarian on the backend
- MacLarian - the backend powerhouse behind MacPak (like the sports car... and Larian... and macOS...)
- macpak-cli - command-line interface, called by MacLarian and passed into MacPak
- macpak-gui - the desktop app

Status:

- [x] Working: pak operations (list contents, unpack single/batch, verify structure, and pack single/batch)
- [x] Working: .lsf <> .lsx <> .lsj conversion
- [x] Working: local Floem GUI (not yet packaged for production)
- [x] Working: inline text file editor with multiple tabbed support and meta.lsx generator
- [x] Working: various QOL things - UUID/TranslatedString generator, color picker/saver, etc.
- [x] Temporarily working: DDS file previews (it's a little jank because it's a stopgap fix)
- [x] Working: convert .gr2 files to .glb
- [ ] Currently: converting .glTF back to .gr2
- [ ] Future: incorporate bevy for 3D model rendering (just previews)

I'll get to making a fleshed out wiki once it's release-worthy.

### Technical

1. Self-contained: no need to download/install/build any dependencies.
2. ~~Automatically checks MacPak repo for updates on launch.~~

## What it does not do:

1. Edit meshes or textures. You still need to do those in Blender, Photoshop, etc.
2. Hook into the game. Meaning, it doesn't work with anything that would require [Norbyte's BG3SE](https://github.com/Norbyte/bg3se), but check out the [BG3SE-macOS](https://github.com/tdimino/bg3se-macos) project.
3. Edit/access game files. Maybe in the future, but not now.
4. Access or recreate the Official Larian Modding Toolkit. I looked into it and decided it's not worth the time and effort.
5. Work on Windows. Because, well, duh. You people already have these tools.
6. Work on Linux - check out [xiba](https://gitlab.com/saghm/xiba/) for a Linux-focused project.
7. Work for DOS/DOS2 - I don't have those games and therefore have no desire to implement that support.

I mean, yes, *technically* Rust is OS-agnostic, so it could be finagled to work for Windows or Linux. But I'm not interested in maintaining cross-platform compatibility when this is a macOS-focused project.

## What I'm not sure about:

- If it works on Intel Macs: just got my hands on a 2019 Intel iMac with 32 GB RAM/4 GB VRAM, will be testing in the future.
- Efficiency on less powerful Apple silicon Macs: I built this using my 2023 MacBook Pro M2 Max with 32 GB RAM/VRAM.
- If it works with older versions of BG3: I built this for [V4.1.1.6897358](https://bg3.wiki/wiki/Patch_Notes) and I'm not going to check against older versions.

## Will I ever do X, Y, Z super involved/difficult thing?

Maybe? But this has been a lot to put together, it's my first time doing anything like this, and I work full-time. Feel free to [make suggestions](https://github.com/CyberDeco/MacPak/issues/new/) and/or initiate pull requests, it's just not guaranteed that I'll implement them.

## Why is this written in *Rust*???

Because attempting this in Python first made me realize I needed a beefier (read: compiler) language and I didn't want to learn C, C#, or C++, among other reasons.

# Credits

Core MacPak functionality is derived from (and wouldn't be possible without):
- [LSLib](https://github.com/Norbyte/lslib) 
  - LSF/LSX/LSJ handling
  - GR2/glTF conversion
- [xiba](https://gitlab.com/saghm/xiba/)
  - PAK handling
- [Knit](https://github.com/neptuwunium/Knit)
  - GR2 decompression