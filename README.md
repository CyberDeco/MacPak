# MacPak - A BG3 Modding Toolkit for macOS

<p align="center">
    <a href="#">
        <img src="MacPak/src/gui/assets/icons/icon.iconset/icon_512x512.png" alt="MacPak Icon" width="128">
    </a>
</p>

---

<p align="center">
<i>Baldurs Gate 3 modding tools for macOS? In <b>this</b> economy?</i>
</p>

---

## Install Instructions

It's going to be packaged into a .dmg probably, since I don't want to pay for an Apple dev license and I'm assuming it'll be a large file (~1 GB). Just haven't figured out hosting yet.

## Features:

### General

Components:

- MacPak - the package itself; contains GUI and CLI
- MacLarian - the "engine" behind MacPak (geddit, because macOS + Larian = the sports car manufacturer...?)

This is meant to be an all-in-one tool, so if you're looking for a simple or lightweight GUI... this ain't it. I'll get to making a fleshed out wiki once it's release-worthy.


## Dye Lab

No more going back and forth between the Official Larian Toolkit, a hex code generator, and a text editor! The Dye Lab tab uses the native Color Picker in macOS (which also has sliders, hex code input, etc.) to choose a

## Technical

1. Self-contained: no need to download/install/build any dependencies - will be at release.
2. Ported over most of what [LSLib](https://github.com/Norbyte/lslib) does, with the exception of . LSLib metadata is still used in MacPak's LSF/LSX/LSJ output as a nod to the GOAT.
3. Removed reliance on granny2.dll for GR2 handling, which was, by far, the biggest hurdle for macOS. ***Huge*** shoutout to [neptuwunium](https://github.com/arves100/opengr2/issues/8) for their clean room reverse-engineering.
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
- [BG3 Dialog Reader](https://github.com/angaityel/bg3-dialog-reader)
  - Character dialogue handling