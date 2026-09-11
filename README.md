# Mzizi console

> `app.mzizi.dev` — the browser for the Mzizi registry: components, design tokens and the DNA-helix architecture, read live from the API. **Astro in front, Rust behind.**

[![CI](https://github.com/mzizi-dev/mzizi-console/actions/workflows/ci.yml/badge.svg)](https://github.com/mzizi-dev/mzizi-console/actions/workflows/ci.yml)
[![Lint](https://github.com/mzizi-dev/mzizi-console/actions/workflows/lint.yml/badge.svg)](https://github.com/mzizi-dev/mzizi-console/actions/workflows/lint.yml)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![Rust](https://img.shields.io/badge/Rust-Dioxus_0.7-000000?style=flat-square&logo=rust&logoColor=white)
![Astro](https://img.shields.io/badge/Astro-shell-BC52EE?style=flat-square&logo=astro&logoColor=white)
![Cloudflare Workers](https://img.shields.io/badge/Cloudflare-Workers-F38020?style=flat-square&logo=cloudflare&logoColor=white)

**Version:** 0.1.0 | **Live:** [app.mzizi.dev](https://app.mzizi.dev) | **Reads:** [api.mzizi.dev/v1](https://api.mzizi.dev/v1/ui) | **Docs:** [docs.bundu.org](https://docs.bundu.org)

---

## Status

**Live.** `https://app.mzizi.dev` returns 200 and serves
`<title>Mzizi · Mzizi console</title>`. All four routes — `/`, `/components`,
`/tokens`, `/architecture` — answer.

The console reads the registry API at **`https://api.mzizi.dev/v1`**. Measured
2026-09-12, `/v1/ui`, `/v1/brand` and `/v1/architecture` all return 200 on that
host. See [The API address](#the-api-address), which is the section of this
README with the most history behind it and the most reason to be read before
anything is "simplified".

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
becomes Rust: the framework doctrine is that the UI is Astro and underneath is
Rust first, TypeScript second, with no third UI framework.
`mzizi-dev/agent-tools#82` records the decision.

Mzizi is an open-architecture project of the **Bundu Foundation**, operated and
developed by **Nyuchi**. This console is one of its surfaces;
[`mzizi-dev/mzizi-api-gateway`](https://github.com/mzizi-dev/mzizi-api-gateway)
is another.

## Ported by contract, not translated

The rule `#82` sets: _a faithful port of a broken component still compiles_. Two
of the five routes could not have been translated even in principle.

**Architecture** called `/architecture/frontend/axes` and
`/architecture/frontend/layers`. Both answered **410 Gone** in production —
checked, not assumed — and had since the axis model was retired:

> The axis model is retired. Mzizi serves the DNA double helix — nodes on an
> engineering and a meaning backbone, held by cross-cutting rungs.

So it is rewritten against nodes, rungs and strands: **8 nodes, 4 rungs, 6
strands**, read live from `/v1/architecture`. Rungs are listed separately rather
than as nodes with an empty backbone, because belonging to neither backbone is
the fact the model turns on. "Axis", "axes" and "layer" are retired vocabulary
and should not come back in prose here, whatever the legacy route names say.

**Tokens** was described in the Svelte manifest as the _"Five African Minerals
palette"_. That was wrong, and the correction this repository shipped — "there
are seven" — is also not the whole truth.

**The palette is 21 colour families**, in three groups of seven:

| Group        | Count |                                                         Families |
| ------------ | ----: | ---------------------------------------------------------------: |
| Minerals     |     7 | cobalt, tanzanite, malachite, gold, terracotta, sodalite, copper |
| Heritage     |     7 |       indigo, savanna, baobab, sunset, river, hematite, kalahari |
| Experimental |     7 |                 ember, acacia, fern, lagoon, storm, dusk, protea |

Verified against `GET https://api.mzizi.dev/api/v1/brand`, which returns three
arrays of seven.

**Known gap, stated rather than hidden:** `api::Brand` deserialises only
`minerals`, and the Tokens island renders `"{brand.minerals.len()} minerals"`.
So the console shows **7 of the 21 families**, and `src/pages/tokens.astro`
still leads with "Seven African minerals". That is an accurate count of one
group presented as if it were the palette. Fixing it means adding `heritage` and
`experimental` to the struct and the view — a code change, not a README one, and
not in this commit.

|                  | Svelte app             | here                     |
| ---------------- | ---------------------- | ------------------------ |
| `GET /ui`        | typed as an array      | a registry doc (`items`) |
| a failed request | rendered an empty list | shows the status and URL |

That last one matters most: a 410 and an empty registry looked identical, so a
permanently broken route presented itself as "no data".

### The API address

`api::DEFAULT_API_BASE` is **`https://api.mzizi.dev/v1`**. The constant has held
three values and the middle one was right at the time, so the order matters.

**1. As imported**, it read `https://api.mzizi.dev/v1`, justified in a comment as
a move to the canonical address because "the old form still resolves — the API
Worker accepts both". Both halves were asserted rather than measured, and both
were false: there was no API Worker, and the host did not exist.

**2. Corrected** to `https://mzizi.dev/api/v1`, because measurement said:

```text
api.mzizi.dev      ->  NXDOMAIN, no DNS record at all
mzizi.dev/api/v1   ->  200
```

The console had been pointed at a host that did not resolve, and would have
rendered every view empty against a perfectly healthy API. That correction
carried a condition: switch back the day `api.mzizi.dev` answers, and not before.

**3. Switched back**, because that day arrived. `api.mzizi.dev` resolves and
serves the registry API.

**And the condition attached to step 2 has now been vindicated in full.**
This README used to say that `mzizi.dev` was "due to stop serving the API" and
that "anything still addressing `mzizi.dev/api/v1` begins returning 404" on that
day. Measured 2026-09-12:

| Address                       | Then  | Now                                         |
| ----------------------------- | ----- | ------------------------------------------- |
| `https://api.mzizi.dev/v1/ui` | `200` | `200`                                       |
| `https://mzizi.dev/api/v1/ui` | `200` | **`404`** — the apex is now the static site |

The apex is served by
[`mzizi-dev/mzizi-site`](https://github.com/mzizi-dev/mzizi-site), a three-page
Astro Worker. A console still pointed at `mzizi.dev/api/v1` would today render
every view empty. **That indirection is the whole point, and it is why this must
not be "simplified" back.**

**One thing this README asserted that is no longer true.** It said the gateway
"currently proxies to the apex — `GET api.mzizi.dev/v1/health` reports
`"origin": "https://mzizi.dev/api"`". It does not. That response now carries no
`origin` field at all, and every response from `api.mzizi.dev` arrives with
`x-opennext: 1` and Next.js `vary` headers — the signature of the registry's own
Next.js app on Cloudflare, not of the Rust Worker in `mzizi-api-gateway`. What
holds `api.mzizi.dev` today is not what this README last recorded. The address
is stable; the thing behind it moved, which is exactly what owning the name was
for.

**A note on `/v1` versus `/api/v1`.** Both prefixes serve every resource this
client reads. The bare discovery document is the one asymmetry:
`https://api.mzizi.dev/api/v1` returns the JSON index, while
`https://api.mzizi.dev/v1` returns a 404 HTML page. Nothing here requests it.
**Never write the old `mzizi.dev/api/v1` form** — that host no longer serves the
API at all.

## The API has three envelope conventions

Measured against production, not assumed:

| endpoint        | shape                                                 |
| --------------- | ----------------------------------------------------- |
| `/architecture` | `{ "data": { … }, "meta": … }`                        |
| `/ui`           | a shadcn registry document — components under `items` |
| `/brand`        | no envelope; fields top-level, keys camelCase         |

Assuming one would decode two of the three to nothing — and nothing renders as an
empty page rather than an error.

There is **no database** behind any of it. The registry is disk: `registry.json`
and `content/doctrine/**` in
[`mzizi-dev/mzizi-registry`](https://github.com/mzizi-dev/mzizi-registry).
Anything describing Supabase as the source of truth for components, brand or
tokens is wrong.

## Toolchain

Two languages, two tools, one set of script names. `agent-tools#82` settles the
tension: _the interface stays uniform; the tool fits the language._

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

## Commands

| Command                                       | What it does                     |
| --------------------------------------------- | -------------------------------- |
| `pnpm run build`                              | `build:wasm`, then `astro build` |
| `pnpm run check`                              | Both halves                      |
| `pnpm test`                                   | `cargo test`                     |
| `cargo clippy --all-targets -- -D warnings`   | Rust lints                       |
| `cargo fmt --all -- --check`                  | Rust formatting                  |
| `cargo check --target wasm32-unknown-unknown` | The real target                  |
| `pnpm exec astro check`                       | Astro typecheck                  |

Both halves must be installed for `pnpm run build` to produce a working site — a
machine with only Node completes `astro build` happily and ships a page whose
island never loads. [CONTRIBUTING.md §1](CONTRIBUTING.md#1-two-toolchains-one-build)
lists exactly what to install, including the `wasm-bindgen-cli` version pin.

### The crate name is load-bearing

The island bundle's filename is derived from the crate name, and four places
spell it out independently: `name` in `Cargo.toml`, `CRATE=` in
`scripts/build-island.sh`, the `import("/island/mzizi-console.js")` in
`src/layouts/Island.astro`, and — as `mzizi_console`, with underscores — the
`use` paths in `src/main.rs` and `tests/live_shapes.rs`.

A rename that updates some and not others produces a blank console at a URL that
returns HTTP 200. That happened during the move from `agent-tools`, where this
was `mzizi-dashboard`. **Nothing in CI catches the JS half of it** —
`Island.astro`'s script is `is:inline`, so Astro never resolves the import, and
the `web` job runs `astro build` without the island bundle present at all. Run a
full `pnpm build` after any rename. See
[CONTRIBUTING.md §3](CONTRIBUTING.md#3-the-crate-name-is-load-bearing-in-four-places).

## Verifying

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

**Do not reformat them** — not with a formatter, not by hand. Reformatting turns
them into fixtures this repo wrote rather than bytes the API sent, which is the
entire distinction they exist to preserve. The monorepo this was extracted from
enforced that with a `.prettierignore` entry; no such file came across, so for
now the rule is written down here and nowhere else.

## Size

The island bundle, after `wasm-opt -Oz`:

| file                    | size        |
| ----------------------- | ----------- |
| `mzizi-console_bg.wasm` | 921 KiB     |
| `mzizi-console.js`      | 74 KiB      |
| `snippets/` (7 files)   | 25 KiB      |
| **total**               | **1.0 MiB** |

**Optimisation here is a deployability requirement, not a nicety.** Cloudflare's
individual static-asset limit is **25 MiB** on every plan, and an unoptimised
debug build of this crate is **39.5 MB in a single file** — it would be rejected
at deploy. `scripts/build-island.sh` warns loudly rather than silently if
`wasm-opt` is missing, because skipping it does not produce a slightly larger
bundle; it produces one that may not deploy at all.

## Deploying

**Deployed.** `app.mzizi.dev` resolves and serves this console. Everything below
describes the path that produced it.

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
its own, just static assets. See [SECURITY.md](SECURITY.md).

### The first deploy is what creates the DNS record

`wrangler.jsonc` declares one route:

```jsonc
"routes": [{ "pattern": "app.mzizi.dev", "custom_domain": true }]
```

`mzizi.dev` is on Cloudflare, so a custom domain on a zone in the same account
**provisions its own DNS record**: the first successful production deploy is what
made `app.mzizi.dev` start resolving. There was nothing to add by hand first, and
the absence of a record beforehand was a symptom of never having deployed rather
than a missing step.

**The apex behaved differently, and the difference has since been demonstrated
the hard way.** `mzizi.dev` already had a record, so a custom domain there does
not create one — it **takes** it. That is what happened: the apex now serves
`mzizi-site`, the registry's developer portal has no live address, and neither
the cutover runbook (`mzizi-site#3`) nor the pull request porting the displaced
pages (`mzizi-site#5`) had been merged first. See
[`mzizi-site/README.md`](https://github.com/mzizi-dev/mzizi-site#readme).

Two things about this route are worth not re-learning:

- It is a **bare hostname**. Wildcards and paths are rejected outright
  (`Wildcard operators (*) are not allowed in Custom Domains`), and a custom
  domain already routes every path on the hostname here, so a `/*` is both
  invalid and redundant. The imported config carried `dashboard.mzizi.dev/*`
  with `custom_domain: true` and therefore could never deploy — which is why
  `dashboard.mzizi.dev` never existed.
- **A broken route reads green on a pull request.** Workers Builds previews
  upload a version _without_ applying routes, so the config is only validated on
  the production deploy. `mzizi-mcp` carried the identical defect and failed the
  same silent way (`mzizi-dev/agent-tools#102`).

## Ecosystem

| Repository                                                            | What it is                                     | Address                                       |
| --------------------------------------------------------------------- | ---------------------------------------------- | --------------------------------------------- |
| [`mzizi`](https://github.com/mzizi-dev/mzizi)                         | The language — Rust compiler research, Phase 0 | —                                             |
| [`mzizi-registry`](https://github.com/mzizi-dev/mzizi-registry)       | The component registry, brand and architecture | Portal currently unrouted                     |
| [`mzizi-api-gateway`](https://github.com/mzizi-dev/mzizi-api-gateway) | The registry API as a pure-Rust Worker         | [api.mzizi.dev](https://api.mzizi.dev/api/v1) |
| [`mzizi-site`](https://github.com/mzizi-dev/mzizi-site)               | The ecosystem front door                       | [mzizi.dev](https://mzizi.dev)                |
| `mzizi-console`                                                       | This repository                                | [app.mzizi.dev](https://app.mzizi.dev)        |

## Contributing

[CONTRIBUTING.md](CONTRIBUTING.md) — the two-toolchain build, the crate-name
trap, and what CI enforces.

- [SECURITY.md](SECURITY.md) — a read-only browser of a public registry, with no
  auth and no secrets; what that does and does not mean.
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) — Contributor Covenant 2.1.

**This repository is rebase-only.** Read off the API on 2026-09-12,
`allow_rebase_merge` is `true` and `allow_merge_commit` and
`allow_squash_merge` are both `false` on all nine repos in `mzizi-dev` — and on
all 75 in the enterprise. Auto-merge is enabled. CONTRIBUTING.md still describes
the merge-only convention that preceded this; the API is the authority.

```sh
gh pr merge <n> --rebase --auto
```

Never `--admin`.

## Licence

Licensed under the [Apache License 2.0](LICENSE).

Mzizi is an open-architecture project of the **Bundu Foundation**, operated and
developed by **Nyuchi**.
