# Changelog

All notable changes to this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- `docs/RFC-console-astro.md`: the design for rebuilding `app.mzizi.dev` on Astro with a
  Rust backend (a single workers-rs Worker serving Astro's static output) and Mzizi Roots
  components with no Dioxus. It records the survey of the current console, the owner's
  decisions (quoted verbatim), the numbered PR plan, the acceptance criteria and the open
  decisions with recommendations. This is design only: no code changes in this entry, and
  the 1.0 MiB island size it cites is documented in `AGENTS.md`, not measured here.
- The console is standalone. Billing for services stays in the Nyuchi Console; this console
  builds no billing or payment flows.
