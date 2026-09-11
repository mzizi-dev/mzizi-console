/**
 * AuthKit, in the browser.
 *
 * The console is `output: "static"` on purpose (see `astro.config.mjs`): there
 * is no server here to receive an OAuth callback, and the Worker that serves
 * it has no `main` at all — it is Static Assets and nothing else. So this uses
 * `@workos-inc/authkit-js`, the browser SDK, which runs the authorization-code
 * flow with PKCE entirely client-side and exchanges the code from the callback
 * page itself.
 *
 * The alternative — `@workos/authkit-astro` — requires `output: "server"`, an
 * adapter, a `WORKOS_API_KEY`, and React as a peer dependency. Each of those
 * three is a change this repo has explicitly decided against.
 *
 * NOTHING IN THIS FILE IS A SECRET. A WorkOS client ID is public by
 * construction: it travels in the authorization URL of every sign-in. The
 * browser SDK never sees an API key or a client secret, which is the whole
 * reason it is safe to ship in a static bundle. If a change to this file ever
 * needs one, the change is wrong — move the flow to a server instead.
 */
import { createClient, type Client, type User } from "@workos-inc/authkit-js";

/** The AuthKit client ID for the environment this bundle was built against. */
export const CLIENT_ID: string = import.meta.env.PUBLIC_WORKOS_CLIENT_ID ?? "";

/**
 * The Authentication API hostname, when the environment has a custom domain.
 *
 * WorkOS: "Continuing to make requests to `api.workos.com` after a custom
 * domain is configured for the Authentication API can result in issues with
 * your integration." So this is not cosmetic — an environment with a custom
 * auth domain MUST set it, and one without MUST NOT.
 */
export const API_HOSTNAME: string =
  import.meta.env.PUBLIC_WORKOS_API_HOSTNAME ?? "";

/** The path of the page that completes the OAuth redirect. */
export const CALLBACK_PATH: string =
  import.meta.env.PUBLIC_WORKOS_CALLBACK_PATH ?? "/callback";

/**
 * Where the refresh token lives.
 *
 * `devMode: false` keeps the refresh token OUT of `localStorage` — the SDK
 * holds it in memory and relies on an HttpOnly cookie set by the
 * Authentication API. `devMode: true` falls back to `localStorage`, which is
 * readable by any script on the page. The SDK's own default is
 * `location.hostname === "localhost" || "127.0.0.1"`; leaving this unset keeps
 * that default, which is the correct behaviour for both `astro dev` and
 * production.
 */
export const DEV_MODE: string = import.meta.env.PUBLIC_WORKOS_DEV_MODE ?? "";

/** Whether this build carries enough configuration to attempt a sign-in. */
export function isConfigured(): boolean {
  return CLIENT_ID.length > 0;
}

/** The absolute redirect URI, which must match one registered in WorkOS. */
export function redirectUri(): string {
  return new URL(CALLBACK_PATH, window.location.origin).href;
}

/**
 * Resolve a `returnTo` carried through the OAuth `state` parameter.
 *
 * `state` round-trips as plaintext in the URL and WorkOS does not integrity
 * protect it, so it is untrusted input. Anything that is not same-origin is
 * discarded rather than followed — otherwise a crafted sign-in link is an
 * open redirect, and a `javascript:` URI is worse than that.
 */
export function safeReturnTo(value: unknown, fallback = "/"): string {
  if (typeof value !== "string") return fallback;
  let url: URL;
  try {
    url = new URL(value, window.location.origin);
  } catch {
    return fallback;
  }
  return url.origin === window.location.origin ? url.href : fallback;
}

let pending: Promise<Client> | null = null;

/**
 * The one AuthKit client for this page.
 *
 * `createClient` is async and its promise is the initialization — on the
 * callback page it is what exchanges the code, and everywhere else it is what
 * restores an existing session. Awaiting a second `createClient` would run
 * that twice, so the promise is memoised.
 */
export function authKit(): Promise<Client> {
  if (!isConfigured()) {
    return Promise.reject(
      new Error(
        "PUBLIC_WORKOS_CLIENT_ID is not set, so this build has no AuthKit configuration.",
      ),
    );
  }
  pending ??= createClient(CLIENT_ID, {
    redirectUri: redirectUri(),
    ...(API_HOSTNAME ? { apiHostname: API_HOSTNAME } : {}),
    ...(DEV_MODE ? { devMode: DEV_MODE === "true" } : {}),
  });
  return pending;
}

export type { Client, User };
