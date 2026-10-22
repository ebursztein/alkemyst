# Static site delivery

One Astro/Starlight build serves `/`, `/download/` and `/docs/` through one
[Cloudflare Workers Static Assets](https://developers.cloudflare.com/workers/static-assets/)
deployment. It hosts public marketing and documentation only. Arena history,
decks, application databases, credentials and MCP are never deployed.

The configured initial target is `https://alkemyst.3713.workers.dev`. Account access
and the Workers subdomain have been verified. `alkemyst.cards` was not present in
the connected Cloudflare account at setup; no domain purchase or DNS change is
included. Bind it only after ownership/access is established, then update the
canonical origin and deployment environment URL together.

## Build and verify

Use the pinned Node version, pnpm and frozen lockfile. Wrangler is an exact pinned
workspace development dependency, including the workerd engine used by previews.

```sh
SITE_URL=https://alkemyst.3713.workers.dev pnpm site:build
SITE_URL=https://alkemyst.3713.workers.dev pnpm test:site:worker
```

The deployment build requires a clean committed candidate and an HTTPS root
origin. It writes `deployment.json` into the built output with the source commit,
Git tree and origin. Incomplete output or invalid provenance fails the build.
Worker preview tests exercise links, Pagefind search, light/dark accessibility,
download status, response headers, real 404s and the source manifest. Missing paths
never become a successful marketing page; `/mcp` and `/v1/decks` return 404.

For an authorized local deployment using an existing Wrangler login:

```sh
SITE_URL=https://alkemyst.3713.workers.dev pnpm site:deploy
SITE_URL=https://alkemyst.3713.workers.dev \
  SITE_TEST_URL=https://alkemyst.3713.workers.dev pnpm test:site
```

Set `ALKEMYST_EXPECTED_SITE_COMMIT` to the deployed source commit when verifying a
local deployment. CI compares the public manifest with `GITHUB_SHA` automatically.

## GitHub delivery

The Site delivery workflow validates pull requests without deployment credentials.
On `main`, or manual dispatch from `main`, it deploys the exact static artifact
validated in the preceding job, then runs the browser checks against the live site.
Failed validation prevents deployment. A failed or skipped deployment is not a
successful publication. Reports and static artifacts are retained for 14 days.

Automatic deployment requires:

- Repository variable `CLOUDFLARE_ACCOUNT_ID` for the intended account.
- Secret `CLOUDFLARE_API_TOKEN`, scoped to that account with Workers Scripts edit
  permission. Follow Cloudflare's
  [CI authentication instructions](https://developers.cloudflare.com/workers/ci-cd/external-cicd/github-actions/).

The developer OAuth login is not copied into GitHub. Missing CI credentials fail
explicitly. Add tokens through GitHub's secret settings, never source files or chat.
Desktop installers remain on GitHub Releases; this deployment does not upload them.
