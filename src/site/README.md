# Public site

One static Astro + Starlight project serves the custom marketing homepage,
`/download/` and `/docs/`. The docs collection keeps its entries under `docs/` so
Starlight routes coexist with custom Astro pages without a second deployment.

The site imports the desktop package's public `palette.css` export. Marketing and
documentation layouts stay separate. The screenshot uses the public Boros fixture;
no private Arena data, player identifiers or MCP credentials are included.

Use the Node version in `.node-version`, then install with the frozen pnpm lockfile.
`pnpm --filter @alkemyst/site build` runs strict Astro checks and builds the pages
and Pagefind index. `pnpm test:site` serves that actual static output and verifies
search, local destinations, preview availability, keyboard focus, narrow layouts,
system themes and WCAG Axe checks. The preview script uses Astro's foreground mode
so Playwright owns and stops its server, including in agent environments.

Set `SITE_URL` to the verified deployment origin when building for publication.
Without it the local build omits the sitemap; it does not invent a public origin.
Cloudflare Workers Static Assets deployment is tracked separately. No domain,
account, storage service or hosted MCP endpoint is provisioned by this package.

Installers are not published. The download page states that explicitly and points
to GitHub Releases. Update availability only after actual platform artifacts and
checksums exist and have passed their release checks. Keep in-development features
clearly identified until their implementation items close.

Original site code is Apache-2.0. Starlight's MIT license is preserved in
`public/licenses/starlight.txt`; see the root NOTICE for attribution.
