//! app.mzizi.dev — the Mzizi console.
//!
//! The registry browser, the token reference and the DNA-helix architecture
//! view, as a Dioxus WASM app served from a Cloudflare Worker.
//!
//! Astro in front, Rust behind. Astro renders the chrome — nav, headings,
//! prose — as static HTML at build time; these islands render the DATA, fetched
//! from the API at runtime so the console shows the live registry rather than a
//! snapshot frozen at deploy time.
//!
//! Split into a library plus a thin `main.rs` so the parts worth testing — the
//! API client and its decoding — are testable natively with `cargo test` rather
//! than only inside a browser. The islands render what these types carry; the
//! types are where a mismatch with the live API actually shows up, and that is
//! what the tests here cover.

pub mod api;
pub mod island;
