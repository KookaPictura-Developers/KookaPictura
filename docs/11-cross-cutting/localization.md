# Localization

- **Spec ID:** `XC-001`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `No` — CS6 localizes the same way as CS5: language is chosen by the installer/regional build, and the string tables ship per locale. CS6 adds no in-app language switcher; the Arabic/Hebrew "Middle Eastern" engine and the East Asian text engines continue the CS5 model. Localization changes between CS5 and CS6 are string-content changes, not architecture.
- **Depends on:** `01-architecture/qt6-ui-design.md`, `01-architecture/rust-qt-interop.md`, `02-ui-ux/preferences.md` (`UI-010`), `02-ui-ux/accessibility.md` (`UI-012`), `02-ui-ux/panels/character-and-paragraph.md`, `11-cross-cutting/preference-storage.md` (`XC-002`).

> Rust crate/module names and Qt components below are **design proposals**. CS6's exact string tables, its full shipped language list, and its RTL UI layout are only partly documented; claims marked *(secondary)* or *(inferred)* are collected under `## Open questions` and must not be treated as fact.

## CS6 behavior

### Shipped languages

CS6 launched (May 2012) in **English, French, German, Spanish, Danish, Dutch, Italian, Finnish, Norwegian, Swedish, Portuguese, and Japanese**, with  *(secondary: ProDesignTools launch report)*. Each language is a separate regional installer/build; the user's UI language is fixed by which build was installed. There is no documented `Edit > Preferences` language selector in CS6.

### How CS6 changes language

CS6 stores the installed UI language under its program directory as locale bundles. Community documentation for changing a CS6 interface language without reinstalling describes editing files under `…/Adobe Photoshop CS6 (64 Bit)/Locales/<locale>/Support Files/`, in particular renaming `tw10428.dat` (the localized string table) to force the application back to English *(secondary: DownloadSource.net)*. This establishes two facts about the CS6 architecture without giving its internal format: (a) the UI language lives in per-locale resource data, not in preferences, and (b) the runtime loads one language at a time. Kooka Pictura does not need bit-compatibility with that data (see `00-overview/licensing-and-independent-creation.md`).

### Locale-sensitive behavior that *is* documented

The CS6 Help reference documents several controls whose behavior is locale-dependent, and those are the parity contract that matters:

- **Interface → Text → Show Font Names In English** — displays Asian font names in English instead of the localized name.
- **Type → East Asian / Middle Eastern / Choose Text Engine Options** — selects a text layout engine. The Middle Eastern engine is what enables Arabic and Hebrew.
- **Type → Language Options → Middle Eastern Features** — exposes the Middle Eastern submenu (after enabling the Middle Eastern engine).
- **Units & Rulers** — ruler/type units and the **Point/Pica Size** choice (PostScript 72 pt/in vs. Traditional 72.27 pt/in). The Info panel's units follow the ruler unit.
- **Guides, Grid & Slices / gridline values** accept `%` as well as absolute units.

CS6's own number display is not CLDR- or ICU-driven in any documented way: rulers show a fixed unit set (px, in, cm, mm, pt, pica, %, columns) and values are formatted with the application's own routines, not necessarily with locale decimal/group separators *(inferred from the Help reference; not stated)*.

### Right-to-left (Arabic / Hebrew)

CS6 shipped a **Middle Eastern (English/Arabic-enabled)** regional version and a **North African French** version. In those builds the default typing font is set to the installation language (for example `Adobe Arabic` for the English/Arabic build). Users enable RTL/middle-eastern type through **Preferences → Type → Choose Text Engine Options → Middle Eastern** and then **Type → Language Options → Middle Eastern Features**, which adds RTL text direction options *(secondary: CS6 Arabic/Hebrew-type manual mirror and Adobe-community/superuser threads; the current Adobe Help page describes the modern Photoshop feature and is not CS6 evidence)*. The modern Adobe language page states that Arabic and Hebrew are supported in the Middle Eastern and North African French regional versions with correct right-to-left handling; this is consistent with, but not proof of, CS6's exact behavior.

### What "localization" means here

Three separable concerns must not be conflated:

1. **UI chrome strings** — menus, dialogs, panels, tool names, tooltips, messages.
2. **Locale-aware data formatting** — numbers, units, dates, plural forms, sort order.
3. **Text editing/layout** — RTL base direction, bidi runs, CJK line breaking, glyph shaping.

CS6's user-visible contract is (1) plus the Type/Units preferences in (3). Concern (2) is barely documented and will be a Linux-native improvement, marked as a superset of parity.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| Installer / regional build | Install-time choice | — | CS6 UI language is fixed at install; no in-app switch. |
| `Locales/<locale>/Support Files/` | Program data | — | Per-locale resource data (community-documented); not a UI surface. |
| `Edit > Preferences > Interface > Show Font Names In English` | Preference | — | Locale-sensitive; moved/likely under Type in CS6. |
| `Edit > Preferences > Type > Choose Text Engine Options` | Preference | — | System / East Asian / Middle Eastern. |
| `Edit > Preferences > Units & Rulers` | Preference | — | Units and Point/Pica size; feeds rulers and Info panel. |
| `Type > Language Options > Middle Eastern Features` | Submenu | — | Visible only when the Middle Eastern engine is enabled. |
| `Type > Language Options` (other) | Submenu | — | East Asian options when that engine is enabled. |
| `Character` / `Paragraph` panels | Panel | — | Expose CJK/ME controls when the engine is on. |
| Rulers / Info panel | Readout | — | Follow the Units & Rulers selection. |
| `Edit > Keyboard Shortcuts` | Dialog | — | Shortcut labels are localized; the editor is a translation consumer. |

> Proposal (not CS6): a `Preferences → Interface → Language` selector and a `LANG`/`LC_MESSAGES`-driven default, with a restart prompt. CS6 has no equivalent in-app control; the install-time-only model is a parity constraint, not a design requirement for Kooka Pictura.

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| UI language | enum | system/install locale | CS6: per-install build; Kooka Pictura proposed: any shipped catalog | CS6 count: 12 at launch, ≤24 total *(secondary)*. |
| Text engine | enum | System | System / East Asian / Middle Eastern | ME required for Arabic/Hebrew. |
| Show Font Names In English | bool | off | on / off | CS6 default locale-dependent. |
| Ruler units | enum | Inches (US) / cm (metric) | px, in, cm, mm, pt, pica, %, columns | Follows `UI-010`. |
| Type units | enum | Points | px, in, cm, mm, pt, pica | Type panel size field. |
| Point/Pica size | enum | PostScript | 72 / 72.27 pt-per-inch | Affects pica↔point math. |
| Number/unit locale | derived | `QLocale` default | any BCP-47 locale | Proposed; CS6 behavior unverified. |
| Layout direction | enum | from locale | LTR / RTL | Proposed; CS6 ME builds mirror UI *(inferred)*. |
| Plural forms | n/a | locale rules | CLDR plural categories | Proposed; not a CS6 field. |

## Algorithms & pipeline

### Translation catalog (proposed)

A single **key-based catalog** is the source of truth, not the English string. Each user-visible string has a stable ID (`prefs.performance.memory.label`, `error.scratch.full`), so a wording change does not orphan a translation. Two consumers resolve keys:

- the **Qt shell**, via `QTranslator`/`.qm`;
- the **Rust core**, via a Fluent (`fluent-bundle`) or gettext catalog.

Keeping two runtime resolvers risks drift, so the proposal is one extraction/build pipeline: authors write message sources once (a Fluent `.ftl` resource is the recommended authoring format), and a build step emits both the Qt `.ts` (context = generated class/key namespace) and the Rust runtime bundle. Alternatively, if `.ts` is the authoring format, a Rust `fluent` bundle can be generated from the same keys. Either direction is a proposal; the invariant is **one key namespace, one extraction step**.

### Rust catalogs

| Option | Crate (verified) | Plurals | Interpolation | Direction safety | Fit |
|---|---|---|---|---|---|
| Fluent | `fluent` 0.17 / `fluent-bundle` 0.16 | Yes (`intl_pluralrules`) | Message references, variables, functions | Yes — `format_pattern` wraps substituted values in FSI/PDI isolates so a value's direction does not leak into the sentence | Recommended for core messages. |
| gettext | `gettext-rs` 0.8 | Yes (`ngettext`, CLDR plural forms) | `printf`-style | No built-in bidi isolation | Familiar on Linux; LGPL static-link caveat unless `gettext-system`. |

Recommendation: **Fluent for the Rust core** (typed variables, plural selection, bidi-safe interpolation, pure-Rust, no LGPL linkage concern) with `unic-langid` for language negotiation; fall back to a Qt-only path if the generated-bundle pipeline proves fragile.

### Qt catalogs

Qt uses `.ts` source and compiled `.qm` binaries loaded by `QTranslator`, installed with `QCoreApplication::installTranslator`. `QTranslator::load(QLocale, filename, prefix, directory, suffix)` resolves UI languages through `QLocale::uiLanguages()` and tries `fr_CA → fr → <none>` fallbacks, which matches how CS6-era applications pick the closest shipped locale. Widget strings come from `tr()`/`QCoreApplication::translate()`; contexts are class names or generated key namespaces. `.qm` is a binary format that the Qt docs explicitly warn may crash the application if malformed: **only ship trusted, signed catalogs** and treat downloaded language packs as untrusted input (see `XC-005`).

### Locale-aware formatting

The proposal is that **Qt owns presentation formatting** for anything the user reads. `QLocale` provides `toString` for integers/floats, `formattedDataSize` (IEC/SI/traditional), `measurementSystem`, `decimalPoint`, `groupSeparator`, `percent`, `zeroDigit`, `createSeparatedList`, and `textDirection()`. Rust passes canonical values across the bridge and never formats user-facing numerals itself:

- Numeric fields: parse/emit via `QLocale::toDouble`/`toString`; **internal storage is always canonical** (C locale, `.` decimal), and the locale decimal separator is applied only at the widget boundary. This prevents a comma-decimal locale from corrupting stored values.
- Units: a `Quantity { value: f64, unit: Unit }` crosses the FFI; Qt renders it with `QLocale`. Ruler/Info/Type values use this.
- Plurals and lists: plural selection happens in the catalog runtime (Fluent or gettext), not with hand-written "s" appending.
- Dates/times (file info, History Log, logs): `QLocale` formatting; stored timestamps remain UTC/Unix.

### Right-to-left and bidi

- Application direction: `QLocale::textDirection()` → `QGuiApplication::setLayoutDirection(Qt::RightToLeft)`; mirrored layouts use `QWidget::setLayoutDirection`. Qt widgets and Qt Quick handle mirroring.
- Text layout: the Qt text engine implements Unicode Annex #9 bidi and shaping; canvas type laid out with `QTextLayout` respects the base direction. Arabic/Hebrew shaping and ligatures come from the font engine, so **font fallback must include a script-capable font** (the ME builds defaulted to `Adobe Arabic`).
- Core messages: Fluent's FSI/PDI isolation keeps an LTR inserted value (a filename, a number) from reversing inside an RTL sentence.
- Canvas overlays, rulers, and numeric HUDs should be explicitly LTR even in an RTL UI, matching how coordinate/measurement readouts behave in graphics tools.

### Pipeline

```text
.ftl / .ts sources
   → lupdate / fluent extraction  (keys, contexts, plural forms)
   → translators (Poedit / Crowdin / Qt Linguist)
   → build: lrelease → .qm   +   fluent bundle (or generated .ftl)
   → packaged in app resources (:/i18n) and/or $XDG_DATA_DIRS/kooka-pictura/i18n
   → runtime: QTranslator installed before widgets; Rust LocaleService selects bundle
   → locale negotiation from LANG/LC_MESSAGES, overridable in Preferences (proposed)
```

Behavioral parity note: CS6 resolves exactly one installed language with no negotiation; the negotiation/fallback chain and the runtime language switch are Kooka Pictura additions, not parity requirements.

## Rust module mapping

- `pictura-i18n::Locale` — BCP-47 wrapper over `unic-langid`; negotiation and fallback.
- `pictura-i18n::MessageKey` — compile-time key type generated from the catalog; prevents raw-string drift.
- `pictura-i18n::Catalog` — `FluentBundle` set for a locale; `format(key, args) -> String`.
- `pictura-i18n::LocaleService` — holds the active locale, exposes `format`, emits `LocaleChanged`; consumed by core error/progress/dialog code.
- `pictura-i18n::Quantity` / `format_quantity` — unit-carrying value; Rust emits canonical numbers and defers rendering (Qt) or uses a minimal C-locale fallback for logs.
- `pictura-prefs::InterfacePrefs` — stores UI language, text engine, and "show font names in English".
- Types crossing the Rust↔Qt boundary: `Locale`, `Quantity`, `MessageKey` (as a string), formatted `String`s. No Qt types in Rust.

## Qt6 component mapping

- `QTranslator` — loaded via `load(QLocale, …)`; `QCoreApplication::installTranslator`; multiple catalogs searched newest-first.
- `QCoreApplication::translate` / `QObject::tr` — widget strings; `QT_TR_NOOP`/`QT_TRANSLATE_NOOP` for non-widget strings.
- `QLocale` — number/unit/date formatting, `uiLanguages()`, `textDirection()`, `measurementSystem()`.
- `QGuiApplication::setLayoutDirection` / `QWidget::setLayoutDirection` — RTL mirroring.
- `QTextLayout` / `QTextDocument` — canvas and rich-text bidi/shaping.
- `QLocale` + `QFontDatabase` — script-capable font fallback for Arabic/Hebrew/CJK.
- `LocaleController` (`QObject`, cxx-qt) — bridges `LocaleService` to Q_PROPERTYs; triggers `retranslateUi()` on change.
- Qt Linguist toolchain: `lupdate`, `lrelease`, `Qt Linguist` (authoring).

## Data-model impact

- **No PSD/XMP schema change.** Localization is presentation-layer; it never serializes into documents.
- **Text layers store Unicode and a base direction.** A text layer needs a `base_direction: Ltr | Rtl` attribute; PSD text-engine data carries direction/justification fields, so this must round-trip through the document model (`01-architecture/document-model.md`). Exact PSD keys are unverified.
- **Preferences gain** `ui_language`, `text_engine`, `show_font_names_in_english`, and formatting preference fields; they follow `XC-002` migration rules.
- **History Log and telemetry** strings are localized at display time; their stored records stay canonical/English-keyed so logs are comparable across locales.
- **Undo:** changing the UI language is a preference edit, not a document command; it is not undoable via the History panel (`UI-010`).

## Edge cases

- **Untranslated/partial catalog:** fall back key-by-key to the source language; never show a raw key to the user.
- **Malformed `.qm`:** treat as untrusted; a bad catalog can crash the process — validate/sign language packs and fail closed (`XC-005`).
- **Plural correctness:** languages with multiple plural forms (Russian, Czech, Arabic) must use catalog plural rules, not `n == 1`.
- **Decimal separator in numeric fields:** a comma-decimal locale must not write `1,5` into canonical storage; parse via `QLocale`, store canonical.
- **RTL + mixed content:** numbers, filenames, and version strings stay LTR; test bidi ordering explicitly.
- **Font fallback:** missing Arabic/Hebrew/CJK fonts must substitute and warn, not render tofu silently; `Missing Glyph Protection` is a CS6 behavior to honor.
- **CJK line breaking:** rely on the Qt text engine for word/syllable boundaries rather than manual splitting.
- **Locale vs. system language mismatch:** UI language and number-format locale can differ; keep them as separate settings.
- **Installed vs. portable resources:** Flatpak confines the app; catalogs must be inside the app/runtime or read via XDG data dirs (`XC-002`, `XC-005`).
- **Accessible names** must come from the same localized catalog so screen readers and menus agree (`UI-012`).
- **Log timestamps** stay UTC and ISO-8601 regardless of display locale.
- **Restart-required switch:** a runtime language change retranslates live widgets; some strings (already-built models) need a full restart — document which.

## Parity acceptance criteria

1. Given a CS6-supported locale catalog is installed, launching with `LANG` set to that locale shows menus, dialogs, and tool names in that language; no raw keys appear.
2. Given a string absent from the active catalog, the source-language string is shown and no key leaks.
3. Given `Preferences → Type → Choose Text Engine Options = Middle Eastern` and a Hebrew/Arabic font, typing in a Type layer produces correctly shaped, right-to-left text, and `Type → Language Options → Middle Eastern Features` is present.
4. Given an RTL locale, the application frame mirrors (panels/menus) while canvas coordinate readouts remain LTR and numerically identical to the LTR build.
5. Given a locale with a comma decimal separator, entering `1,5` in a numeric field stores `1.5` canonically and redisplaying shows `1,5`; reopening the document yields the same value.
6. Given a plural-sensitive message (e.g. "N layers selected") in a multi-plural language, `n = 1`, `2`, and `5` select the correct CLDR plural category.
7. Given a ruler unit change, the ruler and Info panel labels and values update consistently with `UI-010`.
8. Given an untrusted/malformed translation file, the app refuses to load it, logs the failure, and continues in the source language.
9. Given `Show Font Names In English = on`, Asian font names display in English regardless of UI language.

## Sources

Fetched for this document:

- `https://doc.qt.io/qt-6/internationalization.html` — Qt 6 internationalization: `QTranslator`, `QLocale`, `QCollator`, encodings, RTL/bidi (Unicode Annex #9), CJK line breaking, Linux UTF-8 defaults.
- `https://doc.qt.io/qt-6/qtranslator.html` — `QTranslator::load(QLocale, …)`, `uiLanguages` fallback order (`fr_CA → fr → …`), multiple translators searched newest-first, and the explicit security warning that malformed `.qm` files may crash the app.
- `https://doc.qt.io/qt-6/qlocale.html` — `QLocale` number/date formatting, `formattedDataSize` with IEC/SI formats, `measurementSystem`, `decimalPoint`, `groupSeparator`, `percent`, `zeroDigit`, `createSeparatedList`, `textDirection`, `uiLanguages`, CLDR v48.2 basis.
- `https://docs.rs/fluent/latest/fluent/` — Fluent Rust umbrella crate (`FluentBundle`, `FluentResource`, `FluentArgs`, `FluentValue`), plural/bidi-safe example using FSI/PDI isolation marks.
- `https://docs.rs/fluent-bundle/latest/fluent_bundle/` — mid-level Fluent runtime; deps include `intl_pluralrules` and `fluent-langneg`; `format_pattern` semantics.
- `https://docs.rs/gettext-rs/latest/gettextrs/` — `gettext`/`ngettext`/`pgettext`/`npgettext`, PO/MO workflow, `gettext-system` vs statically linked GNU gettext (LGPL caveat).
- `https://prodesigntools.com/adobe-releases-cs6-shipping-free-trials.html` — *(secondary)* CS6 launch announcement: initial 12 shipping languages and the "up to 24" total including Chinese, Korean, Russian.
- `https://www.downloadsource.net/how-to-change-photoshop-cs6-language-to-english-without-reinstalling/n/9320` — *(secondary)* CS6 per-locale data under `Locales/<locale>/Support Files/` and the `tw10428.dat` string-table rename technique; evidence that CS6 has no in-app language switch.
- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` — **primary CS6 reference; not re-fetched in this pass** (large). Cited via the repo's existing `02-ui-ux/preferences.md` extract for Units & Rulers, Type engine, and Show Font Names In English.

Found via search, **not fetched** (treated as leads only):

- `https://helpx.adobe.com/photoshop/desktop/get-started/technical-requirements-installation/photoshop-language-availability.html` — returned HTTP 403. Modern Photoshop language table (Arabic/Hebrew regional versions); **not CS6 evidence**.
- `https://manualzz.com/doc/o/mw0zh/adobe-photoshop-cs6-user-manual-arabic-and-hebrew-type--cs6-` — returned HTTP 403. Mirror of the CS6 "Arabic and Hebrew type" Help page; would establish the exact ME preference path.
- Adobe-community and Super User threads on the CS6 Middle Eastern text engine (`community.adobe.com/.../support-arabic-typing-in-cs6/...`, `superuser.com/questions/993923`) — not fetched; consistent with the ME engine path above.

Internal cross-references (not sources): `docs/02-ui-ux/preferences.md` (`UI-010`), `docs/02-ui-ux/accessibility.md` (`UI-012`), `docs/01-architecture/system-architecture.md` (`ARCH-001`).

## Open questions

- **Exact CS6 shipped language list and per-version availability.** The 12-launch/24-total figure is secondary. *Resolve:* Adobe CS6 release notes or the archived CS6 language page.
- **Did CS6 mirror the entire UI in RTL, or only text direction in the Type tool?** No Adobe document found. *Resolve:* run the CS6 Middle Eastern build and inspect menu/panel mirroring.
- **CS6 locale number/date formatting.** Whether CS6 uses OS locale formatting for rulers/Info or fixed formatting is unstated. *Resolve:* compare a CS6 run under `de_DE` vs `en_US`.
- **`tw10428.dat` format.** Unknown and possibly legally encumbered; Kooka Pictura should not read it. *Resolve:* legal review (`00-overview/licensing-and-independent-creation.md`) and a independent-creation decision; default is to ignore it.
- **Fluent authoring direction.** Whether `.ftl` or `.ts` is the authoring source and how the generated Qt/Rust catalogs are kept in sync is unresolved. *Resolve:* a prototype of the extraction pipeline.
- **Runtime language switch vs. install-time only.** CS6 requires a reinstall/edit; Kooka Pictura proposes a restart-required switch. *Resolve:* product decision.
- **Fluent number/date builtins.** Whether Fluent's builtin functions cover all `QLocale` formatting needs, or whether Qt must own all numeric formatting, is unconfirmed. *Resolve:* inspect `fluent_bundle::builtins`.
- **Do text-layer RTL base direction and ME attributes survive PSD round-trip byte-exactly?** *Resolve:* test against CS6-generated ME PSD files and the PSD spec text-engine keys.
