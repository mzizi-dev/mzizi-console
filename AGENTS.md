# AGENTS.md — mzizi-console

> Vendor-neutral instructions for any AI agent working in this repository. See
> [`README.md`](./README.md) for what this console is and the history behind its API address.
> [`CONTRIBUTING.md`](./CONTRIBUTING.md) has the full two-toolchain build writeup.

## What this repo is

`app.mzizi.dev` — Astro renders the chrome (navigation, headings, page shell) as static
HTML at build time; a Rust/Dioxus island renders the data, fetched live from the registry
API at runtime. Two languages, two toolchains, one set of script names.

## Toolchain

|        | front (Astro)            | behind (Rust)              |
| ------ | ------------------------ | -------------------------- |
| build  | `astro build`            | `cargo` + `wasm-bindgen`   |
| check  | `astro check`, `vp lint` | `cargo clippy`, `--wasm32` |
| format | `vp fmt`                 | `cargo fmt`                |
| test   | —                        | `cargo test`               |

**Vite+ cannot build the Rust half.** It's JS/TS tooling (Vite, Vitest, Oxlint, Oxfmt,
Rolldown) — no cargo, no wasm-bindgen, no WebAssembly target. `vp run` drives the Astro half
and the package scripts; Cargo drives the Rust half; `pnpm run build` composes both.

## Commands

| Command                                       | What it does                                                |
| --------------------------------------------- | ----------------------------------------------------------- |
| `pnpm run build`                              | `build:wasm`, then `astro build`                            |
| `pnpm run check`                              | Both halves (`astro check && vp lint`, then the Rust gates) |
| `pnpm test`                                   | `cargo test`                                                |
| `cargo clippy --all-targets -- -D warnings`   | Rust lints                                                  |
| `cargo fmt --all -- --check`                  | Rust formatting                                             |
| `cargo check --target wasm32-unknown-unknown` | The real target                                             |
| `pnpm exec astro check`                       | Astro typecheck                                             |

Both toolchains must be installed for `pnpm run build` to produce a working site — a
machine with only Node completes `astro build` happily and ships a page whose island never
loads. [`CONTRIBUTING.md` §1](./CONTRIBUTING.md#1-two-toolchains-one-build) lists exactly
what to install, including the `wasm-bindgen-cli` version pin (must track the `dioxus`
dependency's resolved `wasm-bindgen` version exactly — see `scripts/build-island.sh`).

## The crate name is load-bearing — read before renaming anything

The island bundle's filename is derived from the crate name, and **four places spell it out
independently**: `name` in `Cargo.toml`, `CRATE=` in `scripts/build-island.sh`, the
`import("/island/mzizi-console.js")` in `src/layouts/Island.astro`, and — as
`mzizi_console`, with underscores — the `use` paths in `src/main.rs` and
`tests/live_shapes.rs`.

A rename that updates some and not others produces a blank console at a URL that returns
HTTP 200. This happened once already, during the move from `agent-tools` (where this crate
was `mzizi-dashboard`). **Nothing in CI catches the JS half of it** —
`Island.astro`'s script is `is:inline`, so Astro never resolves the import, and the `web` CI
job runs `astro build` without the island bundle present at all. Run a full `pnpm build`
after any rename, and see
[`CONTRIBUTING.md` §3](./CONTRIBUTING.md#3-the-crate-name-is-load-bearing-in-four-places).

## Verifying against production, not fixtures

`tests/live_shapes.rs` decodes **captured live responses** with the production types — a
different claim from the unit tests, which decode fixtures this repo wrote and prove only
internal self-consistency (the Svelte client this replaced had types that were
self-consistent and wrong). Refresh the captures by hand:

```bash
for e in ui brand architecture; do
  curl -s "https://api.mzizi.dev/v1/$e" -o "tests/live/$e.json"
done
```

**Do not reformat them** — not with a formatter, not by hand. Reformatting turns them into
fixtures this repo wrote rather than bytes the API actually sent, which is the whole
distinction they exist to preserve. There is no `.prettierignore` entry covering this yet, so
this rule is written down here and nowhere else — don't let a lint autofix touch
`tests/live/`.

## Size budget

The island bundle, after `wasm-opt -Oz`, totals ~1.0 MiB (`mzizi-console_bg.wasm` 921 KiB,
`mzizi-console.js` 74 KiB, `snippets/` 25 KiB). **This is a deployability requirement, not a
nicety** — Cloudflare's individual static-asset limit is 25 MiB on every plan, and an
unoptimised debug build of this crate is 39.5 MB in a single file, which would be rejected at
deploy. `scripts/build-island.sh` warns loudly if `wasm-opt` is missing rather than silently
shipping an unoptimised bundle.

## Deploying — read this before touching `wrangler.jsonc` or `routes`

Via the Cloudflare GitHub app, configured per-Worker in the Cloudflare dashboard — there is
no deploy workflow in this repo, and there should not be one: CI does build checks and
tests, not publishing.

|                |                                  |
| -------------- | -------------------------------- |
| Root directory | `.` (repo root)                  |
| Build command  | `pnpm install && pnpm run build` |
| Deploy command | `npx wrangler deploy`            |

The build needs `wasm-bindgen-cli` and `wasm-opt` on `PATH` alongside a Rust toolchain with
the `wasm32-unknown-unknown` target. No secrets: the console reads only the public API, so
there's nothing to configure and nothing to leak.

### The route is a bare hostname, and previews don't validate it

`wrangler.jsonc` declares one route: `{"pattern": "app.mzizi.dev", "custom_domain": true}`.
`mzizi.dev` is on Cloudflare, so a custom domain on a zone in the same account
**provisions its own DNS record** on first successful production deploy — there's nothing to
add by hand first.

**The apex behaved differently, and it's worth not re-learning why.** `mzizi.dev` already
had a record, so a custom domain there doesn't create one — it **takes** it, which is
exactly what happened to `mzizi-site`'s apex cutover (see
[`mzizi-site/README.md`](https://github.com/mzizi-dev/mzizi-site#readme)). Two things about
this route class of config are worth keeping straight:

- It's a **bare hostname**. Wildcards and paths are rejected outright, and a custom domain
  already routes every path on the hostname, so a `/*` is both invalid and redundant. The
  imported config once carried `dashboard.mzizi.dev/*` with `custom_domain: true` and could
  never deploy — which is why `dashboard.mzizi.dev` never existed.
- **A broken route reads green on a pull request.** Workers Builds previews upload a version
  _without_ applying routes, so the config is only validated on the production deploy.
  `mzizi-mcp` carried the identical defect and failed the same silent way
  (`mzizi-dev/agent-tools#102`).

## Merge convention

This repository is rebase-only: `allow_rebase_merge` is `true`, `allow_merge_commit` and
`allow_squash_merge` are both `false` (read off the GitHub API 2026-09-12, org-wide).
`CONTRIBUTING.md` still describes the merge-commit convention that preceded this — the live
API is the authority. Land changes with `gh pr merge <n> --rebase --auto`. Never `--admin`.

## Honesty rules

- **No database behind the registry.** It's disk: `registry.json` and
  `content/doctrine/**` in `mzizi-registry`. Anything describing Supabase as the source of
  truth for components, brand, or tokens is wrong.
- **"Axis", "axes" and "layer" are retired vocabulary** — the model is nodes, rungs, and
  strands (8 nodes, 4 rungs, 6 strands). Legacy route names may still say otherwise; prose
  shouldn't.
- **The palette is 21 colour families**, not "seven" or "five" — see README's "Ported by
  contract" section for the known gap where the console itself only surfaces 7 of 21.
