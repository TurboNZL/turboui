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

## Themes

A look is a **palette** (colours) plus a **style** (shapes and materials), the same split turbothemes uses.
- `themes/palettes/*.toml` are generated from the real turbothemes palettes by `tools/import-palettes.mjs`, which runs turboengine's own colour derivation, so the colours match the web themes exactly. Included: macOS Dark, macOS, Nord, Dracula (the importer can do any of the 67: `node tools/import-palettes.mjs <turboengine>/themes themes/palettes <id>...`).
- `themes/styles/flat.toml`: flat, Breeze-like. `themes/styles/liquid.toml`: **macOS Liquid Glass**, ported from turbothemes' `liquid` style.

Try one: `cargo run -p turboui-demo -- --theme nord/flat --png out.png` (any palette with either style, e.g. `macos-dark/liquid`).

**Liquid Glass here:** translucent surfaces that blur and bend what is behind them (Snell's-law lens at the rim), saturate and tint it, add a diagonal sheen and a rim lit from the top left, on top of soft ambient colour light. The software renderer does it per pixel. GPUI cannot blur what is behind a surface, so the GPUI backend asks the software renderer for the finished glass and paints it as a picture (a surface's picture is redone only when something under it changes, so selecting items elsewhere costs nothing). Not done: colour fringing at the rim, the springy press and open animations, glow under the pointer, the Clear variant, and the window-inactive fade.

## Status (v0)

Works: boxes, single-line text, filled-polygon shapes (a small built-in icon set and the TurboMC cube), row/column layout with wrapping grids, padding, gap, grow, alignment, colours, rounded corners, borders, clickable areas by id, TOML palettes and styles, Liquid Glass and ambient light, a software reference renderer, golden tests, and a GPUI window that re-lays out on resize and on clicks.

The demo (`turboui-demo`) has two Prism-style screens: the server list (toolbar, icon grid with group heading, right-hand action list, status bar) and a server's own window (page list, console, command box, buttons). In a glass style the toolbar and action list float as glass over ambient light.

Also works: scroll areas that clip their content, overlays (dialogs over a dimmed scrim), hover, wheel and keyboard events (the app gets `Event`s and changes its own state), and a first widget set (buttons, text field with caret, checkbox, switch, progress bar, dialog).

Not yet: text selection and IME in text fields, virtual lists for very long content, bitmap images, text wrapping and ellipsis, animation, accessibility, mobile and web backends. The icons are simple placeholders.

Verified: layout and the reference renderer (golden tests for seven themed screens). The real GPUI renderer, drawn offscreen with software Vulkan, matches the reference within about 1 out of 255 on average for flat and Liquid Glass screens, light and dark, and a simulated click on a server tile changes the picture exactly as the app state says (tests in `gpui_matches_reference`). That comparison caught a real bug (images were premultiplied twice, which dimmed the ambient light). The live window itself (real mouse, window chrome, glass cost while animating) is only checked by hand on a desktop.

See [docs/DESIGN.md](docs/DESIGN.md) for the rules the project follows.
