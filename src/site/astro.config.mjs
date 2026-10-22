import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
export default defineConfig({
  output: "static",
  site: process.env.SITE_URL,
  trailingSlash: "always",
  integrations: [
    starlight({
      title: "Alkemyst",
      description:
        "A local Arena deck laboratory. Tracking, seeded simulations, and your own agents.",
      social: [{ icon: "github", label: "GitHub", href: "https://github.com/ebursztein/alkemyst" }],
      customCss: ["./src/styles/docs.css"],
      sidebar: [
        {
          label: "Start here",
          items: [
            { label: "Overview", slug: "docs" },
            { label: "Setup", slug: "docs/setup" },
          ],
        },
        { label: "Your local lab", items: [{ autogenerate: { directory: "docs/guides" } }] },
      ],
    }),
  ],
});
