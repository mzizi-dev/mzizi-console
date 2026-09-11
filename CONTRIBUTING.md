# Contributing to `mzizi-console`

`app.mzizi.dev` — the Mzizi console. Astro in front, Rust behind.

This file covers the things about this repository that are **not** guessable
from reading the source. If something here reads as over-explained, it is
because the project has already been broken by that exact thing once — the
crate-name section below is the record of a real production-blank-page defect,
not a hypothetical.

---

## 1. Two toolchains, one build

This repository is a Rust crate and an Astro site in the same tree. Neither half
builds the other, and `pnpm build` composes them:

```jsonc
// package.json
"build":      "pnpm run build:wasm && vp run astro:build",
"build:wasm": "bash scripts/build-island.sh",
"astro:build": "astro build"
```

`scripts/build-island.sh` runs three steps into `public/island/`:

| step           | does                                           |
| -------------- | ---------------------------------------------- |
| `cargo build`  | compiles the crate to `wasm32-unknown-unknown` |
| `wasm-bindgen` | emits the JS module and the glue               |
| `wasm-opt -Oz` | shrinks the `.wasm` — see §2                   |

Then `astro build` renders the pages and copies `public/` — including the island
bundle that has just appeared there — verbatim into `dist/`.

**You need both halves installed.** A machine with only Node gets through
`astro build` perfectly happily and produces a site whose island never loads,
because `public/island/` was never written. There is no error; the page shows
`Could not start the console: TypeError: Failed to fetch dynamically imported
module`.

### What to install

```bash
# Rust half
rustup toolchain install stable            # edition 2024, so >= 1.85
rustup target add wasm32-unknown-unknown

# The wasm-bindgen CLI version MUST match the `wasm-bindgen` version resolved
# in Cargo.lock — currently 0.2.127. wasm-bindgen refuses to process a module
# built by a different version of itself, and the error names both versions.
cargo install wasm-bindgen-cli --version 0.2.127

# wasm-opt ships in binaryen. See §2 — this one is not optional.
brew install binaryen        # macOS
# apt-get install binaryen   # Debian/Ubuntu

# JS half
corepack enable && corepack prepare pnpm@10.33.0 --activate   # CI pins 10.33.0
pnpm install --frozen-lockfile
```

### Known wart: `build-island.sh` needs GNU `stat`

The script's closing size report uses `stat -c '%s %n'`, which is GNU coreutils
syntax. macOS ships BSD `stat`, which rejects `-c` outright — and because the
script runs under `set -euo pipefail`, that failure aborts `pnpm run build:wasm`
with a non-zero exit **after the bundle has already been built correctly**. The
bundle in `public/island/` is fine; only the report failed.

CI runs on `ubuntu-latest` and never sees this. On macOS, either
`brew install coreutils` and put `gnubin` on `PATH`, or just read the exit code
for what it is. Fixing the script to use `stat -f` on BSD would be welcome.

---

## 2. `wasm-opt` is not an optimisation, it is a deploy gate

Skipping `wasm-opt` does not produce a slightly larger bundle. It produces one
that may not deploy at all.

- Cloudflare's limit on an **individual static asset is 25 MiB**, on every plan.
- An unoptimised debug build of this crate is **39.5 MB in a single file**.
- Release + `wasm-opt -Oz` brings the `.wasm` to **~921 KiB**.

`scripts/build-island.sh` therefore warns loudly rather than silently when
`wasm-opt` is missing:

```
::warning::wasm-opt not found; the bundle is UNOPTIMISED and may exceed
Cloudflare's 25 MiB per-asset limit
```

It is a warning and not a hard failure so that a contributor without binaryen can
still get a working local dev loop. Do not push a build made that way, and do not
convert the warning into a silent skip.

---

## 3. The crate name is load-bearing in four places

The island bundle's filename is **derived from the crate name**, and nothing
reconciles the places that spell it out. They must agree:

| #   | file                                  | what it says                         | form            |
| --- | ------------------------------------- | ------------------------------------ | --------------- |
| 1   | `Cargo.toml`                          | `name = "mzizi-console"`             | hyphenated      |
| 2   | `scripts/build-island.sh`             | `CRATE=mzizi-console`                | hyphenated      |
| 3   | `src/layouts/Island.astro`            | `import("/island/mzizi-console.js")` | hyphenated      |
| 4   | `src/main.rs`, `tests/live_shapes.rs` | `use mzizi_console::…`               | **underscored** |

(4) is the one that surprises people: a crate's _library_ name is its package
name with hyphens turned into underscores, so renaming the package renames the
`use` path too.

**This has already happened here.** The console arrived from
`mzizi-dev/agent-tools` as `mzizi-dashboard` and was renamed in the move
(commit `35262a5`). The rename updated `Cargo.toml`, `build-island.sh` and
`Island.astro` — and missed the two Rust `use mzizi_dashboard::…` imports, which
CI caught on the first run and `52218dc` fixed. Before that, `Island.astro` was
still importing `/island/mzizi-dashboard.js`, a file the build no longer emits:
a blank console at a URL that returns HTTP 200.

### What actually catches a mismatch — and what does not

Be careful here, because the in-repo comments overstate it. The comment in
`.github/workflows/ci.yml` says `astro build` is "what catches a crate rename
that updates the build script and not the import". **It does not, and it cannot:**

- `Island.astro`'s `<script>` is `is:inline`, which is precisely the directive
  that tells Astro _not_ to resolve or bundle the import. That is deliberate and
  correct — the file it names is an output of the other toolchain and does not
  exist when the JS build runs — but it also means Astro never checks the name.
- The `web` CI job runs `pnpm exec astro build`, not `pnpm build`, so
  `public/island/` is empty in CI regardless.

Verified: `astro build` completes cleanly in a tree with no `public/island/`
directory at all.

So the only things that catch a name mismatch today are the Rust compiler (for
form 4) and a human running a full local build. Until CI grows a check, **after
any rename, run `pnpm build` and confirm the name**:

```bash
pnpm build
ls dist/island/                          # must contain <crate>.js and <crate>_bg.wasm
grep -o '/island/[a-z-]*\.js' src/layouts/Island.astro   # must name the same file
```

A CI step asserting those three strings agree would be a genuinely useful
contribution.

---

## 4. Why the split is what it is

Astro renders the **chrome** — nav, headings, prose, the page shell — as static
HTML at build time. Rust/Dioxus **islands** render the **data**, fetched from the
registry API at runtime. Both halves of that are choices, not accidents, and a
change that blurs them needs an argument.

**The chrome is static** because a console's navigation has no reason to cost a
~900 KiB WASM download before it can show a heading. The Overview page
(`src/pages/index.astro`) mounts no island at all and ships zero bytes of WASM.

**The data is live** because a component list baked in at build time goes stale
the moment a component ships — and a stale copy that still looks authoritative is
the defect class this ecosystem keeps removing. What the console shows is what a
consumer would install right now.

Two consequences worth knowing before you write a component:

- **One bundle, three islands.** `src/layouts/Island.astro` passes a
  `data-island` attribute (`components` / `tokens` / `architecture`) and
  `island::Root` reads it. One binary rather than one per route, because the
  Dioxus runtime is most of the bytes and three bundles would ship it three
  times. An unknown name renders an error naming the attribute rather than
  falling back to a default view.
- **No router, no layout, no nav in Rust.** Astro owns all three. `src/main.rs`
  is an entry point, not an app shell.

### The API base is measured, not preferred

`api::DEFAULT_API_BASE` is `https://api.mzizi.dev/v1` — the **gateway**, not the
apex.

It read `https://mzizi.dev/api/v1` for as long as `api.mzizi.dev` was NXDOMAIN,
and that earlier value was correct: the imported version pointed at a host with
no DNS record and would have rendered every view empty against a perfectly
healthy API. The correction carried a condition — switch the day the host
answers. `mzizi-dev/mzizi-api-gateway` shipped, `api.mzizi.dev` answers, and all
four endpoints this client reads are byte-identical through both hosts, so the
condition is met.

Do not "simplify" it back to the apex. The two are identical today only because
the gateway proxies to the apex; `mzizi.dev` is due to stop serving `/api/v1`
when the apex becomes `mzizi-dev/mzizi-site`, and the gateway is the name that
survives that. Measure before you change it either way:

```bash
curl -s https://api.mzizi.dev/v1/health     # names the gateway's current origin
```

### The API has three envelope conventions

Measured against production, not assumed:

| endpoint        | shape                                                 |
| --------------- | ----------------------------------------------------- |
| `/architecture` | `{ "data": { … }, "meta": … }`                        |
| `/ui`           | a shadcn registry document — components under `items` |
| `/brand`        | no envelope; fields top-level, keys camelCase         |

Assuming one convention would decode two of the three to nothing, and nothing
renders as an empty page rather than as an error. When you add an endpoint,
**measure its envelope** and add a capture (§6).

---

## 5. Local development

```bash
pnpm install --frozen-lockfile
pnpm run build:wasm        # once — writes public/island/, which is gitignored
pnpm dev                   # astro dev on the static chrome
```

`pnpm dev` does **not** rebuild the island. Astro's dev server serves `public/`
as-is, so after any change under `src/*.rs` you must re-run `pnpm run build:wasm`
and reload. If you skip the first `build:wasm`, every island page shows
"Could not start the console: …" — which is the failure being reported honestly,
not a bug.

The console reads the live public API in dev exactly as it does in production.
There is no fixture server and nothing to configure.

---

## 6. Tests

```bash
cargo test
```

Two kinds, and the difference is the point:

- **Unit tests in `src/api.rs`** decode fixtures this repo wrote. They prove the
  types are self-consistent and nothing more. The Svelte client this replaces had
  types that were self-consistent and wrong.
- **`tests/live_shapes.rs`** decodes **captured live responses** under
  `tests/live/` with the production types. That is a different and stronger claim.

The captures are refreshed by hand:

```bash
for e in ui brand architecture; do
  curl -s "https://api.mzizi.dev/v1/$e" -o "tests/live/$e.json"
done
```

Capture through the **same base the console reads**. Capturing from the apex
while the client reads the gateway would mean the live-shape tests stop testing
the path that is actually in use — which is the exact failure mode `tests/live/`
exists to catch.

**Do not prettify, sort, or trim them.** Reformatting turns them into fixtures
this repo wrote rather than bytes the API sent, and that distinction is the only
reason they are worth having. The monorepo this was extracted from held the line
with a `.prettierignore` entry; that file did not come across, so nothing
currently enforces it — adding one would be a small, useful contribution.

If you add a type that decodes an API response, add a live capture for it. A
`#[serde(default)]` that quietly swallows a renamed field is exactly the failure
mode `live_shapes.rs` exists to catch, and it catches nothing for an endpoint
with no capture.

---

## 7. What CI enforces

Three jobs, all required. Run them locally with `pnpm run check` (both halves)
plus `pnpm test`.

**`rust`**

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo check --target wasm32-unknown-unknown --all-targets
```

The wasm32 check is the one that matters, and it is separate from the native
build on purpose: code can pass `cargo check` and fail to compile for the
browser. The crate also sets `unsafe_code = "forbid"`, `missing_docs = "warn"`,
and denies `unwrap_used` / `expect_used` / `todo` — so in tests use
`unwrap_or_else(|e| panic!("…: {e}"))` rather than `.unwrap()`.

**`web`**

```bash
pnpm exec astro check     # typechecks pages and frontmatter
pnpm exec astro build
```

Note what this does **not** cover — see §3.

**`secret scan`** — `gitleaks detect` over full history. The repo has no secrets
and should acquire none (see [SECURITY.md](SECURITY.md)); the scan is there so
that stays true.

CI runs on pushes to `main` and on pull requests targeting `main` or any
`claude/**` branch. The second one is not decoration: stacked pull requests target the
branch below them, and a `[main]`-only filter gives every layer but the bottom
zero checks — a PR with nothing run looks identical to a passing one.

---

## 8. Merging: merge commits only

**Squash and rebase merging are disabled on this repository**, org-wide policy,
and the setting is enforced by GitHub rather than by convention
(`squashMergeAllowed: false`, `rebaseMergeAllowed: false`).

From `mzizi/MIGRATION.md` §1.1, which sets the repository spec for the org:

> | Allow merge commits | **yes** | The ecosystem convention is merge-only; history stays truthful |
> | Allow squash merging | **no** | Squash discards the per-commit reasoning this project depends on |

Merge with:

```bash
gh pr merge <n> --merge --delete-branch
```

This project's commit messages carry the measurement behind a change — which
host answered, which status a route returns, which limit a bundle is up against.
`52218dc` is a worked example: it is the record of _why_ two CI jobs failed and
what the underlying cause was, and a squash would have folded it into a feature
commit and thrown that away.

Practical consequences:

- **Write commit messages for a future reader.** State what you measured, not
  just what you changed. "Points at the address that answers" plus the two
  probe results beats "fix API URL".
- **Do not tidy history before merge.** No interactive rebase, no
  `--amend` on pushed commits, no force-push to a branch under review.
- Branches are deleted on merge automatically.

---

## 9. Deploying

You almost certainly are not deploying. The console **is** live at
`app.mzizi.dev`, published through the Cloudflare GitHub app rather than from
this repo. See [README.md § Deploying](README.md#deploying) for the full
picture, including what the first production deploy did to DNS.

There is no deploy workflow here and there should not be one: CI does build
checks and tests, not publishing. Do not add a `wrangler deploy` step to
`ci.yml`.

Do not change `wrangler.jsonc`'s route without reading the comment above it
first. A custom domain takes a **bare hostname**; the wildcard form the console
arrived with is rejected by the API, which is why `dashboard.mzizi.dev` never
existed. Worker Builds previews upload a version _without_ applying routes, so a
broken route config reads green on a PR and fails only on the production deploy.

---

## 10. Reporting and conduct

- **Bugs and proposals** — open an issue. Include what you measured: the URL,
  the status, the versions of `cargo`/`wasm-bindgen`/`node` if it is a build
  problem.
- **Security** — [SECURITY.md](SECURITY.md). Do not open a public issue.
- **Conduct** — [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), Contributor Covenant
  2.1, reports to `conduct@nyuchi.com`.

Licensed under Apache-2.0. By contributing you agree your contributions are
licensed under the same terms.
