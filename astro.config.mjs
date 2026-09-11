// @ts-check
import { existsSync, readFileSync } from "node:fs";
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
 *
 * `output: "static"` is also what chose the AuthKit flavour. A static site has
 * no server to receive an OAuth callback or set a session cookie, and this
 * Worker has no `main` at all — it is Static Assets and nothing else. So auth
 * runs in the browser through `@workos-inc/authkit-js`; see `src/lib/authkit.ts`
 * for the trade-off that choice makes and the one it refuses to make.
 */

// Loud rather than silent. A build without a client ID still succeeds — CI has
// no WorkOS configuration and should not need any — but it ships a console
// whose gate refuses everyone, and that deserves a line in the build log
// rather than only a sentence on the page.
//
// Both sources are checked by hand because Vite has not read `.env` at config
// time and `vite` is not a direct dependency here. A build variable covers
// CI and Workers Builds; the file covers local development.
const ENV_FILE = new URL(".env", import.meta.url);
const configured =
  Boolean(process.env.PUBLIC_WORKOS_CLIENT_ID) ||
  (existsSync(ENV_FILE) &&
    /^\s*PUBLIC_WORKOS_CLIENT_ID\s*=\s*\S/m.test(readFileSync(ENV_FILE, "utf8")));
if (!configured) {
  console.warn(
    "[mzizi-console] PUBLIC_WORKOS_CLIENT_ID is not set. The AuthKit gate will refuse every visitor and sign-in will be unavailable. See .env.example.",
  );
}

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
