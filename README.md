# mzizi-console

`app.mzizi.dev` — the Mzizi console. **Astro in front, Rust behind.**

## The split

Astro renders the **chrome** — navigation, headings, prose, the page shell — as
static HTML at build time. Rust/Dioxus **islands** render the **data**, fetched
from the registry API at runtime.

Both halves of that are deliberate.

The chrome is static because a console's navigation has no reason to cost a
900 KiB WASM download before it can show a heading. The Overview page mounts no
island at all and ships zero bytes of WASM.

The data is live because a component list baked in at build time would go stale
the moment a component shipped — and a stale copy that still looks authoritative
is the defect class this ecosystem keeps removing. What the console shows is what
a consumer would install right now.

## What it replaces

`@nyuchi/mzizi-console-app`, a Svelte 5 mini-app the Nyuchi Console mounted under
`/apps/mzizi/*`. It becomes a standalone surface at its own domain, and it
becomes Rust: the framework doctrine (`nyuchi/mzizi#269`) is that the UI is Astro
and underneath is Rust first, TypeScript second, with no third UI framework.
`mzizi-tools#82` records the decision.

This separates the two owners cleanly — the **Mzizi framework** (Bundu
Foundation) serves `mzizi.dev` and `api.mzizi.dev`; the **console** (Nyuchi)
serves `app.mzizi.dev`.

## Ported by contract, not translated

The rule #82 sets: _a faithful port of a broken component still compiles_. Two of
the five routes could not have been translated even in principle.

**Architecture** called `/architecture/frontend/axes` and
`/architecture/frontend/layers`. Both answer **410 Gone** in production — checked,
not assumed — and have since the axis model was retired:

> The axis model is retired. Mzizi serves the DNA double helix — nodes on an
> engineering and a meaning backbone, held by cross-cutting rungs.

So it is rewritten against nodes, rungs and strands. Rungs are listed separately
rather than as nodes with an empty backbone: belonging to neither backbone is the
fact the model turns on.

**Tokens** was described in the Svelte manifest as the _"Five African Minerals
palette"_. There are **seven**. `nyuchi/mzizi#265` removed that naming from the
framework repo and added a guard; the guard does not reach this repo.

|                  | Svelte app             | here                     |
| ---------------- | ---------------------- | ------------------------ |
| API base         | `mzizi.dev/api/v1`     | `api.mzizi.dev/v1`       |
| `GET /ui`        | typed as an array      | a registry doc (`items`) |
| a failed request | rendered an empty list | shows the status and URL |

That last one matters most: a 410 and an empty registry looked identical, so a
permanently broken route presented itself as "no data".

## The API has three envelope conventions

Measured against production, not assumed:

| endpoint        | shape                                                 |
| --------------- | ----------------------------------------------------- |
| `/architecture` | `{ "data": { … }, "meta": … }`                        |
| `/ui`           | a shadcn registry document — components under `items` |
| `/brand`        | no envelope; fields top-level, keys camelCase         |

Assuming one would decode two of the three to nothing — and nothing renders as an
empty page rather than an error.

## Toolchain

Two languages, two tools, one set of script names. `mzizi-tools#82` settles the
tension: _the interface stays uniform; the tool fits the language._

| | front (Astro) | behind (Rust) |

|        | front (Astro)            | behind (Rust)              |
| ------ | ------------------------ | -------------------------- |
| build  | `astro build`            | `cargo` + `wasm-bindgen`   |
| check  | `astro check`, `vp lint` | `cargo clippy`, `--wasm32` |
| format | `vp fmt`                 | `cargo fmt`                |
| test   | —                        | `cargo test`               |

**Vite+ cannot build the Rust half.** It is JS/TS tooling — Vite, Vitest, Oxlint,
Oxfmt, Rolldown — and while it is itself written in Rust, it has no cargo, no
wasm-bindgen and no WebAssembly target. That is not a gap to work around; it is
what `vp run` exists for. Vite+ drives the Astro half and the package scripts,
Cargo drives the Rust half, and `pnpm run build` composes them.

```bash
pnpm run build     # build:wasm, then astro build
pnpm run check     # both halves
pnpm test          # cargo test
```

## Verifying

```bash
cargo test                                  # unit + live-shape decoding
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
cargo check --target wasm32-unknown-unknown
pnpm exec astro check
```

`tests/live_shapes.rs` decodes **captured live responses** with the production
types. That is a different claim from the unit tests, which decode fixtures this
repo wrote and prove only that the types are self-consistent — the Svelte
client's types were self-consistent and wrong. The captures are refreshed by
hand:

```bash
for e in ui brand architecture; do
  curl -s "https://api.mzizi.dev/v1/$e" -o "tests/live/$e.json"
done
```

They are in `.prettierignore` on purpose: reformatting them would make them
fixtures this repo wrote rather than bytes the API sent.

## Size

The island bundle, after `wasm-opt -Oz`:

| file                      | size        |
| ------------------------- | ----------- |
| `mzizi-console_bg.wasm` | 921 KiB     |
| `mzizi-console.js`      | 74 KiB      |
| `snippets/` (7 files)     | 25 KiB      |
| **total**                 | **1.0 MiB** |

**Optimisation here is a deployability requirement, not a nicety.** Cloudflare's
individual static-asset limit is **25 MiB** on every plan, and an unoptimised
debug build of this crate is **39.5 MB in a single file** — it would be rejected
at deploy. `scripts/build-island.sh` warns loudly rather than silently if
`wasm-opt` is missing, because skipping it does not produce a slightly larger
bundle; it produces one that may not deploy at all.

## Deploying

Via the **Cloudflare GitHub app**, configured per Worker in the Cloudflare
dashboard. There is no deploy workflow in this repo and there should not be one:
CI does build checks and tests, not publishing.

|                |                                  |
| -------------- | -------------------------------- |
| Root directory | `.` (repo root)                  |
| Build command  | `pnpm install && pnpm run build` |
| Deploy command | `npx wrangler deploy`            |

The build needs `wasm-bindgen-cli` and `wasm-opt` on `PATH` alongside a Rust
toolchain with the `wasm32-unknown-unknown` target.

No secrets. The console reads only the public API, so there is nothing to
configure and nothing to leak — which is also why it is a Worker with no code of
its own, just static assets.
