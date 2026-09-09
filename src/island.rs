//! The Rust half: the islands Astro mounts.
//!
//! Astro renders the chrome — nav, headings, prose — as static HTML. These
//! render the DATA, fetched from the API at runtime, so the console shows the
//! live registry rather than a snapshot frozen at deploy time. A console whose
//! component list was baked at build time would go stale the moment a component
//! shipped, and a stale copy that still looks authoritative is the defect class
//! this ecosystem keeps removing.
//!
//! One bundle serves all three islands, selected by the mount point's
//! `data-island` attribute. Three bundles would ship the Dioxus runtime three
//! times, and the runtime is most of the bytes.

use dioxus::prelude::*;

use crate::api::{
    Architecture, Brand, ComponentSummary, Envelope, RegistryIndex, install_command_for, join,
};

/// Which island a mount point asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Island {
    /// The registry browser.
    Components,
    /// The palette.
    Tokens,
    /// The DNA double helix.
    Architecture,
}

impl Island {
    /// Read an island name off the mount point.
    ///
    /// Returns `None` rather than defaulting, and the caller renders an error.
    /// A typo'd `data-island` that silently rendered the component browser
    /// would look like a working page on the wrong route — worse than a message
    /// naming the attribute that is wrong.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "components" => Some(Self::Components),
            "tokens" => Some(Self::Tokens),
            "architecture" => Some(Self::Architecture),
            _ => None,
        }
    }
}

/// Fetch and decode one endpoint.
async fn get<T: serde::de::DeserializeOwned>(base: &str, path: &str) -> Result<T, String> {
    let url = join(base, path);
    let response = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    let status = response.status();
    if !status.is_success() {
        // The status stays in the message. `/architecture/frontend/axes`
        // answers 410 Gone, and "HTTP 410" tells a reader the route is retired;
        // a bare "failed to load" sends them looking for an outage.
        return Err(format!("HTTP {} from {url}", status.as_u16()));
    }
    response.json::<T>().await.map_err(|e| e.to_string())
}

/// The element `Island.astro` renders for the bundle to mount into.
pub const MOUNT_ID: &str = "island";

/// The bundle's root.
///
/// Reads `data-island` off the mount point and renders that one view. An
/// unknown name renders a message naming it rather than falling back to a
/// default: a typo that silently rendered the component browser would look like
/// a working page on the wrong route.
#[component]
pub fn Root() -> Element {
    let name = mount_island_name().unwrap_or_default();
    match Island::parse(&name) {
        Some(island) => rsx! {
            IslandRoot { island, base: crate::api::DEFAULT_API_BASE.to_string() }
        },
        None => rsx! {
            p { class: "state error",
                "Unknown island “{name}”. Expected one of: components, tokens, architecture."
            }
        },
    }
}

/// Read `data-island` off the mount point, if there is one.
#[cfg(target_arch = "wasm32")]
fn mount_island_name() -> Option<String> {
    web_sys::window()?
        .document()?
        .get_element_by_id(MOUNT_ID)?
        .get_attribute("data-island")
}

// Off the web there is no document to read. This exists so `cargo check`,
// `clippy` and the test runner can build natively — otherwise a native-only
// mistake here would not surface until the WASM build.
#[cfg(not(target_arch = "wasm32"))]
fn mount_island_name() -> Option<String> {
    None
}

/// Render whichever island was asked for.
#[component]
pub fn IslandRoot(island: Island, base: String) -> Element {
    match island {
        Island::Components => rsx! { ComponentsIsland { base } },
        Island::Tokens => rsx! { TokensIsland { base } },
        Island::Architecture => rsx! { ArchitectureIsland { base } },
    }
}

/// Shared loading / error / ready presentation.
///
/// Factored out because the three islands differ only in what they render on
/// success, and because the error branch is the one worth having in exactly one
/// place: the Svelte console this replaces rendered an empty list on failure, so
/// a 410 and an empty registry were indistinguishable.
macro_rules! resolved {
    ($resource:expr, $data:ident => $body:expr) => {
        match &*$resource.read_unchecked() {
            None => rsx! { p { class: "state", "Loading…" } },
            Some(Err(e)) => rsx! { p { class: "state error", "Could not load: {e}" } },
            Some(Ok($data)) => $body,
        }
    };
}

/// The registry, grouped by helix node.
#[component]
fn ComponentsIsland(base: String) -> Element {
    let loaded = use_resource({
        let base = base.clone();
        move || {
            let base = base.clone();
            async move { get::<RegistryIndex>(&base, "ui").await }
        }
    });

    resolved!(loaded, index => {
        // Grouped by node, groups ordered by node NUMBER. The N-numbers are
        // labels rather than a sequence, but within one list a numeric order is
        // at least stable — sorting the labels as strings would put N10 and N11
        // between N1 and N2.
        let mut groups: Vec<(Option<u32>, Vec<&ComponentSummary>)> = Vec::new();
        for item in &index.items {
            match groups.iter_mut().find(|(n, _)| *n == item.node) {
                Some((_, bucket)) => bucket.push(item),
                None => groups.push((item.node, vec![item])),
            }
        }
        groups.sort_by_key(|(n, _)| n.unwrap_or(u32::MAX));

        rsx! {
            p { class: "count", "{index.items.len()} components" }
            for (node, bucket) in groups {
                section { class: "node-group",
                    h2 {
                        match node {
                            Some(n) => format!("N{n}"),
                            // Not "N0", and not hidden. A component with no node
                            // is a registry defect worth seeing, not smoothing
                            // into a plausible-looking bucket.
                            None => "No node".to_string(),
                        }
                    }
                    ul { class: "component-list",
                        for item in bucket {
                            li { key: "{item.name}", class: "card",
                                strong { "{item.name}" }
                                if let Some(d) = &item.description {
                                    p { class: "muted", "{d}" }
                                }
                                pre { class: "install",
                                    code { "{install_command_for(&item.name, &base)}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

/// The palette.
#[component]
fn TokensIsland(base: String) -> Element {
    let loaded = use_resource({
        let base = base.clone();
        move || {
            let base = base.clone();
            async move { get::<Brand>(&base, "brand").await }
        }
    });

    resolved!(loaded, brand => rsx! {
        p { class: "count", "{brand.minerals.len()} minerals" }
        ul { class: "mineral-list",
            for m in &brand.minerals {
                li { key: "{m.name}", class: "mineral card",
                    // The swatches are painted from the token's own values, so
                    // this page cannot show a colour the palette does not define.
                    span { class: "swatch", style: "background:{m.light_hex.clone().unwrap_or_default()}" }
                    span { class: "swatch", style: "background:{m.dark_hex.clone().unwrap_or_default()}" }
                    strong { "{m.name}" }
                    if let Some(v) = &m.css_var { code { "{v}" } }
                    if let Some(s) = &m.symbolism { p { class: "muted", "{s}" } }
                    if let Some(o) = &m.origin { p { class: "muted origin", "{o}" } }
                }
            }
        }
    })
}

/// The DNA double helix.
#[component]
fn ArchitectureIsland(base: String) -> Element {
    let loaded = use_resource({
        let base = base.clone();
        move || {
            let base = base.clone();
            async move {
                get::<Envelope<Architecture>>(&base, "architecture")
                    .await
                    .map(|e| e.data)
            }
        }
    });

    resolved!(loaded, arch => {
        let mut strands = arch.strands.clone();
        strands.sort_by_key(|s| s.sort_order);
        rsx! {
            p { class: "count",
                "{arch.nodes.len()} nodes · {arch.rungs.len()} rungs · {strands.len()} strands"
            }
            h2 { "Strands" }
            ul { class: "node-list",
                for s in &strands {
                    li { key: "{s.name}", class: "card",
                        strong { "{s.title}" }
                        if let Some(b) = &s.backbone { span { class: "muted", " · {b}" } }
                        if let Some(d) = &s.description { p { class: "muted", "{d}" } }
                    }
                }
            }
            h2 { "Nodes" }
            ul { class: "node-list",
                for n in &arch.nodes {
                    li { key: "node-{n.node_number}", class: "card",
                        strong { "N{n.node_number} {n.title}" }
                        if let Some(b) = &n.backbone { span { class: "muted", " · {b}" } }
                        if let Some(d) = &n.description { p { class: "muted", "{d}" } }
                    }
                }
            }
            h2 { "Rungs" }
            // Listed separately rather than merged into the node list with an
            // empty backbone column. A rung belongs to NEITHER backbone, and
            // that is the fact the helix model turns on — rendering it as a node
            // with a missing field states the opposite.
            ul { class: "node-list",
                for n in &arch.rungs {
                    li { key: "rung-{n.node_number}", class: "card",
                        strong { "N{n.node_number} {n.title}" }
                        if let Some(d) = &n.description { p { class: "muted", "{d}" } }
                    }
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::Island;

    #[test]
    fn island_names_parse() {
        assert_eq!(Island::parse("components"), Some(Island::Components));
        assert_eq!(Island::parse("tokens"), Some(Island::Tokens));
        assert_eq!(Island::parse("architecture"), Some(Island::Architecture));
    }

    #[test]
    fn an_unknown_island_name_is_none_not_a_default() {
        // A typo'd `data-island` that silently rendered the component browser
        // would look like a working page on the wrong route.
        assert_eq!(Island::parse("component"), None);
        assert_eq!(Island::parse(""), None);
        assert_eq!(Island::parse("Components"), None);
    }
}
