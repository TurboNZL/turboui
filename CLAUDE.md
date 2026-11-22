# turboui: notes for the AI

Read `README.md` and `docs/DESIGN.md` first. Rules that matter:
- `turboui-core` must never depend on a renderer. Backends only draw a `Scene`.
- A change that alters rendering must update `goldens/` (`TURBOUI_BLESS=1 cargo test`) and you must look at the new PNGs.
- Say plainly what was and wasn't verified. The GPUI window cannot be pixel-checked in the cloud sandbox (software Vulkan output never reaches Xvfb), so a human must look at it.
- Part of TurboNZL's TurboSuite; see TurboNZL/turboengine for conventions (MIT licence, `main` branch, no automatic updates).
