#!/usr/bin/env bash
# Build the console's WASM island bundle into `public/island/`.
#
# Deliberately NOT `dx bundle`. That builds a whole Dioxus web app — its own
# index.html, its own CSS pipeline, content-hashed filenames — and this is not
# an app. Astro owns the page; what is wanted here is one JS module with a
# predictable name that a `<script>` can import.
#
# Three steps, each doing one thing:
#   cargo         compile the crate to wasm32
#   wasm-bindgen  emit the JS module and the glue
#   wasm-opt      shrink it — see the size note below
#
# SIZE IS NOT COSMETIC. Cloudflare's individual static-asset limit is 25 MiB, and
# an unoptimised debug build of this crate is 39.5 MB in a single file — it would
# be REJECTED at deploy. Release plus `wasm-opt -Oz` brings it to ~0.9 MiB.
set -euo pipefail

CRATE=mzizi-console
OUT=public/island
WASM="target/wasm32-unknown-unknown/release/${CRATE}.wasm"

cargo build --release --target wasm32-unknown-unknown

rm -rf "$OUT"
mkdir -p "$OUT"
wasm-bindgen --target web --out-dir "$OUT" --out-name "$CRATE" "$WASM"

# Type declarations describe the module for a TypeScript consumer. Nothing
# imports this from TypeScript — Astro loads it as a plain module — so shipping
# them would be two more files served to every visitor for no reader.
rm -f "$OUT/${CRATE}.d.ts" "$OUT/${CRATE}_bg.wasm.d.ts"

if command -v wasm-opt >/dev/null 2>&1; then
  wasm-opt -Oz --enable-bulk-memory --enable-reference-types \
    "$OUT/${CRATE}_bg.wasm" -o "$OUT/${CRATE}_bg.wasm"
else
  # Loud, not silent. Skipping this does not produce a slightly larger bundle —
  # it produces one that may not deploy at all.
  echo "::warning::wasm-opt not found; the bundle is UNOPTIMISED and may exceed Cloudflare's 25 MiB per-asset limit" >&2
fi

printf '%s\n' "island bundle:"
# `find -printf` has no float format; sizes come from `stat`, which does.
find "$OUT" -type f -exec stat -c '%s %n' {} + \
  | sort -rn \
  | awk '{ total += $1; printf "  %8.1f KiB  %s\n", $1/1024, $2 }
         END { printf "  %8.1f KiB  TOTAL across %d files\n", total/1024, NR }'
