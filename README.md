<p align="center">
    <img width="100" alt="WgShadertoy Logo" src="https://raw.githubusercontent.com/fralonra/wgshadertoy/master/extra/logo/wgshadertoy.svg">
</p>

<h1 align="center">WgShadertoy</h1>

![Flathub](https://img.shields.io/flathub/downloads/io.github.fralonra.WgShadertoy?label=flathub)
![AUR](https://img.shields.io/aur/version/wgshadertoy)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

A [WGSL](https://www.w3.org/TR/WGSL/) playground inspired by [Shadertoy](https://www.shadertoy.com/).

<img src="https://raw.githubusercontent.com/fralonra/wgshadertoy/master/doc/basic.png" alt="preview">

## Installation

### Windows

Check out the [latest release](https://github.com/fralonra/wgshadertoy/releases) for a `msi` installer.


### MacOS

Available on [MacPorts](https://ports.macports.org/port/wgshadertoy/):

```
sudo port install wgshadertoy
```

### Linux

Available on [Flathub](https://flathub.org/apps/io.github.fralonra.WgShadertoy).

<a href="https://flathub.org/apps/io.github.fralonra.WgShadertoy">
  <img width="240" alt="Download on Flathub" src="https://dl.flathub.org/assets/badges/flathub-badge-en.png" align="start"/>
</a>

<a href="https://repology.org/project/wgshadertoy/versions">
  <img src="https://repology.org/badge/vertical-allrepos/wgshadertoy.svg" alt="Packaging status" align="right">
</a>

For Arch Linux users, `wgshadertoy` is also available on AUR:

```
yay -S wgshadertoy
```

## Wgs format

The application use a binary format [`wgs`](https://github.com/fralonra/wgs) to save and load shaders and textures.

It helps to share your shaders amoung people.

You can find examples in [wgs's repo](https://github.com/fralonra/wgs/tree/master/examples).

## Limits

- The amount of the texture you can upload is [the max bind group count of your device](https://docs.rs/wgpu/latest/wgpu/struct.Limits.html#structfield.max_bind_groups) - 1.
- Won't show any hints on invalid shaders. They just won't compile.

## Todos

- Web version
- Advanced shader validation (detailed in https://github.com/fralonra/wgshadertoy/issues/1)

## Credits

- [wgpu](https://github.com/gfx-rs/wgpu) for rendering.
- [egui](https://github.com/emilk/egui) for UI.
- [binrw](https://github.com/jam1garner/binrw) for binary data read/write.
- [shadertoy](https://github.com/adamnemecek/shadertoy) for the wonderful vertex shader.
