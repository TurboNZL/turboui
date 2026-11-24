# turboui: notes for the AI

Read `README.md` and `docs/DESIGN.md` first. Rules that matter:
- `turboui-core` must never depend on a renderer. Backends only draw a `Scene`.
- A change that alters rendering must update `goldens/` (`TURBOUI_BLESS=1 cargo test`) and you must look at the new PNGs.
- Say plainly what was and wasn't verified. A live GPUI window cannot be pixel-checked in the cloud sandbox (its output never reaches Xvfb), but the offscreen renderer can: `VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json cargo test -p turboui-demo --features headless` (apt: `mesa-vulkan-drivers`), or `turboui-demo --gpui-png out.png`. Look at the PNG after changing the GPUI backend.
- Part of TurboNZL's TurboSuite; see TurboNZL/turboengine for conventions (MIT licence, `main` branch, no automatic updates).
