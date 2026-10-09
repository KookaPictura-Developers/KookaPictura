# Design: filter-alpha-allowlist

## Allowlist, not the fourth plane

The real fix folds alpha into the working buffer so each kernel receives the
fourth plane and decides for itself. That touches all 97 kernels and the GPU
path. Inverting the existing blocklist fixes every reported kind now, and it
fails safe: a kind added later leaves alpha untouched until someone adds it to
the list, rather than silently rewriting opacity. The `ponytail:` on the
predicate keeps the real fix named.

A kind is on the list when the grey pass is exactly what CS6 shows for
transparency: the kernel treats every plane the same way regardless of value,
so the layer edge moves, smears, or erodes with the colour, and a flat plane
stays flat. Kinds that compare values against a threshold or cluster by
colour (Facet, Mezzotint, Diffuse's modes, Wind, Extrude, Tiles) stay off,
because the grey pass would partition alpha differently from colour.

## Offset's exposed area

`Filter::Offset` has one `background` colour, and the dialog labels it
`Background:`, which is CS6's "Set to Background". On a normal layer that mode
paints the exposed area opaque in the Background colour, so the alpha pass
runs Offset with a white (255) background while the colour pass keeps the
user's colour. Wrap Around moves alpha with colour as before.

## Tests

`pictura-render` `filter::tests`:

- `value_kernels_leave_alpha_bit_identical`: Solarize, Clouds, Difference
  Clouds, Fibers, Lens Flare, Trace Contour, Tiles, Find Edges, Emboss, High
  Pass, Add Noise, plus the three earlier exemptions, on an opaque and a
  half-transparent layer.
- `moving_kernels_keep_an_opaque_layer_opaque`: a sample of the allowlist,
  including Offset 2000/2000 without wrap and a black Background.
- `offset_moves_the_layer_edge_and_fills_what_it_exposes_opaque`.

Each fails against the previous predicate.
