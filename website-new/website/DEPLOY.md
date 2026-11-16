# Deploy

1. Put your real screenshots in `shots/` (1455x978 PNG): play, accounts, versions, servers, mods, skins, worlds, settings, discord.
2. Create `favicon.ico` from `favicon.svg` (the site links the SVG; add the .ico for older browsers).
3. Publish: `npx gh-pages -d website`
4. In the repo: Settings > Pages > Source = `gh-pages` branch.
5. If your Pages URL differs from `https://monk81694-crypto.github.io/cinder`, update `sitemap.xml`, `robots.txt` and the canonical/OG URLs in each page.

The download page reads `https://api.github.com/repos/monk81694-crypto/cinder/releases/latest`. Attach `.exe`, `.dmg` and `.AppImage` files to a release and put the SHA-256 hash (64 hex chars) in the release notes.
