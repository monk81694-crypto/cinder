# Deploying the Cinder website

The site in `website/` is plain static HTML/CSS/JS — no build step.

## Local preview

```powershell
cd website
python -m http.server 8080
# open http://localhost:8080
python check.py --root .
```

`check.py` verifies: all 6 pages exist, one `<h1>` each, titles,
meta descriptions, image `alt` text, internal links resolve, the Mojang
legal line is present everywhere, and key color pairs pass WCAG contrast.

## GitHub Pages

1. Push the repo to GitHub.
2. Settings → Pages → Deploy from branch → `main`, folder `/website`.
3. The site goes live at `https://<user>.github.io/<repo>/`.
4. Then update the placeholder domain in `sitemap.xml` and `robots.txt`
   (`https://radoslavgeme.github.io/cinder/`) to the real URL.

## Netlify

- Drag the `website/` folder into Netlify Drop, or
  `netlify deploy --dir website --prod` with the Netlify CLI.
- No build command, no environment variables.

## Release checklist

- [ ] Version badge matches `package.json` (`index.html`, download page).
- [ ] `changelog.html` matches root `CHANGELOG.md`.
- [ ] `python check.py` exits 0.
- [ ] `favicon.ico` + `og.svg` load (replace `og.svg` with a raster
      `og.png` and update meta tags when real screenshots exist).
- [ ] Mockups still labeled as mockups until replaced by real screenshots.
