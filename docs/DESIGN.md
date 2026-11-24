# TurboUI design rules

1. **One place decides layout.** `turboui-core` computes every position and text size. Backends receive a flat list of rectangles and text and only draw them. A backend must never re-layout; that is how screens end up different per platform.
2. **Small building blocks.** Every new block must be written in every backend, so add one only when it cannot be built from the existing ones.
3. **Design is data.** Colours, sizes and radii come from theme files (`themes/palettes`, `themes/styles`), by name, using turbothemes' token names. Palettes are generated from turbothemes, never edited by hand. A missing token stops the program with a clear message instead of showing the wrong colour.
4. **Same font everywhere.** The bundled fonts are used by every backend, so text widths agree. (Currently DejaVu; swappable.)
5. **Golden images.** `goldens/` holds the expected picture of each test screen. A change that alters a picture fails the test until it is reviewed and re-saved with `TURBOUI_BLESS=1 cargo test`.
6. **Two backends before trusting the model.** The reference renderer is the second backend, and a test (`gpui_matches_reference`) checks that the real GPUI output agrees with it. The next real one (Android or web) will show where the model is too GPUI-shaped.
7. **Desktop widgets come from GPUI Kit** (`gpui-kit`, Apache-2.0) behind the desktop backend. Apps never import it; they describe screens with TurboUI blocks.

## Materials that a backend cannot draw
Some looks need effects a toolkit lacks (GPUI cannot blur what is behind a surface). The model describes the effect (`Glass`); the reference renderer can always do it; a backend either does it natively or paints the reference renderer's result as a picture. Both give the same screen, and the picture route costs CPU when the screen changes, so animation-heavy glass may need a native implementation later.

## Next
Scrolling and clipping, hover and focus, scroll and virtual lists, images, text input (mapped onto GPUI Kit's list and input), then a second real backend.
