# Changelog

All notable changes to this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Changed — `vite-plus` 1.3.0, and every dependency is a minimum (2026-10-21)

- `vite-plus` moves from 1.0.0 to 1.3.0 (oxfmt 0.73, oxlint 1.88, Vitest 5.0.5). Only that tree moves in
  `pnpm-lock.yaml`; the console's own dependencies resolve as before.
- Every entry in `dependencies` and `devDependencies` is now `>=x.y.z` with no maximum, at the version the
  lockfile already resolved (owner, 2026-10-21: "mark packages with minimum acceptable, latest shouldn't be a
  blocker on too old of a version").
- New `pnpm.overrides`, each a minimum with no maximum, clear the high advisories the org `audit` job
  reported through `astro`: `devalue@<5.9.3` `>=5.9.3` (resolves 5.9.4), `http-cache-semantics@<4.3.0`
  `>=4.3.0`, `sharp@<0.35.5` `>=0.35.5`, `source-map-js@<1.2.2` `>=1.2.2` and `vite@>=8.0.0 <8.3.5`
  `>=8.3.5` (astro's vite moves from 8.2.2 to 8.3.5, with rolldown 1.2.15). The console renders as before.

### Added — Dependabot version updates target `staging` (2026-10-21)

- `.github/dependabot.yml` is new: weekly version updates for GitHub Actions and this repository's package manifests, every entry with `target-branch: staging` (owner rule, 2026-10-21: "all dependabot should target staging not main.") and a seven-day cooldown before a new release is proposed.

### Changed

- **The docs say where the console is going, as decided and not built** (`mzizi-console#34`; the
  RFC is open as #35). `README.md`, `AGENTS.md` and `CONTRIBUTING.md` add a section: one Rust
  Worker serves Astro's output, sign-in moves to the server (WorkOS, session cookie), any state the
  console keeps goes in Cloudflare D1, and it depends on the published Mzizi Roots crates with no
  Dioxus in new work. Today the console keeps no database, signs in with client-side AuthKit and
  renders data with Dioxus islands, and the docs still describe that as what runs. The console is
  standalone at its own domain, not a page inside the Nyuchi Console.
- **The API repository is `mzizi-dev/mzizi-api`** (formerly `mzizi-api-gateway`), renamed by the
  owner. The README's ecosystem table also corrects "a pure-Rust Worker": the API is a Hono
  (TypeScript) Worker. `AGENTS.md` gains a short "Branches" section (branch from `staging`).
- **Svelte is named only as the earlier client the console replaced.** The README, `AGENTS.md`,
  `CONTRIBUTING.md`, `Cargo.toml`, `src/api.rs`, `src/island.rs`, `tests/live_shapes.rs` and
  `.github/workflows/ci.yml` now say "the earlier client" or "an earlier mini-app" where they
  named Svelte. The notes on what each earlier client got wrong are kept. No code path, route,
  response or test expectation changes.
