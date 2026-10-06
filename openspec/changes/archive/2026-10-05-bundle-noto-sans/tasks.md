# Tasks

## 1. Vendor the font set

- [x] 1.1 Download the Noto Sans static instances (`Regular`, `Medium`, `SemiBold`, `Bold`, `Italic`, `MediumItalic`, `SemiBoldItalic`, `BoldItalic`) from `notofonts/noto-fonts` (`hinted/ttf/NotoSans/`) at commit `ffebf8c1ee449e544955a7e813c54f9b73848eac` into `assets/fonts/`; verify each parses as TrueType.
- [x] 1.2 Add the eight `fonts/NotoSans-*.ttf` files to `assets/pictura.qrc` under the `/` prefix; verify `tst_fonts` finds each `:/fonts/…` resource.
- [x] 1.3 Add `LICENSES/NotoSans-OFL.txt` (the bundled faces' SIL Open Font License 1.1 text) and record the source, version, and license in `assets/PROVENANCE.md`.

## 2. Register and pin the application font

- [x] 2.1 Add `crates/pictura-app/cpp/fonts.{h,cpp}` with `registerBundledFonts()` (idempotent `addApplicationFont` for every face), `bundledUiFont()` (`Noto Sans`, `SansSerif` hint, `setPixelSize(kBundledUiFontPx)`), and `applyBundledUiFont()`.
- [x] 2.2 Add `fonts.cpp`/`fonts.h` to the `pictura_shell` target in `CMakeLists.txt`.
- [x] 2.3 Call `pictura::applyBundledUiFont()` in `main.cpp` immediately after the `QApplication` is constructed and before any widget; verify the pinned 12 px size keeps the tab (`baseFontPx - 2`) and footer derivations correct.

## 3. Notices

- [x] 3.1 Add a "Font assets" section to `scripts/third-party-licenses.py` and regenerate `THIRD-PARTY-LICENSES`.

## 4. Tests

- [x] 4.1 Add `crates/pictura-app/cpp/tests/tst_fonts.cpp` (registered in `tests/CMakeLists.txt`) covering: bundled faces exist, the family exposes `Regular`/`Medium`/`SemiBold`/`Bold`, `QFont::Medium` resolves to `Medium` (`QFontInfo`), italic semibold resolves, and the application font is `Noto Sans` at 12 px.
- [x] 4.2 Add a case asserting `QFontDatabase::addApplicationFont(":/fonts/NotoSans-Medium.ttf")` returns an id whose `applicationFontFamilies` contains `Noto Sans` (proves the resource loads, independent of any host install).

## 5. Verification

- [x] 5.1 Build with `cmake --build build --parallel` and run `ctest --test-dir build -R '^tst_' --output-on-failure`.
- [x] 5.2 Run `./build/pictura --headless --self-test` (the app path that installs the font) and confirm the summary has zero failures.
- [x] 5.3 Run `bash scripts/verify-fast.sh` (fmt, clippy, test-report, file-size, guard, openspec) with `TASK_ALLOWS_DOCS` set for the `docs/dev/STATE.md` update — `TOTAL 2844 passed · 18 skipped · 0 failed`, guard OK, openspec 185 passed.
- [x] 5.4 Confirm the medium weight resolves to the bundled face (`tst_fonts` `QFontInfo` check; self-test runs the app path) and record the resource cost: `assets/fonts/` is 4.4 MB raw, `qrc_pictura.cpp.o` is 4.77 MB, summed in every binary that links `pictura_assets`.

## 6. Tab chrome weight

- [x] 6.1 Raise the tab labels to Bold: `font-weight: 700` in `QTabBar#panelTabBar::tab` and `QTabBar#documentTabBar::tab` (`theme.cpp`), `QFont::Bold` on the panel tab bar (`panels/panel_group.cpp`), and update the `lss_tab_weight` self-test assertion to `font-weight: 700`.
- [x] 6.2 Rebuild and confirm the tab weight is now visibly distinct, and rerun the verification gates — 28/28 Qt suites, self-test 475/475, `verify-fast.sh` `TOTAL 2845 passed · 18 skipped · 0 failed`.
- [x] 6.3 Extract the tab-bar chrome font into `applyTabBarFont()` (`fonts.{h,cpp}`) and call it for the document tab bar (`frame.cpp`) as well as the panel tab bar (`panels/panel_group.cpp`); add a `tst_fonts` case asserting both bars' resolved font is the bundled family at Bold / app px − 2.

