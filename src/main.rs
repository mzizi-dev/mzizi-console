//! Entry point for the console's WASM island bundle.
//!
//! Astro renders every page's chrome as static HTML and mounts this only where
//! a page actually needs live data. So this is not an app shell: there is no
//! router, no layout, and no navigation here — Astro owns all three.
//!
//! One bundle serves all three islands, selected by the mount point's
//! `data-island` attribute, because the Dioxus runtime is most of the bytes and
//! three bundles would ship it three times.

use mzizi_console::island::{MOUNT_ID, Root};

fn main() {
    dioxus::logger::initialize_default();

    // MOUNT AT `#island`, EXPLICITLY. This is the whole reason the console
    // rendered nothing.
    //
    // `dioxus::launch(Root)` uses the default web config, whose root element is
    // `#main`. `Island.astro` renders the mount point as `<div id="island">`,
    // and the page's only `<main>` is a TAG with no id — so the lookup missed,
    // dioxus-web logged
    //
    //     element '#main' not found. mounting to the body.
    //
    // and mounted to `<body>`. Nothing appeared: the island's `Loading…`
    // placeholder inside `#island` was never replaced, so every data page sat
    // at "Loading…" forever while the API request beside it returned 200.
    //
    // That is the worst shape this failure could take. The bundle downloads,
    // the fetch succeeds, no error is thrown, the page returns HTTP 200, and CI
    // is green — because none of those things look at what was painted. The
    // only signal was a console warning that reads like a note.
    //
    // `island::MOUNT_ID` already named the element; nothing was passing it to
    // Dioxus. Using the constant rather than a literal means the Astro
    // template and the Rust entry point cannot drift apart silently again.
    dioxus::LaunchBuilder::web()
        .with_cfg(dioxus::web::Config::new().rootname(MOUNT_ID))
        .launch(Root);
}
