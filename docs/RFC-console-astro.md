# RFC: Rebuild the Mzizi console on Astro with a Rust backend and Mzizi Roots

**Status:** proposed, 2026-10-20. **Tracking:** mzizi-dev/mzizi-console#34.
**Scope:** `app.mzizi.dev`, this repository only. **Amends:** `README.md` ("the split") and `AGENTS.md` ("Toolchain"), which describe the Astro-shell-plus-Dioxus-islands design this RFC replaces.

## 0. Owner decisions (verbatim)

- 2026-10-20: "The mzizi console needs to be built on Astro with rust in the backend and primitives where needed".
- 2026-10-20: "Mzizi components are rust not borrowed from another rust version like dioxus. We build our own completely."
- 2026-10-20 (scope): "No the mzizi console is standalone. And billing will run through the nyuchi console. For services but what the mzizi console does is different from the nyuchi console."

Consequences of the third decision, binding on every PR in the plan:

1. `app.mzizi.dev` is a **standalone** app. It is not mounted inside the Nyuchi Console and does not share its shell, routes or auth session.
2. **Billing for services stays in the Nyuchi Console.** This console builds no billing, invoice, checkout, payment-method or subscription flow. At most it links out to the Nyuchi Console for billing.
3. The console's scope is its own: the Mzizi registry (components), design tokens, the DNA-helix architecture view and developer tooling. Its feature set is not modelled on the Nyuchi Console's.

## 1. Survey: what the console is today

Measured against this repository at `origin/main` 9ff0c80. The only commit visible in the shallow clone is the release merge, so history is not surveyed.

**The app is already Astro plus Dioxus islands, not a pure Dioxus app.** The brief described a Rust/Dioxus WASM app. The repository is an Astro static site (`output: "static"`) whose four pages are real `.astro` files, with Dioxus islands mounted into them. The Dioxus part is smaller than the brief implied: five `#[component]` functions in `src/island.rs` (`Root`, `IslandRoot`, `ComponentsIsland`, `TokensIsland`, `ArchitectureIsland`) and one `rsx!`-based entry in `src/main.rs`.

### 1.1 Routes and pages

| Route           | Source                                                 | Gated | Data                                    | Needs browser behaviour           |
| --------------- | ------------------------------------------------------ | ----- | --------------------------------------- | --------------------------------- |
| `/`             | `src/pages/index.astro`                                | no    | none (four static links)                | no                                |
| `/components`   | `src/pages/components.astro` + island `components`     | yes   | `GET /v1/ui` (registry index)           | data only, rendered in the island |
| `/tokens`       | `src/pages/tokens.astro` + island `tokens`             | yes   | `GET /v1/brand`                         | data only, rendered in the island |
| `/architecture` | `src/pages/architecture.astro` + island `architecture` | yes   | `GET /v1/architecture` (helix envelope) | data only, rendered in the island |
| `/callback`     | `src/pages/callback.astro`                             | no    | AuthKit code exchange in the browser    | yes (auth)                        |
| `/404`          | `src/pages/404.astro`                                  | no    | none                                    | no                                |

Every page shares `src/layouts/Console.astro` (nav, headings, the AuthKit gate script). `Island.astro` mounts the bundle into `<div id="island" data-island="…">`, with a `<noscript>` fallback that names the page on the public site.

### 1.2 Islands and what each does

| Island               | What it renders                                                                                                                         | Data                                 | Browser behaviour it needs |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------ | -------------------------- |
| `ComponentsIsland`   | Count, components grouped by helix node (N-number, or "No node"), name, description, and a `shadcn add` command built from the API base | `/v1/ui`                             | none beyond display        |
| `TokensIsland`       | Count, seven minerals, two colour swatches per mineral painted from `lightHex`/`darkHex`, CSS variable, symbolism, origin               | `/v1/brand`                          | none beyond display        |
| `ArchitectureIsland` | Counts; strands sorted by `sort_order`; nodes; rungs (listed apart, because they belong to neither backbone)                            | `/v1/architecture` (`data` envelope) | none beyond display        |
| `Root`               | Reads `data-island` and picks one of the three; unknown name renders an error                                                           | none                                 | none                       |

The only interaction in the whole console is sign-in, sign-out and the copy of an install command (the command is rendered as text; no copy button exists in the source). **Nothing in the console needs client-side reactivity.** The three islands are `use_resource` fetches with a loading/error/ready match, which is what `resolved!` in `island.rs` does.

### 1.3 Data sources

- **Mzizi registry API**, read live from the browser today: `https://api.mzizi.dev/v1` (`DEFAULT_API_BASE` in `src/api.rs`). Endpoints: `/ui`, `/brand`, `/architecture`. `/architecture/frontend/axes` and `/layers` answer 410 Gone and are not read.
- **WorkOS AuthKit**, client-side via `@workos-inc/authkit-js`, with `PUBLIC_WORKOS_CLIENT_ID` (public) and optional `PUBLIC_WORKOS_API_HOSTNAME`, `PUBLIC_WORKOS_CALLBACK_PATH`, `PUBLIC_WORKOS_DEV_MODE`. The gate is client-side only, and the repository says so: it is not an authorization boundary.
- **Supabase: not used.** The brief named Supabase access. A search of this repository finds no Supabase client, environment variable or migration. The only mentions are in test-capture JSON (registry descriptions) and `AGENTS.md`, which says no database sits behind the registry. This RFC does not design a Supabase path. If the owner expects one, that is an open question (§7).
- **Billing: none.** Per decision 3 above.

### 1.4 Build, deploy and tests

- **Build.** `pnpm run build` = `build:wasm` (`scripts/build-island.sh`: `cargo build --release --target wasm32-unknown-unknown`, `wasm-bindgen --target web`, `wasm-opt -Oz`) then `astro build`. `scripts/bootstrap-toolchain.sh` installs Rust, `wasm-bindgen-cli` and `wasm-opt` on Cloudflare Workers Builds, and only when `WORKERS_CI` is set.
- **Deploy.** `wrangler.jsonc` has no `main`. It is a Worker with Static Assets (`./dist`, `not_found_handling: 404-page`) and a custom domain route `app.mzizi.dev` (bare hostname, as the repository's own comments require). Workers Builds deploys it; the repository holds no deploy workflow.
- **Size, as documented.** `AGENTS.md` records about 1.0 MiB for the island bundle (`mzizi-console_bg.wasm` 921 KiB, `mzizi-console.js` 74 KiB, `snippets/` 25 KiB). **This RFC did not measure it.** No `wasm32` build was run (the disk has 3.9 GB free and `wasm-bindgen` and `wasm-opt` are not installed). Treat the figure as documented, not measured, until PR 7 of the plan measures it.
- **CI** (`.github/workflows/ci.yml`): `rust` (fmt, clippy, `cargo test`, `cargo check --target wasm32`), `web` (`pnpm install --frozen-lockfile`, `astro check`, `vp lint`, `astro build`), `secret scan` (gitleaks). `web` builds without the island bundle, so it does not prove the two toolchains compose. The build does, and it is not in CI.
- **Tests.** `src/api.rs` has unit tests over fixtures this repository wrote (join, install command, decoding of `/ui`, `/brand`, `/architecture`, the 410 error). `tests/live_shapes.rs` decodes three captured responses under `tests/live/` with the production types. Those captures must not be reformatted (`AGENTS.md`). `src/island.rs` has two tests on `Island::parse`.
- **Staging.** The repository has no `staging` branch. PRs go to `main`, as the brief said.

### 1.5 What needs browser behaviour and what is static

- **Static, no JS needed:** the overview, the nav, every heading and lede, the 404 page, the install-command text, the descriptions and the palette's names and values (all of which are data but are rendered from a list).
- **Data, which today needs JS to show:** the three data pages. This is the main defect the rebuild fixes. With JavaScript off, every data page shows the `<noscript>` sentence, not the registry.
- **Browser behaviour needed:** only auth. Sign-in and sign-out can be plain links to Worker routes (§3.4), so nothing needs a browser WASM module.

## 2. Target architecture

### 2.1 Decision: one Worker written in Rust, serving Astro's output

**Recommendation: a single Cloudflare Worker whose entry point is written in Rust with `workers-rs` (the `worker` crate), compiled to `wasm32` and run server-side. Astro builds the pages to static HTML in `dist/`, and the Worker serves them as Static Assets through the `ASSETS` binding.** The Worker handles every request that needs live data, auth or an API call.

Why this and not the alternatives:

- **Astro SSR on the `@astrojs/cloudflare` adapter** would render pages in JavaScript inside the Worker. The brief asks for the backend in Rust, so JS would own the request path. It also adds an adapter and a server output mode the console has so far avoided. Rejected.
- **Two Workers (a Rust API Worker plus a static Astro Worker).** Works, but it needs a service binding, two deploys and two route configurations, and the console is one custom domain. Rejected for the first cut; it stays available if the API and the site ever need separate deploys.
- **Client-side fetch from the browser (today's model).** Pages need JavaScript to show data, and that breaks "every page renders without JS". Rejected.
- **Build-time snapshot of the registry.** Pages would go stale between deploys, which `astro.config.mjs` already warns against. Rejected.

How a data page renders, on the server, with no JavaScript:

1. Astro prerenders the page shell to `dist/components.html`, with a placeholder element `<div data-fill="components">` and a `<noscript>`-free, no-JS-needed skeleton. The shell is the chrome: nav, heading, lede, footer.
2. A request for `/components` reaches the Rust Worker (`run_worker_first` lists the data routes, see §2.3).
3. The Worker fetches `GET https://api.mzizi.dev/v1/ui` with `worker::Fetch`, decodes it with the same types `src/api.rs` defines today, and renders the body with Mzizi Roots (`render(&Props) -> String`, §2.4).
4. The Worker fetches the shell from `env.ASSETS`, and `HTMLRewriter` replaces the inner content of `[data-fill="components"]` with the rendered string (`set_inner_content` with HTML). Cloudflare documents this pattern for Static Assets: the Worker fetches the asset, and HTMLRewriter transforms it before it is returned.
5. The response is HTML with the registry in it. No WASM and no script are required.

If the API call fails, the Worker returns the shell with a server-rendered error message in the placeholder, with the HTTP status named (for example "HTTP 410 from …"), as today's island does. It does not return an empty list. It returns 200 for the page and names the upstream status in the body; a 502 is also reasonable and is an open question (§7).

### 2.2 Rust in the Worker, and what runs where

- **Worker (server, wasm32, workers-rs):** routing, the registry client (`api.rs` moved over unchanged in types), the render calls, auth and session, the HTMLRewriter splice. No Tokio: the Worker runs on the Workers runtime's futures, and Tokio is server-only and not needed here. `tests/live_shapes.rs` stays a native `cargo test`.
- **Browser:** no WASM at all for any page in the target state. Sign-in and sign-out are links to Worker routes. If a later page needs a small interaction (copying an install command), it is written as a few lines of vanilla JS in an Astro `<script>`, not WASM. The owner's rule allows at most one WASM module per page, attached through `web-sys` to a `data-slot` root; the console needs none. The zero-WASM target is a choice, not a limit: a later page that needs WASM may add one module under the page-budget rule.

### 2.3 Routing and config

Target `wrangler.jsonc` shape (PR 2):

```jsonc
{
  "name": "mzizi-console",
  "main": "build/index.js",          // worker-build output from the Rust crate
  "compatibility_date": "2026-05-22",
  "assets": {
    "directory": "./dist",
    "binding": "ASSETS",
    "not_found_handling": "404-page",
    "run_worker_first": ["/", "/components", "/tokens", "/architecture", "/auth/*", "/healthz"]
  },
  "routes": [{ "pattern": "app.mzizi.dev", "custom_domain": true }]
}
```

- `run_worker_first` takes an array of patterns (Cloudflare docs: up to 100 entries, `*` globs, `!` for exceptions). The data routes are listed; every other path (the chrome-only pages and assets) is served as a static asset without invoking the Worker.
- The route stays a bare hostname, as the repository's existing comment requires.
- `not_found_handling: 404-page` stays. `dist/404.html` is served by the asset server for unknown paths, and the Worker is not invoked for them.

### 2.4 Mzizi Roots: the component layer

Components are Mzizi's own Rust, following `docs/roots/RFC-roots-renderer.md` (PR #506 in `mzizi-dev/mzizi-registry`, branch `claude/roots-own-renderer`):

- Each component exposes `pub fn render(props: &Props) -> String`, with no Dioxus, Leptos, Yew or other UI framework. The shared HTML writer (`escape_into`, `open`, `close`) is Mzizi's own and matches the escaping the renderer RFC specifies.
- The console's own presentational pieces (the card, the list row, the palette swatch, the error state, the page section) are Roots components where a Roots component exists. Where none exists yet, the console writes the markup in its own small render functions and records the gap in §6 rather than importing a framework.
- **The console depends on mzizi-ui through a pinned version or git revision, not on Dioxus.** The renderer RFC's PR 0 added `skeleton::render` and the `html` writer; PRs 1 to 6 are the rest. Until the crates are published, the console can take a git dependency pinned to a commit that has `render` (open question, §7).

Behaviour, per the renderer RFC §3.3: a component that needs interaction gets `attach(root, state)` behind a `web` feature. The console has no such component in its target state, so it ships no WASM.

### 2.5 Auth and session

Today: AuthKit runs in the browser with PKCE, and the gate is client-side. That cannot be made server-side without a secret, and the repository's `authkit.ts` says a change that needs a client secret is wrong. Moving the flow server-side changes that rule on purpose, so it is decided here and not by accident:

- **Recommended:** a server-side AuthKit authorization-code flow in the Worker. `/auth/signin` redirects to WorkOS. `/auth/callback` exchanges the code using a Worker **secret** (`WORKOS_CLIENT_SECRET`, set in the dashboard, never in the repository) and sets an `HttpOnly; Secure; SameSite=Lax` session cookie. The gate is then a server check: without a valid session the data routes return the sign-in page, and the data is never fetched.
- Until this is decided, PR 2 to PR 4 keep the current client gate working. The data pages still gate client-side; the Worker-rendered content is then visible to anyone who fetches the HTML. Since the content is public registry data, that is acceptable for a short transition. The gate is not an authorization boundary (repository comment in `Console.astro`), and this RFC does not change that statement.
- **Billing and the Nyuchi Console:** the session is for this console only. It does not share a cookie or shell with the Nyuchi Console, per decision 1.

### 2.6 The page WASM budget

Target: **0 bytes of WASM on every page.** A budget of one WASM module per page, if ever needed, is set in PR 7 after measurement, per the renderer RFC's decision 6.2. Until a page has a measured WASM payload, no budget number is claimed.

## 3. Operational notes

### 3.1 Build

- Rust side: `cargo` for `wasm32-unknown-unknown` with `worker-build` (the workers-rs build tool that emits the Worker's JS shim and wasm), pinned in `Cargo.toml` and `Cargo.lock`. Native `cargo test` covers decoding and render.
- Astro side: unchanged (`astro build` to `dist/`), with exact-version pins (§7).
- The build order is Astro first (it writes `dist/`), then the Worker. The Worker's assets directory is `dist/`.

### 3.2 Tests

- `cargo test` (native): the API types, the live-shape captures, the render functions (golden HTML for each page's data fragment), the splice (HTMLRewriter over a fixture shell) and the error-body cases.
- `cargo check --target wasm32-unknown-unknown` (already in CI).
- `astro check` and `astro build` (already in CI).
- A new job runs the full build (Astro, then `worker-build`) so the two toolchains are proven to compose, which CI does not do today.

### 3.3 Dioxus removal

Dioxus is removed in the last PR of the plan (PR 6). Until then the existing islands keep working, because the new routes sit beside them. `src/island.rs`, `src/main.rs`, `scripts/build-island.sh`, `scripts/bootstrap-toolchain.sh` (which exists for the Dioxus WASM build) and `public/island/` go in that PR.

### 3.4 Sign-in and sign-out without JavaScript

Sign-in and sign-out are `<a href="/auth/signin">` and `<a href="/auth/signout">`, because the server does the redirect. This removes the `authkit-js` dependency from the browser and the client gate script.

## 4. Billing boundary (owner decision 3)

- This console contains **no** billing, invoice, checkout, payment-method, subscription, plan or usage-metering flow.
- If a page needs to point at billing for a service, it links to the Nyuchi Console with a plain link. No billing state is read or stored here.
- No Nyuchi Console routes, components, styles or auth state are imported. The Mzizi console is not modelled on the Nyuchi Console's feature list.

## 5. Acceptance criteria

1. Every page renders its full content with JavaScript disabled, including the three data pages (verified by fetching the page and asserting the registry content is in the HTML; no script runs).
2. A per-page WASM size table in the repository: each page, its WASM bytes (0 for the target), its JS bytes. Produced by a script in CI.
3. `Cargo.toml` contains no `dioxus` dependency and no `dioxus` feature. A CI check (`cargo tree -i dioxus` is empty) enforces it.
4. The same data as today is served, proven by tests: the captured live responses decode and render through the new render path; the rendered counts match the source counts (components, minerals, nodes, rungs, strands).
5. The 410 error case names the status in the rendered body.
6. No billing, payment or subscription markup exists in the repository (a grep gate).
7. `pnpm run build` produces a working site (Astro plus Worker) in CI.
8. Nothing in the repository holds a secret value. Secrets live in the Worker dashboard only.

## 6. Known gaps

- The renderer RFC's migration (its PRs 1 to 6) is not finished on `main` of `mzizi-registry`. The console depends on it being at least PR 0, and on its PR 1 for the Roots primitives the console uses.
- No Roots component exists yet for every console element. Gaps are written into the issue and tracked, not papered over with a framework.
- The 1.0 MiB size is documented but not measured here.

## 7. Open decisions

See the issue for the list with recommendations. Summary:

1. Worker runtime: single workers-rs Worker serving Astro's output (recommended).
2. Data rendering: request-time server render with HTMLRewriter splice (recommended).
3. Auth: server-side AuthKit code flow with an HttpOnly session cookie (recommended), or keep the client SDK for now.
4. Roots dependency: git pin to a commit with `render` now, crates.io version later (recommended), or wait for mzizi-ui 0.4.0 (owner approves publish).
5. Exact-version pins for `package.json` (recommended), since the current ranges are `^`.
6. Upstream failure response code: 200 with the error in the body (recommended, matches today) or 502.
7. Supabase: confirm there is no Supabase path (none exists in this repository).
8. Workers Builds failure (~10 seconds in): the cause is unknown; needs the owner's build log.

## 8. Plan

See the issue for the numbered PR plan. PR 1 is this document and a CHANGELOG entry. No code changes ship in PR 1, so the Dioxus app is untouched.
