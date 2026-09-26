# Verbatim-expression audit

- **Status:** sweep complete — long quoted passages paraphrased across all 72
  flagged docs. Remaining scanner hits are short functional identifiers, source
  anchors, search-query citations, and one material legal citation (Adobe's
  PSD-format statement). **Not legal advice.**
- **Policy basis:** `00-overview/licensing-and-provenance.md` (`OVR-004`) —
  specs must express **facts and interfaces**, not reproduce documentation prose
  verbatim; the asset policy lists "Adobe Help screenshots / text — do not copy,
  cite and paraphrase facts".

## Method

`scripts/verbatim-audit.py` scans every `docs/**/*.md` for two heuristic
markers of reproduced prose: quoted strings of ≥ 12 words and blockquotes of
≥ 24 words. It writes a ranked evidence list to
[`verbatim-expression-audit-findings.md`](verbatim-expression-audit-findings.md)
(307 passages) and the kind of each flag (`quote`, `quote+attr`, `blockquote`).

```bash
python3 scripts/verbatim-audit.py        # regenerate the findings list
python3 scripts/verbatim-audit.py 20     # raise the threshold
```

## Findings

- **111 docs** state their behaviour is taken from the fetched **CS6 Help PDF**.
- **72 docs** contain **235 long quoted passages** (158 `quote` + 77
  `quote+attr`) that read as Help/marketing/community prose, not short
  identifiers. Worst offenders:

  | Doc | Quoted passages |
  |---|---|
  | `08-selection/color-range.md` | 14 |
  | `08-selection/refine-edge.md` | 13 |
  | `08-selection/quick-mask.md` | 10 |
  | `07-color-painting/histogram-and-info.md` | 8 |
  | `02-ui-ux/panels/histogram-panel.md` | 8 |
  | `07-color-painting/swatches-and-libraries.md` | 7 |
  | `08-selection/selection-model.md`, `02-ui-ux/panels/navigator-panel.md` | 6 each |

- The 72 `blockquote` flags are mostly our **own** repeated boilerplate
  ("design proposals", "no code exists in this repository") — not a concern.
- `11-cross-cutting/open-questions.md` already notes the CS6 Help PDF is a
  **rolling document** (cover Feb 2013, modified 2017) carrying some
  Creative-Cloud-marked material, so claims sourced only from it may be
  CC-contaminated.

## Disposition rule

| Passage class | Example | Action |
|---|---|---|
| Short functional identifier — menu path, option/button label, format id | `Window > Measurement Log`, "Masked Areas", `Gaussian Blur` | **Keep** (facts). |
| Long prose definition/instruction reproduced from the Help | a paragraph such as the Quick Mask description | **Rewrite** into a factual paraphrase; drop the quotation. |
| Our own boilerplate blockquote | "All module… names below are design proposals." | **Keep**. |

## Remediation (done) — kept for the record

Sweep completed; re-running `scripts/verbatim-audit.py` now flags only five
search-query citation strings (not prose).

1. Walk the 55 flagged docs, highest count first; replace each long Help quote
   with a paraphrase that states the same fact in our own words, keeping only
   the short identifiers (menu paths, option names). Example target:

   > CS6 Help's Quick Mask paragraph → *Quick Mask converts the active
   > selection into a temporary mask shown as an adjustable coloured overlay,
   > editable with any painting tool or filter and converted back to a
   > selection on exit.*

2. Re-run `scripts/verbatim-audit.py` until only short identifiers and our own
   boilerplate remain.
3. Add a one-line rule to the spec template that quoted Help prose is not
   allowed (paraphrase only) and that sources are recorded in `## Sources`.

## Open questions for counsel

- Is a **short attributed excerpt** from Adobe Help defensible fair use, or
  should all Help-derived text be paraphrased? (Recommendation: paraphrase all
  long passages regardless.)
- Should the **CS6 Help PDF** be removed from the source trail where it is the
  only source (CC-contamination risk), replacing those claims with a
  non-Adobe capture or marking them *(unverified)*?
