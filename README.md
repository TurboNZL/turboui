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

Works: boxes and single-line text, row/column layout, padding, gap, grow, alignment, colours, rounded corners, borders, TOML themes, a software renderer, golden tests, a GPUI window that resizes.

Not yet: clicks, hover, keyboard and text input, scrolling and lists, images, text wrapping, animation, accessibility, mobile and web backends. Effects such as blur and glass are not in the model yet.

Verified: layout and the reference renderer (tests). The real GPUI renderer, drawn offscreen with software Vulkan, matches the reference on the sample screen (mean difference about 1 out of 255, test `gpui_matches_reference`). The live window itself (input, resizing, window chrome) has not been run on a real desktop here.

See [docs/DESIGN.md](docs/DESIGN.md) for the rules the project follows.
