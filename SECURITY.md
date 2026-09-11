# Security Policy

## What this is, honestly

`mzizi-console` is a **read-only browser of a public registry**. It is worth
stating its actual shape before anything else, because a security policy that
implies a threat model a project does not have is worse than none — it sends
researchers looking for an attack surface that is not there, and it lets the
real one hide behind boilerplate.

As of today:

- **No authentication.** There is no login, no session, no cookie, no token.
- **No secrets.** No environment variables, no bindings, no KV, no D1, no R2.
  `wrangler.jsonc` declares a static-asset directory and nothing else. The
  `secret scan` CI job runs `gitleaks` over full history to keep that true.
- **No backend of its own.** The Worker has no `main` — it is static assets and
  Cloudflare's asset server. There is no code path on the server to attack.
- **No user data.** The console collects nothing, stores nothing, and sets no
  cookies. There is no analytics script and no third-party embed.
- **No writes.** Every request it makes is a `GET` against the public registry
  API at `https://api.mzizi.dev/v1` — the same data any reader can `curl`. The
  gateway itself advertises only `GET, OPTIONS`, so there is no write verb to
  reach through it.
- **Deployed** at `https://app.mzizi.dev`, as a Worker with static assets and no
  code of its own.

The realistic risk from this repository is **supply-chain and integrity**, not
confidentiality: something that changes what bytes a visitor's browser executes,
or what a visitor is told to run on their own machine.

## In scope

Report these:

- **Anything that changes what the page executes.** A dependency (npm or crate)
  that ships malicious code into `dist/`; a way to make the island bundle load
  from somewhere other than `/island/`; a compromise of the build path in
  `scripts/build-island.sh`.
- **Cross-site scripting via registry content.** The islands render strings the
  API supplies — component names, titles, descriptions, token values, helix node
  labels. A registry document that escapes into markup or script is in scope
  here even though the content originates in
  [`mzizi-dev/mzizi`](https://github.com/mzizi-dev/mzizi).
- **A bad install command.** `api::install_command_for` builds the
  `npx shadcn@latest add …` string the Components island displays for a reader to
  copy and run. Any way to make that string point somewhere unintended is the
  highest-severity class this repository has — the end of that path is code
  executing on a developer's machine, on their say-so but on our text.
- **GitHub Actions issues** in `.github/workflows/ci.yml` — malicious-input,
  token exfiltration, privilege escalation. Every job declares
  `permissions: contents: read`.
- **Deployment configuration** that would expose something: a `wrangler.jsonc`
  change adding a binding, a secret, or a route that is not `app.mzizi.dev`.

## Out of scope

- **The registry API itself** (the Next.js origin the gateway proxies to,
  currently `mzizi.dev/api/v1`) — report to
  [`mzizi-dev/mzizi`](https://github.com/mzizi-dev/mzizi) under its own
  `SECURITY.md`. The console is a consumer.
- **The API gateway** (`api.mzizi.dev`) —
  [`mzizi-dev/mzizi-api-gateway`](https://github.com/mzizi-dev/mzizi-api-gateway).
  This is the console's only upstream: `api::DEFAULT_API_BASE` reads through it.
- **Missing security headers, CSP, SRI.** These are worth having and are welcome
  as issues or pull requests. (This exclusion previously rested on the console
  not being deployed; `app.mzizi.dev` is live now, so the reasoning no longer
  holds and the item is kept only as a routing note — send them as issues, not
  as private security reports.)
- **Denial of service against Cloudflare's edge.** Cloudflare owns that.
- **Findings that require a compromised maintainer account** as a precondition.
- **Social engineering or physical attacks** against maintainers.

## Reporting

**Do not open a public issue or pull request for a security problem.**

Use GitHub's private advisory flow:

1. <https://github.com/mzizi-dev/mzizi-console/security/advisories/new>
2. Include: what you did, what happened, what you expected, the commit SHA, and
   the impact — who is exposed and how.
3. Submit. Maintainers are notified privately.

If GitHub advisories are unavailable to you, email `security@nyuchi.com` with the
same information. PGP is not required.

## Response

| Stage                                  | Target                                             |
| -------------------------------------- | -------------------------------------------------- |
| Acknowledgement                        | Within 5 business days                             |
| Triage and reproduction                | Within 10 business days                            |
| Fix merged to `main` (critical / high) | Within 14 days of triage                           |
| Fix merged to `main` (medium / low)    | Best effort                                        |
| Public disclosure                      | After the fix ships, coordinated with the reporter |

These are the targets for a small, pre-deployment project maintained alongside
others. They are deliberately less aggressive than
[`mzizi-dev/mzizi`](https://github.com/mzizi-dev/mzizi)'s, which serves live
traffic. If a report turns out to affect a deployed surface, it inherits that
repository's timeline instead — say so in the advisory and it will be escalated.

Reporters are credited by name or handle in the advisory unless they ask not to
be. Good-faith research within this policy will not be met with legal action.

## This will change

The threat model above describes the console **as it exists today**, and the
things already planned for it will invalidate most of it:

- **Fundi task views.** The registry's self-healing surface (`/api/v1/fundi`)
  carries operational state about a live system. Reading it in a browser is a
  different proposition from reading a public component list, and it is the
  first thing here that will plausibly need to be gated.
- **An admin path.** Anything that writes to the registry means authentication,
  a session, a token with real authority, and a CSRF surface — none of which
  exist today.

When either lands, this file must be rewritten _in the same pull request_, not
afterwards. A policy that says "no auth, no secrets, no writes" while the code
has all three is worse than no policy at all, and the gap between them will not
be noticed by anyone reading only the code.

## Dependencies

- Rust dependencies are pinned by `Cargo.lock`, which is committed. The crate
  sets `unsafe_code = "forbid"`.
- JS dependencies are pinned by `pnpm-lock.yaml` and installed in CI with
  `--frozen-lockfile`. They are build-time only — **no npm package ships to the
  browser**; the only JavaScript served is the wasm-bindgen glue and the small
  inline loader in `src/layouts/Island.astro`.
- `gitleaks detect --source . --redact` runs over full history on every push and
  pull request.
- There is no `cargo audit` or `pnpm audit` gate yet. Adding one would be a
  welcome contribution — see [CONTRIBUTING.md](CONTRIBUTING.md).

## Contact

- Primary: <https://github.com/mzizi-dev/mzizi-console/security/advisories/new>
- Fallback: `security@nyuchi.com`
