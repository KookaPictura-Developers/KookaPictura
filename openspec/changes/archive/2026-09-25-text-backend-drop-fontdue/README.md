# text-backend-drop-fontdue

Remove the redundant `fontdue` dependency; read metrics/lookup/count from
`ttf-parser` (via `rustybuzz`) so the bundled backend is one pure-Rust stack.
