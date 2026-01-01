# MacPak - A BG3 Modding Toolkit for macOS

<p align="center">
    <a href="#">
        <img src="macpak-gui/assets/icons/icon.iconset/icon_512x512.png" alt="MacPak Icon" width="128">
    </a>
</p>

---

<p align="center">
<i>Baldurs Gate 3 modding tools for macOS? In <b>this</b> economy?</i>
</p>

---

## Install Instructions

It's going to be packaged into a .dmg probably, since I don't want to pay for an Apple dev license and I'm assuming it'll be a large file. Just haven't figured out hosting yet.

## Features:

### General

Components:

- MacPak - the frontend name of the project/app and is the public API which calls MacLarian on the backend
- MacLarian - the "engine" behind MacPak (geddit, because macOS + Larian = the sports car manufacturer...?)
- macpak-bevy - backend for GR2/glTF previews using bevy
- macpak-cli - command-line interface, calls MacLarian via the MacPak API
- macpak-gui - the desktop app

Status:

- [x] Working: pak operations (list contents, unpack single/batch, verify structure, and pack single/batch)
- [x] Working: .lsf <> .lsx <> .lsj conversion
- [x] Working: inline text file editor with multiple tabbed support and meta.lsx generator
- [x] Working: various QOL things - UUID/TranslatedString generator, color picker/saver, etc.
- [x] Temporarily working: DDS file previews (it's a little jank because it's a stopgap fix)
- [x] Working: convert .gr2 files <> .glb or .glTF (same as .glb but .bin is its own file)
- [x] Working: decompress and stitch virtual textures
  - Attempted: reverse-engineering compression for .gr2 files, but got to be too annoying for just a QOL implementation
- [x] Working: local Floem GUI (not yet packaged for production)
  - [x] File tree/preview/browser tab
  - [x] Text editor tab + meta.lsx generator
  - [x] Pak operations tab
  - [x] Tab for gr2 <> glTF/GLB single/batch conversion
  - [x] Tab for extracting and stitching single/batch virtual textures
    - [ ] Future: work in reverse for custom virtual textures (needs BG3SE for testing)
- [ ] Currently: incorporate bevy for 3D model and texture rendering (just previews)
  - [x] .glb/.gltf
  - [x] .gr2 (via temp .glb conversion)
  - [ ] .dds - can already load 512 x 512 previews in the GUI
  - [ ] marry .dds to GR2/glTF (need database for that)
- [ ] Next: Piece together x-ref library for GR2 files to their base/normal/physical virtual textures and/or cloth msk
    - [ ] Option to automatically retrieve companion texture files when working with GR2
- [ ] Currently: dye lab
    - [x] Mimic Official Toolkit's dye menu... thing... and improve it + use built-in macOS Color Picker
    - [ ] Preview on a GR2 + mesh ?
    - [x] Auto-populate mod structure based on Padme4000's template

This is meant to be an all-in-one tool, so if you're looking for a simple or lightweight GUI... this ain't it. I'll get to making a fleshed out wiki once it's release-worthy.

## Dye Lab

No more going back and forth between the Official Larian Toolkit, a hex code generator, and a text editor! The Dye Lab tab uses the native Color Picker in macOS (which also has sliders, hex code input, etc.) to choose a

## Technical

1. Self-contained: no need to download/install/build any dependencies - will be at release.
2. Ported over most of what [LSLib](https://github.com/Norbyte/lslib) does, with the exception of . LSLib metadata is still used in MacPak's LSF/LSX/LSJ output as a nod to the GOAT.
3. Removed reliance on granny2.dll for GR2 handling, which was, by far, the biggest hurdle for macOS. ***Huge*** shoutout to [neptuwunium](https://github.com/arves100/opengr2/issues/8) for their clean room reverse-engineering of the rANS decompression algorithm in [Knit](https://github.com/neptuwunium/Knit).
4. ~~Automatically checks MacPak repo for updates on launch.~~

## What it does not do:

1. Edit meshes or textures. You still need to do those in Blender, Photoshop/GIMP, Substance, etc.
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

Because attempting this in Python first made me realize I needed a beefier (read: compiler) language and I didn't want to learn C, C#, or C++.

## Credits

Core MacPak functionality is derived from (and wouldn't be possible without):
- [LSLib](https://github.com/Norbyte/lslib) 
  - PAK handling
  - LSF/LSX/LSJ handling
  - GR2/glTF conversion
  - Virtual texture handling
  - Loca file handling
- [xiba](https://gitlab.com/saghm/xiba/)
  - PAK handling
- [Knit](https://github.com/neptuwunium/Knit)
  - GR2 decompression