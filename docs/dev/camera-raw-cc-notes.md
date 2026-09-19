# Camera Raw in PSD, CC notes

Working note (`docs/dev/`). Findings from two Photoshop-produced PSD fixtures.
The roadmap is `docs/dev/psd-support-roadmap.md`; the behavioral specs are
`docs/06-filters/camera-raw-filter.md` (`FILT-100`) and
`docs/10-workflow-io/camera-raw-workflow.md` (`WF-012`).

Scope decision (2026-09-19): support the **a reference build** model of Camera Raw in PSD.
CS6 ACR is not available to make reference files, so the CS6 raw-as-Smart-Object
`crs:` path is documented but is not the acceptance target. The CC Camera Raw
Filter is.

This is an extension, not a change to CS6 parity. `FILT-100` still records that
`Filter > Camera Raw Filter` is post-CS6, so it stays outside the CS6 menu model;
the goal here is only that an Adobe file carrying it survives our open→save.

## Fixtures

Both 512x512, RGB, 8-bit, PSD version 1, three composite channels.

- `assets/test_with_smart_object01.psd`: one embedded smart object, source an
  embedded PSB (252x233, 4 channels). No filter, no Camera Raw settings.
- `assets/test_with_smart_object02.psd`: the same embedded PSB shared by two
  smart-object layers; `Layer 1 copy` carries a Camera Raw Filter smart filter.
  Produced with Photoshop a reference build. Despite the name, there is no `crs:` XMP
  anywhere; the settings are in the filterFX descriptor below.

## Smart object container

Confirmed from fixture 01.

- The layer's `SoLd` descriptor keys: `Idnt` (the uuid), `placed` (a second
  uuid), `PgNm`, `totalPages`, `Crop`, `frameStep`, `duration`, `frameCount`,
  `Annt` (16), `Type` (2), `Trnf` (8 corner floats), `nonAffineTransform`,
  `warp` (`warpStyle`, `warpValue`, `warpPerspective`, `warpPerspectiveOther`,
  `warpRotate`, `bounds`, `uOrder`, `vOrder`), `Sz  ` (`Wdth`, `Hght`),
  `Rslt` (72), `comp` (-1), `compInfo`. Photoshop also writes the legacy `PlLd`.
- The document `lnk2` record for the embedded source: `kind` DATA, `version` 7,
  `uuid` matching `SoLd.Idnt`, `filename` `Layer 1.psb\0`, `filetype` `8BPB`
  (not `8BPS`), `creator` `8BIM`, payload the embedded file bytes, `child_id` a
  single NUL, `mod_time` 0.0, `lock_state` 0, `open_file`
  `{compInfo:{compID:-1, originalCompID:-1}}`.
- Photoshop also writes an empty `lnkE`. `lnkD` and `lnk3` were absent.

## Camera Raw settings, two storage models

| Path | Where settings live | Have a fixture |
|---|---|---|
| Raw opened as Smart Object (CS6) | `crs:` XMP in the embedded raw payload | no |
| Camera Raw Filter as Smart Filter (CC) | `SoLd.filterFX.filterFXList[].Fltr`, `filterID` 2683 | yes, fixture 02 |

The CC filter path also writes the document blocks `FEid` (FilterEffects2) and
`FMsk` (filter mask). The settings descriptor is named `Fltr` and appears once
per filter in `filterFXList`.

## `Fltr` key map

Short keys, grouped by the `FILT-100` tab they drive. Values observed in
fixture 02.

| Group | Keys | Notes |
|---|---|---|
| Mode and process | `CMod`, `Sett`, `PrVe` | `Sett` an enum, `PrVe` observed 101122048 |
| White balance | `WBal`, `Temp`, `Tint`, `AWBV`, `CtoG` | `Temp` -30, `Tint` 12 (the filter uses the rendered-image scale, not Kelvin) |
| Basic PV2012 | `Ex12`, `Cr12`, `Hi12`, `Sh12`, `Wh12`, `Bk12`, `Cl12`, `Vibr` | Exposure -1.15, Contrast 12, Highlights -15, Shadows 10, Whites -18, Blacks 9, Clarity -12, Vibrance 13 |
| Legacy Basic | `BlkB`, `Strt`, `Shrp` | |
| HSL | `RHue`/`RSat`, `GHue`/`GSat`, `BHue`/`BSat` | red, green, blue |
| HSL 8 ranges | `HA_*`, `SA_*`, `LA_*` for `_R _O _Y _G _A _B _P _M` | hue, saturation, luminance |
| Split toning | `STSH`, `STSS`, `STHH`, `STHS`, `STB` | |
| Tone curve | `PC_S`, `PC_D`, `PC_L`, `PC_H`, `PC_1`, `PC_2`, `PC_3`, `Crv`, `CrvR`, `CrvG`, `CrvB` | parametric regions plus point curves as `[in, out, ...]` |
| Detail | `Shrp`, `ShpR`, `ShpD`, `ShpM`, `LNR`, `CNR` | sharpening plus luminance/chroma noise |
| Lens | `LPEn`, `MDis`, `VigA`, `PerV`, `PerH`, `PerR`, `PerS`, `PerA`, `PerU`, `PerX`, `PerY`, `DfPA`, `DPHL`, `DPPH`, `DfGA`, `DPGL`, `DPGH` | profile enable, manual distortion, vignette, perspective, defringe |
| Effects | `GRNA`, `GRNS`, `GRNF`, `PCVA`, `PCVM`, `PCVF`, `PCVR`, `PCVS`, `PCVH` | grain plus post-crop vignette |
| Camera calibration | `CamP`, `CP_D`, `PrVe` | `CamP` Embedded, `CP_D` profile digest |
| CC-only | `Dhze`, `Upri`, `GuUr`, `Rtch`, `REye`, `LCs ` | Dehaze -14; Upright stored as XMP strings |

## Open items

- A 16-bit variant of either fixture, produced later once roadmap G4 depth
  support exists.
- Whether Photoshop regenerates the merged composite or trusts ours.
- The CS6 `crs:` location, only if a CS6 ACR fixture ever appears.

## Reproduction

```bash
PYTHONDONTWRITEBYTECODE=1 python3 - <<'PY'
from psd_tools import PSDImage
from psd_tools.constants import Tag
psd = PSDImage.open("assets/test_with_smart_object02.psd")
for layer in psd.descendants():
    if getattr(layer, "smart_object", None):
        d = layer.tagged_blocks.get_data(Tag.SMART_OBJECT_LAYER_DATA1).data
        fx = d.get(b"filterFX")
        if fx:
            f = fx[b"filterFXList"][0]
            print(layer.name, f[b"filterID"], list(f[b"Fltr"].keys()))
PY
```
