// @ts-check
import { defineConfig } from "astro/config";

/**
 * Astro in front, Rust behind.
 *
 * `output: "static"` on purpose. Astro renders the CHROME — layout, navigation,
 * headings, the prose — as plain HTML at build time. It fetches nothing: the
 * data is loaded at runtime by the Rust/WASM islands, so the console shows the
 * live registry rather than a snapshot frozen at deploy time.
 *
 * That split is the point. A console whose component list was baked at build
 * time would go stale the moment a component shipped, and a stale copy that
 * still looks authoritative is the exact defect class this ecosystem keeps
 * removing. Equally, the nav and headings have no reason to cost a WASM
 * download — so they do not.
 */
export default defineConfig({
  output: "static",
  site: "https://app.mzizi.dev",
  build: {
    // The Worker serves `dist/` as Static Assets; `format: "file"` emits
    // `/tokens.html` rather than `/tokens/index.html`, which the asset server
    // resolves from `/tokens` without a redirect hop.
    format: "file",
  },
  devToolbar: { enabled: false },
});
