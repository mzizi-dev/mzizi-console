//! Decode captured LIVE responses with the production types.
//!
//! The unit tests in `api.rs` decode fixtures this repo wrote, which proves the
//! types are self-consistent and nothing more. These decode bytes the live API
//! actually sent. That difference is the whole point: the Svelte client this
//! replaces had types that were self-consistent and wrong, and no test it had
//! could tell.
//!
//! The captures under `tests/live/` are refreshed by hand
//! (`curl -s https://mzizi.dev/api/v1/<endpoint> -o tests/live/<endpoint>.json`).
//! They are snapshots, so they can go stale — but a stale snapshot fails loudly
//! here, which is better than a client that discovers the drift in a browser.

use mzizi_console::api::{Architecture, Brand, Envelope, RegistryIndex};

fn load(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/live/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap_or_else(|e| panic!("missing capture {name}.json: {e}"))
}

#[test]
fn ui_index_decodes_and_is_not_empty() {
    let index: RegistryIndex =
        serde_json::from_str(&load("ui")).unwrap_or_else(|e| panic!("decode /ui: {e}"));
    // An empty list decodes fine and renders as "no components", which is how a
    // shape mismatch disguises itself as an empty registry.
    assert!(
        index.items.len() > 100,
        "expected the full registry, got {}",
        index.items.len()
    );
    assert!(index.items.iter().any(|c| c.name == "button"));
    assert!(
        index.items.iter().any(|c| c.node.is_some()),
        "no component carried a node number — `node` is probably mis-keyed"
    );
}

#[test]
fn brand_decodes_seven_minerals_with_both_hexes() {
    let brand: Brand =
        serde_json::from_str(&load("brand")).unwrap_or_else(|e| panic!("decode /brand: {e}"));
    assert_eq!(brand.minerals.len(), 7, "the palette is seven, not five");
    for m in &brand.minerals {
        assert!(m.light_hex.is_some(), "{} has no lightHex", m.name);
        assert!(m.dark_hex.is_some(), "{} has no darkHex", m.name);
        assert!(m.css_var.is_some(), "{} has no cssVar", m.name);
    }
}

#[test]
fn architecture_decodes_the_helix_behind_its_data_envelope() {
    let env: Envelope<Architecture> = serde_json::from_str(&load("architecture"))
        .unwrap_or_else(|e| panic!("decode /architecture: {e}"));
    let arch = env.data;
    assert!(!arch.nodes.is_empty(), "no nodes");
    assert!(!arch.rungs.is_empty(), "no rungs");
    assert!(!arch.strands.is_empty(), "no strands");
    // A rung crosses both backbones and belongs to neither. If every rung came
    // back with a backbone, the field is being defaulted rather than decoded.
    assert!(
        arch.rungs.iter().all(|r| r.backbone.is_none()),
        "a rung reported a backbone"
    );
}
