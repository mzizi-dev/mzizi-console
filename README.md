# Mzizi console

> `app.mzizi.dev` — the browser for the Mzizi registry: components, design tokens and the DNA-helix architecture, read live from the API. **Astro in front, Rust behind.**

[![CI](https://github.com/mzizi-dev/mzizi-console/actions/workflows/ci.yml/badge.svg)](https://github.com/mzizi-dev/mzizi-console/actions/workflows/ci.yml)
[![Lint](https://github.com/mzizi-dev/mzizi-console/actions/workflows/lint.yml/badge.svg)](https://github.com/mzizi-dev/mzizi-console/actions/workflows/lint.yml)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![Rust](https://img.shields.io/badge/Rust-Dioxus_0.7-000000?style=flat-square&logo=rust&logoColor=white)
![Astro](https://img.shields.io/badge/Astro-shell-BC52EE?style=flat-square&logo=astro&logoColor=white)
![Cloudflare Workers](https://img.shields.io/badge/Cloudflare-Workers-F38020?style=flat-square&logo=cloudflare&logoColor=white)

**Version:** 0.1.0 | **Live:** [app.mzizi.dev](https://app.mzizi.dev) | **Reads:** [api.mzizi.dev/v1](https://api.mzizi.dev/v1/ui) | **Docs:** [docs.mzizi.dev](https://docs.mzizi.dev)

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

Mzizi is an independent open-architecture project that owns, operates and
develops its framework, design system and registry. This console is one of its
surfaces and is run under **Nyuchi**, like everything revenue-generating.
[`mzizi-dev/mzizi-api-gateway`](https://github.com/mzizi-dev/mzizi-api-gateway)
is another surface, and it is Mzizi's.

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

## Authentication

The console is gated behind **WorkOS AuthKit**, entirely client-side —
`@workos-inc/authkit-js`, not the SSR-only `@workos/authkit-astro`, because this site is
`output: "static"` with no server to receive an OAuth callback. No API key, no client
secret, and no cookie password ever enter the browser bundle: the browser SDK runs
authorization-code-with-PKCE and the refresh token lives in memory (or an HttpOnly cookie
off `localhost`), never in `localStorage` on a real deployment.

The gate is client-side, so it's not an authorization boundary — the API's is. What it does
do is real: the island doesn't mount, and the ~1 MiB WASM bundle isn't downloaded, until
AuthKit reports a signed-in user. It fails closed — an unset `PUBLIC_WORKOS_CLIENT_ID`
refuses every visitor, says so on the page, and disables the sign-in button rather than
quietly letting everyone through. See `.env.example` for the four variables involved and
[`AGENTS.md`](./AGENTS.md) for the toolchain and deploy mechanics.

## Toolchain and commands

Two languages, two tools, one set of script names: Vite+ (`vp`) drives the Astro half,
Cargo drives the Rust half, `pnpm run build` composes them. See [`AGENTS.md`](./AGENTS.md)
for the full command reference, the crate-name trap (a rename that misses one of four
places produces a blank console CI won't catch), how `tests/live_shapes.rs` is verified
against production, the WASM size budget, and — before you touch `wrangler.jsonc` or
deploy — the route footgun this org has already hit twice.

## Ecosystem

| Repository                                                            | What it is                                     | Address                                       |
| --------------------------------------------------------------------- | ---------------------------------------------- | --------------------------------------------- |
| [`mzizi`](https://github.com/mzizi-dev/mzizi)                         | The language — Rust compiler research, Phase 0 | —                                             |
| [`mzizi-registry`](https://github.com/mzizi-dev/mzizi-registry)       | The component registry, brand and architecture | Portal currently unrouted                     |
| [`mzizi-api-gateway`](https://github.com/mzizi-dev/mzizi-api-gateway) | The registry API as a pure-Rust Worker         | [api.mzizi.dev](https://api.mzizi.dev/api/v1) |
| [`mzizi-site`](https://github.com/mzizi-dev/mzizi-site)               | The ecosystem front door                       | [mzizi.dev](https://mzizi.dev)                |
| `mzizi-console`                                                       | This repository                                | [app.mzizi.dev](https://app.mzizi.dev)        |

## Contributing

[`CONTRIBUTING.md`](CONTRIBUTING.md) — the two-toolchain build, the crate-name
trap, and what CI enforces. [`AGENTS.md`](AGENTS.md) — the same, in agent-facing
form, plus the merge convention and deploy mechanics.

- [`SECURITY.md`](SECURITY.md) — what a read-only browser of a public registry
  does and doesn't need to worry about.
- [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) — Contributor Covenant 2.1.

## Licence

Licensed under the [Apache License 2.0](LICENSE).

Mzizi is an independent open-architecture project that owns, operates and
develops its framework, design system and registry. This console is run
under **Nyuchi**.
