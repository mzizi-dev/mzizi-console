# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

@AGENTS.md

`AGENTS.md` (imported above) is the primary agent guide: toolchain, the crate-name
trap, live-shape tests, size budget, deploy/route rules, merge convention, honesty
rules and the dev-skills/merge gate. This file adds only what it leaves out.

## Commands

Verified against `package.json`, `Cargo.toml` and `.github/workflows/ci.yml`.

```bash
pnpm install --frozen-lockfile          # CI pins pnpm 10.33.0, Node 22
pnpm run build                          # build:wasm (scripts/build-island.sh) then astro build
pnpm run build:wasm                     # island only -> public/island/ (gitignored)
pnpm dev                                # astro dev; does NOT rebuild the island
pnpm run check                          # check:web (astro check && vp check) + check:rust
pnpm run lint                           # vp lint (oxlint) - CI runs this
pnpm run fmt                            # vp fmt
pnpm exec astro check                   # TS/Astro typecheck only
pnpm test                               # = cargo test
```

Rust gates, exactly as the `rust` CI job runs them:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo check --target wasm32-unknown-unknown --all-targets
```

Running a single test (all tests are Rust; there are no JS tests):

```bash
cargo test --test live_shapes                     # just the captured-live-response suite
cargo test brand_has_no_data_envelope             # any test whose name contains the filter
cargo test --lib api::tests                       # unit tests in src/api.rs
```

Notes:

- `pnpm run check:web` uses `vp check` (lint + format + type-aware check), while CI's
  `web` job runs `astro check`, `pnpm run lint` and `astro build` separately. AGENTS.md's
  table describing `check` as `astro check && vp lint` predates this.
- After editing any `src/*.rs`, re-run `pnpm run build:wasm` before looking at
  `pnpm dev`; Astro serves `public/` as-is.
- Clippy denies `unwrap_used`, `expect_used` and `todo` crate-wide (including tests),
  `unsafe_code` is forbidden, and `missing_docs` warns, which `-D warnings` makes fatal.

## Architecture: how the halves connect

- **Astro (`src/pages/*.astro`, `src/layouts/Console.astro`)** emits static HTML
  (`output: "static"`, `build.format: "file"` so `/tokens` resolves to `tokens.html`).
  It fetches nothing at build time.
- **Mount point:** pages that show data render `src/layouts/Island.astro` with
  `name="components" | "tokens" | "architecture"`, producing
  `<div id="island" data-island=…>`. The Overview page and 404 mount no island.
- **One WASM bundle for all islands.** `Island.astro`'s `is:inline` script dynamically
  imports `/island/mzizi-console.js`. `src/main.rs` launches Dioxus into `#island`
  (not Dioxus' default `#main`); `src/island.rs` reads `data-island`, parses it with
  `Island::parse` (unknown names are an error, not a default) and renders that view.
- **Data:** each island fetches from `api::DEFAULT_API_BASE` (`src/api.rs`,
  `https://api.mzizi.dev/v1`) via `reqwest`, which compiles to browser `fetch` on wasm32.
  `src/api.rs` holds the response types for three different envelope shapes
  (see README "The API has three envelope conventions"). Failed requests render the
  HTTP status and URL rather than an empty list.
- **Auth gate:** `Console.astro` + `src/lib/authkit.ts` run WorkOS AuthKit client-side
  (`@workos-inc/authkit-js`, PKCE, `/callback` page). The island only starts once
  `<html data-auth="in">` is set or a `mzizi:auth` event reports `state: "in"`.
  An unset `PUBLIC_WORKOS_CLIENT_ID` fails closed (`astro.config.mjs` warns at build).
  Env vars are listed in `.env.example`.
- **Code that only runs in the browser** is behind `#[cfg(target_arch = "wasm32")]`
  (e.g. reading `data-island`, `web-sys`); native builds get a stub so `cargo test` works.

## Generated vs hand-written

- Generated, gitignored: `public/island/` (cargo + wasm-bindgen + wasm-opt via
  `scripts/build-island.sh`), `dist/`, `.astro/`, `target/`.
- `tests/live/*.json` are captured production responses, not fixtures: never reformat
  or hand-edit (refresh command in AGENTS.md).
- `scripts/bootstrap-toolchain.sh` installs Rust, `wasm-bindgen-cli` and `wasm-opt`
  only when `WORKERS_CI` is set (Cloudflare Workers Builds); locally it is a no-op.

## Branches, releases and deploys

- Work branches off `staging`; PRs target `staging`. CI runs on pushes to
  `main`/`staging` and PRs into `main`, `staging` and `claude/**` (for stacked PRs).
- `.github/workflows/staging-version.yml`: every merge into `staging` is tagged as the
  next **patch** version by the org's reusable release workflow (minor/major only via
  manual `workflow_dispatch`). Don't hand-edit versions or create tags.
- Pushes to `staging` upload a non-production Workers version, aliased
  `https://staging-mzizi-console.nyuchi.workers.dev` (`preview_urls: true`,
  `workers_dev: false` in `wrangler.jsonc`). Production (`app.mzizi.dev`) deploys via
  the Cloudflare GitHub app; there is no deploy workflow here and none should be added.
- Merge with `gh pr merge <n> --rebase --auto` (rebase-only; CONTRIBUTING.md's
  merge-commit section is stale).
