//! The mzizi.dev API client.
//!
//! Types are modelled on what the API serves TODAY, which is not what the
//! Svelte console app modelled. That app's `getAxes()` and `getLayers()` called
//! `/architecture/frontend/axes` and `/architecture/frontend/layers`; both
//! answer **410 Gone** in production, and have since the axis model was
//! retired. Its Architecture route could not have worked.
//!
//! The 410 body says what replaced it:
//!
//! > The axis model is retired. Mzizi serves the DNA double helix — nodes on an
//! > engineering and a meaning backbone, held by cross-cutting rungs. This route
//! > served axis rows with a horizontal/vertical/depth/external geometry field.
//!
//! So this is a port by CONTRACT, not a translation: `AxisSummary` and the
//! flat `NodeSummary` are gone, and [`Architecture`] models nodes, rungs and
//! strands as the helix actually has them. Machine-translating the old client
//! would have produced a Rust app that compiles and 410s.

use serde::Deserialize;
/// Base URL of the Mzizi registry API.
///
/// `https://api.mzizi.dev/v1`. This constant has now held three values, and the
/// order matters, because the middle one was right at the time and is wrong now.
///
/// **As imported**, it read `https://api.mzizi.dev/v1`, justified as a move to
/// the canonical address because "the old form still resolves — the API Worker
/// accepts both". That was asserted rather than measured, and it was false in
/// both halves: there was no API Worker, and the host did not exist.
///
/// **Corrected** to `https://mzizi.dev/api/v1`, because measurement said:
///
/// ```text
/// api.mzizi.dev      -> NXDOMAIN, no DNS record at all
/// mzizi.dev/api/v1   -> 200
/// ```
///
/// The console had been pointed at a host that did not resolve, and would have
/// rendered every view empty against a perfectly healthy API. That correction
/// carried an explicit condition: switch back the day `api.mzizi.dev` actually
/// answers, and not before.
///
/// **Switched back**, because that day arrived. `mzizi-dev/mzizi-api-gateway`
/// shipped and holds `api.mzizi.dev` as a custom domain. Re-measured before this
/// change, every endpoint this client reads is byte-identical through both:
///
/// ```text
/// endpoint         api.mzizi.dev/v1    mzizi.dev/api/v1
/// /ui              200, sha d50a1a43   200, sha d50a1a43
/// /brand           200, sha d6b4d94c   200, sha d6b4d94c
/// /architecture    200, sha fb2878ac   200, sha fb2878ac
/// /ui/NAME         200, sha e1653f3c   200, sha e1653f3c
/// ```
///
/// They are identical because the gateway currently PROXIES to the apex — its
/// `/v1/health` reports `"origin": "https://mzizi.dev/api"`. So this change does
/// not yet move the console onto a different server. It moves the console onto a
/// different NAME, one the gateway owns and can repoint.
///
/// That indirection is the entire point. `mzizi.dev` is due to stop serving the
/// API — the apex is to become the static site in `mzizi-dev/mzizi-site` — and
/// anything still addressing `mzizi.dev/api/v1` on that day starts 404ing. The
/// gateway is the seam that survives the move. Do not "simplify" this back to
/// the apex: the previous value was correct only for as long as `api.mzizi.dev`
/// was missing, and it is the address with the shorter remaining life.
pub const DEFAULT_API_BASE: &str = "https://api.mzizi.dev/v1";

/// A component, as the registry index lists it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentSummary {
    /// Registry name, as typed into `shadcn add`.
    pub name: String,
    /// Human title, derived by the registry from `meta`.
    #[serde(default)]
    pub title: Option<String>,
    /// One-line description.
    #[serde(default)]
    pub description: Option<String>,
    /// Helix node number, e.g. `2` for an N2 primitive.
    #[serde(default)]
    pub node: Option<u32>,
    /// The node's short label, e.g. `primitives`.
    #[serde(default)]
    pub node_label: Option<String>,
    /// Registry categories.
    #[serde(default)]
    pub categories: Vec<String>,
    /// npm packages the component needs.
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Other registry components it composes.
    #[serde(default)]
    pub registry_dependencies: Vec<String>,
}

/// `GET /ui` — a shadcn registry document, not a bare list.
///
/// The components are under `items`, and `meta.total` is the registry size. The
/// Svelte client typed this endpoint as `ComponentSummary[]` and would have
/// failed to decode every time.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RegistryIndex {
    /// The components.
    #[serde(default)]
    pub items: Vec<ComponentSummary>,
}

/// One node of the DNA double helix.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct HelixNode {
    /// N-number. A LABEL, not a sequence — N11 does not follow N10 in any
    /// meaningful order, and sorting by it implies a progression that is not
    /// there.
    pub node_number: u32,
    /// Short tag, e.g. `tokens`.
    #[serde(default)]
    pub sub_label: Option<String>,
    /// Display title.
    pub title: String,
    /// Which backbone it sits on, when it sits on one. A rung has none, which
    /// is why this is optional rather than a defaulted empty string — "not on a
    /// backbone" and "on a backbone with no name" are different facts.
    #[serde(default)]
    pub backbone: Option<String>,
    /// The strand it belongs to, when it belongs to one.
    #[serde(default)]
    pub strand: Option<String>,
    /// What the node is for.
    #[serde(default)]
    pub description: Option<String>,
}

/// A strand of the helix.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct HelixStrand {
    /// Slug.
    pub name: String,
    /// Display title.
    pub title: String,
    /// Engineering or meaning.
    #[serde(default)]
    pub backbone: Option<String>,
    /// What the strand carries.
    #[serde(default)]
    pub description: Option<String>,
    /// Display order, as the API assigns it.
    #[serde(default)]
    pub sort_order: i64,
}

/// The helix: nodes on two backbones, plus the rungs that cross them.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
pub struct Architecture {
    /// Nodes that sit on a backbone.
    #[serde(default)]
    pub nodes: Vec<HelixNode>,
    /// Rungs — nodes that bridge both backbones and belong to neither.
    #[serde(default)]
    pub rungs: Vec<HelixNode>,
    /// The strands.
    #[serde(default)]
    pub strands: Vec<HelixStrand>,
}

/// A mineral from the brand palette.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mineral {
    /// Token name, e.g. `cobalt`.
    pub name: String,
    /// The CSS custom property that carries it, e.g. `--color-cobalt`.
    #[serde(default)]
    pub css_var: Option<String>,
    /// Light-theme hex.
    #[serde(default)]
    pub light_hex: Option<String>,
    /// Dark-theme hex.
    #[serde(default)]
    pub dark_hex: Option<String>,
    /// Where the mineral is mined — the reason the palette is named as it is.
    #[serde(default)]
    pub origin: Option<String>,
    /// What it means in the brand.
    #[serde(default)]
    pub symbolism: Option<String>,
    /// Where it is meant to be used.
    #[serde(default)]
    pub usage: Option<String>,
}

/// The brand payload.
///
/// There are **seven** minerals and seven heritage tones, not five. The Svelte
/// manifest this replaces described the Tokens route as the "Five African
/// Minerals palette" — retired naming that nyuchi/mzizi#265 removed from the
/// framework repo and added a repo-wide guard against. That guard does not
/// reach this repo, which is how the stale name survived here.
///
/// Note there is NO `data` envelope on this endpoint: the minerals are
/// top-level. See [`Architecture`], which does have one.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
pub struct Brand {
    /// The seven minerals.
    #[serde(default)]
    pub minerals: Vec<Mineral>,
}

/// A response wrapped in `{ "data": … }`.
///
/// Only SOME endpoints use this. Measured against production:
///
/// | endpoint | envelope |
/// | --- | --- |
/// | `/architecture` | `{ "data": { … }, "meta": … }` |
/// | `/ui` | none — a shadcn registry document with `items` |
/// | `/brand` | none — fields are top-level |
///
/// Three conventions across three endpoints, so the client cannot assume one.
/// This exists for the first, and the other two are decoded directly.
#[derive(Debug, Clone, Deserialize)]
pub struct Envelope<T> {
    /// The payload.
    pub data: T,
}

/// What went wrong fetching from the API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    /// The request never completed.
    Transport(String),
    /// A non-2xx response, with its status.
    Status {
        /// HTTP status.
        status: u16,
        /// The URL that produced it, so a 410 names the retired route.
        url: String,
    },
    /// The body did not match the shape this client expects.
    Decode(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transport(m) => write!(f, "request failed: {m}"),
            Self::Status { status, url } => write!(f, "HTTP {status} from {url}"),
            Self::Decode(m) => write!(f, "unexpected response shape: {m}"),
        }
    }
}

impl std::error::Error for ApiError {}

/// Join a base and a path without producing a double slash or losing a segment.
///
/// Split out and tested because the Svelte original did this inline with a
/// trailing-slash strip on the base only, which is correct exactly as long as
/// every caller remembers the leading slash on the path.
#[must_use]
pub fn join(base: &str, path: &str) -> String {
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

/// The `shadcn` command that installs a component into a consumer's project.
///
/// Built from the API base so a dashboard pointed at a preview API hands out an
/// install command for that same API — a copy button that emits a command
/// against a different deployment is worse than no copy button.
#[must_use]
pub fn install_command_for(name: &str, base: &str) -> String {
    format!(
        "npx shadcn@latest add {}",
        join(base, &format!("ui/{name}"))
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_normalises_slashes() {
        assert_eq!(join("https://a.dev/v1", "/ui"), "https://a.dev/v1/ui");
        assert_eq!(join("https://a.dev/v1/", "ui"), "https://a.dev/v1/ui");
        assert_eq!(join("https://a.dev/v1/", "/ui"), "https://a.dev/v1/ui");
    }

    #[test]
    fn install_command_follows_the_configured_base() {
        assert_eq!(
            install_command_for("button", DEFAULT_API_BASE),
            "npx shadcn@latest add https://api.mzizi.dev/v1/ui/button"
        );
        assert_eq!(
            install_command_for("button", "https://preview.example/v1/"),
            "npx shadcn@latest add https://preview.example/v1/ui/button"
        );
    }

    /// `GET /ui`, trimmed from the live response. The point of the fixture is
    /// the SHAPE — a registry document with `items`, camelCase keys — because
    /// that shape is what the Svelte client got wrong.
    const UI_FIXTURE: &str = r#"{
      "$schema": "https://ui.shadcn.com/schema/registry.json",
      "name": "mzizi",
      "items": [
        {
          "name": "accessibility-audit",
          "type": "registry:ui",
          "title": "Accessibility Audit",
          "description": "N8 assurance probe.",
          "categories": ["documentation", "assurance"],
          "dependencies": [],
          "registryDependencies": [],
          "node": 8,
          "nodeLabel": "assurance"
        },
        { "name": "button" }
      ],
      "meta": { "total": 575 }
    }"#;

    #[test]
    fn the_registry_index_decodes_from_items_not_a_bare_list() {
        let index: RegistryIndex =
            serde_json::from_str(UI_FIXTURE).unwrap_or_else(|e| panic!("decode failed: {e}"));
        assert_eq!(index.items.len(), 2);
        assert_eq!(index.items[0].node, Some(8));
        assert_eq!(index.items[0].node_label.as_deref(), Some("assurance"));
        assert_eq!(index.items[0].categories.len(), 2);
    }

    #[test]
    fn a_component_carrying_only_a_name_still_decodes() {
        // The registry OMITS optional fields rather than nulling them. A client
        // that required them would render an empty list against a healthy API —
        // a failure that looks like "no components" rather than like a bug.
        let index: RegistryIndex =
            serde_json::from_str(UI_FIXTURE).unwrap_or_else(|e| panic!("decode failed: {e}"));
        let bare = &index.items[1];
        assert_eq!(bare.name, "button");
        assert_eq!(bare.node, None);
        assert!(bare.registry_dependencies.is_empty());
    }

    #[test]
    fn brand_has_no_data_envelope_and_uses_camel_case() {
        // Both facts are load-bearing and neither is guessable: `/brand` puts
        // `minerals` at the top level, and its keys are `lightHex` / `darkHex`.
        // Assuming the `data` envelope that `/architecture` uses would decode
        // to nothing here.
        // NOTE the `r##"…"##` delimiters. A hex colour puts `"#` in the
        // literal, which closes a plain `r#"…"#` raw string mid-fixture — the
        // one syntax trap in writing colour fixtures in Rust.
        let brand: Brand = serde_json::from_str(
            r##"{"version":"4.2.0","minerals":[{"name":"cobalt","hex":"#0047AB",
                "lightHex":"#0047AB","darkHex":"#00B0FF","cssVar":"--color-cobalt",
                "origin":"Katanga (DRC) and Zambian Copperbelt",
                "symbolism":"Digital future, trust, knowledge",
                "usage":"Primary blue, links, CTAs"}]}"##,
        )
        .unwrap_or_else(|e| panic!("decode failed: {e}"));
        assert_eq!(brand.minerals.len(), 1);
        assert_eq!(brand.minerals[0].light_hex.as_deref(), Some("#0047AB"));
        assert_eq!(brand.minerals[0].dark_hex.as_deref(), Some("#00B0FF"));
        assert_eq!(brand.minerals[0].css_var.as_deref(), Some("--color-cobalt"));
    }

    #[test]
    fn architecture_decodes_the_helix_not_the_retired_axis_model() {
        // And it IS behind a `data` envelope, unlike the two above.
        let env: Envelope<Architecture> = serde_json::from_str(
            r#"{"data":{
                 "nodes":[{"node_number":2,"title":"Primitives","backbone":"engineering",
                   "strand":"build","sub_label":"primitives","description":"Generic UI."}],
                 "rungs":[{"node_number":9,"title":"Fundi","strand":null,"backbone":null}],
                 "strands":[{"name":"build","title":"Build","backbone":"engineering","sort_order":1}]
               },"meta":{"node_count":1}}"#,
        )
        .unwrap_or_else(|e| panic!("decode failed: {e}"));
        let arch = env.data;
        assert_eq!(arch.nodes.len(), 1);
        assert_eq!(arch.rungs.len(), 1);
        assert_eq!(arch.strands.len(), 1);
    }

    #[test]
    fn a_rung_has_no_backbone_and_that_survives_decoding() {
        // The live API sends `"strand": null, "backbone": null` for a rung.
        // Decoding those to `Some("")` would make a rung indistinguishable from
        // a node whose backbone is blank, and the distinction between a node on
        // a backbone and a rung crossing both is the whole helix model.
        let node: HelixNode = serde_json::from_str(
            r#"{"node_number":9,"sub_label":"fundi","title":"Fundi",
                "strand":null,"backbone":null,"description":"Self-healing."}"#,
        )
        .unwrap_or_else(|e| panic!("decode failed: {e}"));
        assert_eq!(node.backbone, None);
        assert_eq!(node.strand, None);
        assert_eq!(node.node_number, 9);
    }

    #[test]
    fn a_retired_route_reports_its_status_and_url() {
        // `/architecture/frontend/axes` answers 410. The Svelte client turned
        // every non-2xx into a bare string; keeping the status and the URL is
        // what lets the UI say "this route is retired" rather than "error".
        let err = ApiError::Status {
            status: 410,
            url: "https://api.mzizi.dev/v1/architecture/frontend/axes".into(),
        };
        assert_eq!(
            err.to_string(),
            "HTTP 410 from https://api.mzizi.dev/v1/architecture/frontend/axes"
        );
    }
}
