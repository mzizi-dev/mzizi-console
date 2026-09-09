//! Entry point for the console's WASM island bundle.
//!
//! Astro renders every page's chrome as static HTML and mounts this only where
//! a page actually needs live data. So this is not an app shell: there is no
//! router, no layout, and no navigation here — Astro owns all three.
//!
//! One bundle serves all three islands, selected by the mount point's
//! `data-island` attribute, because the Dioxus runtime is most of the bytes and
//! three bundles would ship it three times.

use mzizi_dashboard::island::Root;

fn main() {
    dioxus::logger::initialize_default();
    // `launch` takes a plain `fn() -> Element`, so the island name is read
    // inside `Root` rather than captured here. That is also the better place
    // for it: the component that renders the mount point is the one that should
    // know what the mount point asked for.
    dioxus::launch(Root);
}
