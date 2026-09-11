# mzizi-console

`app.mzizi.dev` — the Mzizi console. **Astro in front, Rust behind.**

## Status

**Not deployed.** `app.mzizi.dev` does not resolve, and this repository has never
published a deployment — see [Deploying](#deploying) for what the first one will
do. The code builds and CI is green; nothing is live.

The console reads the registry API at **`https://mzizi.dev/api/v1`**. Not
`api.mzizi.dev` — that host has no DNS record at all. See
[The API address](#the-api-address).

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
Foundation) serves `mzizi.dev`, and will serve `api.mzizi.dev` once
[`mzizi-dev/mzizi-api-gateway`](https://github.com/mzizi-dev/mzizi-api-gateway)
ships; the **console** (Nyuchi) serves `app.mzizi.dev`. Neither of those two
hostnames exists yet — see [Status](#status).

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
| `GET /ui`        | typed as an array      | a registry doc (`items`) |
| a failed request | rendered an empty list | shows the status and URL |

That last one matters most: a 410 and an empty registry looked identical, so a
permanently broken route presented itself as "no data".

### The API address

`api::DEFAULT_API_BASE` is **`https://mzizi.dev/api/v1`**, and that is a
correction rather than a preference.

The imported code read `https://api.mzizi.dev/v1`, justified in a comment as a
move to the canonical address because "the old form still resolves — the API
Worker accepts both". Measured, the reverse is true:

```
api.mzizi.dev      ->  NXDOMAIN, no DNS record at all
mzizi.dev/api/v1   ->  200
```

So the console was pointed at a host that does not exist, and would have rendered
every view empty against a perfectly healthy API.

`api.mzizi.dev` is the address the ecosystem writes down, and
[`mzizi-dev/mzizi-api-gateway`](https://github.com/mzizi-dev/mzizi-api-gateway)
is being built to make it real. Switch the constant the day that host answers —
and not before.

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
  curl -s "https://mzizi.dev/api/v1/$e" -o "tests/live/$e.json"
done
```

**Do not reformat them** — not with a formatter, not by hand. Reformatting turns
them into fixtures this repo wrote rather than bytes the API sent, which is the
entire distinction they exist to preserve. The monorepo this was extracted from
enforced that with a `.prettierignore` entry; no such file came across, so for
now the rule is written down here and nowhere else.

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

**This has never been deployed.** `app.mzizi.dev` does not resolve, the
repository has no deployments recorded, and the Worker is not in the account.
Everything below is the intended path, not a description of something running.

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
makes `app.mzizi.dev` start resolving. There is nothing to add by hand first, and
the absence of a record today is a symptom of never having deployed rather than a
missing step.

Two things about this route are worth not re-learning:

- It is a **bare hostname**. Wildcards and paths are rejected outright
  (`Wildcard operators (*) are not allowed in Custom Domains`), and a custom
  domain already routes every path on the hostname here, so a `/*` is both
  invalid and redundant. The imported config carried `dashboard.mzizi.dev/*`
  with `custom_domain: true` and therefore could never deploy — which is why
  `dashboard.mzizi.dev` never existed.
- **A broken route reads green on a pull request.** Workers Builds previews
  upload a version *without* applying routes, so the config is only validated on
  the production deploy. `mzizi-mcp` carried the identical defect and failed the
  same silent way (`mzizi-dev/agent-tools#102`).

## Contributing

[CONTRIBUTING.md](CONTRIBUTING.md) — the two-toolchain build, the crate-name
trap, what CI enforces, and why this repository is **merge-only** (squash and
rebase are disabled).

- [SECURITY.md](SECURITY.md) — a read-only browser of a public registry, with no
  auth and no secrets; what that does and does not mean.
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) — Contributor Covenant 2.1.

## Licence

[Apache-2.0](LICENSE).
