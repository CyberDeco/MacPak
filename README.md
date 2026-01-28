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

![Custom](https://img.shields.io/badge/MacPak-v0.1.0-7706be?logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz48IS0tIFVwbG9hZGVkIHRvOiBTVkcgUmVwbywgd3d3LnN2Z3JlcG8uY29tLCBHZW5lcmF0b3I6IFNWRyBSZXBvIE1peGVyIFRvb2xzIC0tPg0KPHN2ZyB3aWR0aD0iODAwcHgiIGhlaWdodD0iODAwcHgiIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4NCjxwYXRoIGQ9Ik0yMSAxMi40OTg0TDE0IDEyLjQ5ODRNMyAxMi40OTg0TDEwIDEyLjQ5ODRNNyA0LjVWMTkuNDk4NE0xNyA0LjVWMTkuNDk4NE02LjIgMTkuNUgxNy44QzE4LjkyMDEgMTkuNSAxOS40ODAyIDE5LjUgMTkuOTA4IDE5LjI4MkMyMC4yODQzIDE5LjA5MDMgMjAuNTkwMyAxOC43ODQzIDIwLjc4MiAxOC40MDhDMjEgMTcuOTgwMiAyMSAxNy40MjAxIDIxIDE2LjNWMTAuOUMyMSA4LjY1OTc5IDIxIDcuNTM5NjggMjAuNTY0IDYuNjg0MDRDMjAuMTgwNSA1LjkzMTM5IDE5LjU2ODYgNS4zMTk0NyAxOC44MTYgNC45MzU5N0MxNy45NjAzIDQuNSAxNi44NDAyIDQuNSAxNC42IDQuNUg5LjRDNy4xNTk3OSA0LjUgNi4wMzk2OCA0LjUgNS4xODQwNCA0LjkzNTk3QzQuNDMxMzkgNS4zMTk0NyAzLjgxOTQ3IDUuOTMxMzkgMy40MzU5NyA2LjY4NDA0QzMgNy41Mzk2OCAzIDguNjU5NzkgMyAxMC45VjE2LjNDMyAxNy40MjAxIDMgMTcuOTgwMiAzLjIxNzk5IDE4LjQwOEMzLjQwOTczIDE4Ljc4NDMgMy43MTU2OSAxOS4wOTAzIDQuMDkyMDIgMTkuMjgyQzQuNTE5ODQgMTkuNSA1LjA3OTg5IDE5LjUgNi4yIDE5LjVaTTEwIDEwLjQ5ODRIMTRWMTQuNDk4NEgxMFYxMC40OTg0WiIgc3Ryb2tlPSIjRkZGRkZGIiBzdHJva2Utd2lkdGg9IjEuNSIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIvPg0KPC9zdmc+DQo=)
![Custom](https://img.shields.io/badge/MacLarian-v0.1.0-68fdff?logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz48IS0tIFVwbG9hZGVkIHRvOiBTVkcgUmVwbywgd3d3LnN2Z3JlcG8uY29tLCBHZW5lcmF0b3I6IFNWRyBSZXBvIE1peGVyIFRvb2xzIC0tPgo8c3ZnIGZpbGw9IiNGRkZGRkYiIHdpZHRoPSI4MDBweCIgaGVpZ2h0PSI4MDBweCIgdmlld0JveD0iMCAwIDUxMiA1MTIiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+PHRpdGxlPmlvbmljb25zLXY1LWg8L3RpdGxlPjxwYXRoIGQ9Ik00OTQuMjYsMjc2LjIyYy0zLjYtNDAuNDEtOS41My00OC4yOC0xMS43Ny01MS4yNC01LjE1LTYuODQtMTMuMzktMTEuMzEtMjIuMTEtMTZsMCwwYTMuNiwzLjYsMCwwLDEtLjkxLTUuNjhBMTUuOTMsMTUuOTMsMCwwLDAsNDY0LDE5MC43NywxNi4yNywxNi4yNywwLDAsMCw0NDcuNjUsMTc2aC0xNS42YTE3LDE3LDAsMCwwLTIsLjEzLDguNSw4LjUsMCwwLDAtMS40MS0uNDdsMCwwYy05LjI0LTE5LjUzLTIxLjg5LTQ2LjI3LTQ4LjExLTU5LjMyQzM0MS42NCw5NywyNzAsOTYsMjU2LDk2cy04NS42NCwxLTEyNC40OCwyMC4zMWMtMjYuMjIsMTMuMDUtMzguODcsMzkuNzktNDguMTEsNTkuMzJsLS4wOC4xNmE2LjUyLDYuNTIsMCwwLDAtMS4zNS4zNCwxNywxNywwLDAsMC0yLS4xM0g2NC4zNUExNi4yNywxNi4yNywwLDAsMCw0OCwxOTAuNzdhMTUuOTMsMTUuOTMsMCwwLDAsNC41OSwxMi40NywzLjYsMy42LDAsMCwxLS45MSw1LjY4bDAsMGMtOC43Miw0LjcyLTE3LDkuMTktMjIuMTEsMTYtMi4yNCwzLTguMTYsMTAuODMtMTEuNzcsNTEuMjQtMiwyMi43NC0yLjMsNDYuMjgtLjczLDYxLjQ0LDMuMjksMzEuNSw5LjQ2LDUwLjU0LDkuNzIsNTEuMzNhMTYsMTYsMCwwLDAsMTMuMiwxMC44N2gwVjQwMGExNiwxNiwwLDAsMCwxNiwxNmg1NmExNiwxNiwwLDAsMCwxNi0xNmgwYzguNjEsMCwxNC42LTEuNTQsMjAuOTUtMy4xOGExNTguODMsMTU4LjgzLDAsMCwxLDI4LTQuOTFDMjA3LjQ1LDM4OSwyMzcuNzksMzg4LDI1NiwzODhjMTcuODQsMCw0OS41MiwxLDgwLjA4LDMuOTFhMTU5LjE2LDE1OS4xNiwwLDAsMSwyOC4xMSw0LjkzYzYuMDgsMS41NiwxMS44NSwzLDE5Ljg0LDMuMTVoMGExNiwxNiwwLDAsMCwxNiwxNmg1NmExNiwxNiwwLDAsMCwxNi0xNnYtLjEyaDBBMTYsMTYsMCwwLDAsNDg1LjI3LDM4OWMuMjYtLjc5LDYuNDMtMTkuODMsOS43Mi01MS4zM0M0OTYuNTYsMzIyLjUsNDk2LjI4LDI5OSw0OTQuMjYsMjc2LjIyWk0xMTIuMzMsMTg5LjMxYzgtMTcsMTcuMTUtMzYuMjQsMzMuNDQtNDQuMzUsMjMuNTQtMTEuNzIsNzIuMzMtMTcsMTEwLjIzLTE3czg2LjY5LDUuMjQsMTEwLjIzLDE3YzE2LjI5LDguMTEsMjUuNCwyNy4zNiwzMy40NCw0NC4zNWwxLDIuMTdhOCw4LDAsMCwxLTcuNDQsMTEuNDJDMzYwLDIwMiwyOTAsMTk5LjEyLDI1NiwxOTkuMTJzLTEwNCwyLjk1LTEzNy4yOCwzLjg1YTgsOCwwLDAsMS03LjQ0LTExLjQyQzExMS42MywxOTAuODEsMTEyLDE5MC4wNiwxMTIuMzMsMTg5LjMxWm0xMS45Myw3OS42M0E0MjcuMTcsNDI3LjE3LDAsMCwxLDcyLjQyLDI3MmMtMTAuNiwwLTIxLjUzLTMtMjMuNTYtMTIuNDQtMS4zOS02LjM1LTEuMjQtOS45Mi0uNDktMTMuNTFDNDksMjQzLDUwLDI0MC43OCw1NSwyNDBjMTMtMiwyMC4yNy41MSw0MS41NSw2Ljc4LDE0LjExLDQuMTUsMjQuMjksOS42OCwzMC4wOSwxNC4wNkMxMjkuNTUsMjYzLDEyOCwyNjguNjQsMTI0LjI2LDI2OC45NFptMjIxLjM4LDgyYy0xMy4xNiwxLjUtMzkuNDguOTUtODkuMzQuOTVzLTc2LjE3LjU1LTg5LjMzLS45NWMtMTMuNTgtMS41MS0zMC44OS0xNC4zNS0xOS4wNy0yNS43OSw3Ljg3LTcuNTQsMjYuMjMtMTMuMTgsNTAuNjgtMTYuMzVTMjMzLjM4LDMwNCwyNTYuMiwzMDRzMzIuMTIsMSw1Ny42Miw0LjgxLDQ0Ljc3LDkuNTIsNTAuNjgsMTYuMzVDMzc1LjI4LDMzNy40LDM1OS4yMSwzNDkuMzUsMzQ1LjY0LDM1MVptMTE3LjUtOTEuMzljLTIsOS40OC0xMywxMi40NC0yMy41NiwxMi40NGE0NTUuOTEsNDU1LjkxLDAsMCwxLTUyLjg0LTMuMDZjLTMuMDYtLjI5LTQuNDgtNS42Ni0xLjM4LTguMSw1LjcxLTQuNDksMTYtOS45MSwzMC4wOS0xNC4wNiwyMS4yOC02LjI3LDMzLjU1LTguNzgsNDQuMDktNi42OSwyLjU3LjUxLDMuOTMsMy4yNyw0LjA5LDVBNDAuNjQsNDAuNjQsMCwwLDEsNDYzLjE0LDI1OS41NloiLz48L3N2Zz4K)
![Rust](https://img.shields.io/badge/rust-1.85+-orange?logo=rust)

## Install Instructions

It'll be an app in Releases, bundled with a macOS binary for vgmstream. Size is ~40 MB.

## Features:

### General

Components:

- MacPak - the package itself; contains GUI
- MacLarian - the "engine" behind MacPak (geddit, because macOS + Larian = the sports car manufacturer...?); also contains CLI

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
