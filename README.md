# TurboUI

One way to describe a screen, drawn natively on each platform. Part of [TurboSuite](https://turbosuite.prismalite.org) by TurboNZL. First user: [TurboMC Server](https://github.com/TurboNZL/turbomc-server).

You write a screen once, in Rust, using plain building blocks (boxes, text) and named theme tokens. TurboUI works out where everything goes, and a thin **backend** per platform draws the result.

| Layer | What it does | Language |
|---|---|---|
| Your app | Describes screens and behaviour | Rust |
| `turboui-core` | Building blocks, themes, layout, text sizes. Knows nothing about any platform | Rust |
| Backends | Draw what the core laid out | Desktop: Rust + GPUI via [GPUI Kit](https://github.com/longbridge/gpui-kit) (`turboui-gpui`). Reference/test: software (`turboui-raster`). Android, iOS, web: not started |

## Try it

```bash
cargo test                                            # layout tests + golden-image tests, no GPU needed
cargo run -p turboui-demo -- --png demo.png           # render the sample screen to a picture
cargo run -p turboui-demo --features gpui             # open it in a real GPUI window (needs a GPU / desktop session)

# check the real GPUI renderer against the reference renderer, no display needed (software Vulkan is fine):
VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json cargo test -p turboui-demo --features headless
```

Needs Rust 1.88 or newer (GPUI Kit uses the 2024 edition); `rust-toolchain.toml` pins 1.97.0, which `rustup` installs automatically. Linux build also needs `sudo apt install libxkbcommon-x11-dev libwayland-dev libfontconfig-dev libxcb1-dev libvulkan-dev` and a Vulkan driver (`mesa-vulkan-drivers` for software rendering) to run the GPUI window.

## Status (v0)

Works: boxes, single-line text, filled-polygon shapes (a small built-in icon set and the TurboMC cube), row/column layout with wrapping grids, padding, gap, grow, alignment, colours, rounded corners, borders, clickable areas by id, TOML themes, a software reference renderer, golden tests, and a GPUI window that re-lays out on resize and on clicks.

The demo (`turboui-demo`) has two Prism-style screens: the server list (toolbar, icon grid with group heading, right-hand action list, status bar) and a server's own window (page list, console, command box, buttons).

Not yet: hover, keyboard and text input, scrolling and clipping (content that overflows spills over its neighbours), long/virtual lists, bitmap images, text wrapping and ellipsis, animation, accessibility, mobile and web backends. Effects such as blur and glass are not in the model. The icons are simple placeholders.

Verified: layout and the reference renderer (golden tests). The real GPUI renderer, drawn offscreen with software Vulkan, matches the reference on both screens (mean difference under 1 out of 255) and a simulated click on a server tile changes the picture exactly as the app state says (tests in `gpui_matches_reference`). The live window itself (real mouse, window chrome) is only checked by hand on a desktop.

See [docs/DESIGN.md](docs/DESIGN.md) for the rules the project follows.
