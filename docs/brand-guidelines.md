# Cinder Brand Guidelines v2.0

## Quick Reference
- **Primary Color:** #FF6B1A (Ember)
- **Secondary Color:** #3EF2B6 (Mint)
- **Background:** #06060F (Deep Charcoal)
- **Primary Font:** "Space Grotesk", system-ui, sans-serif
- **Monospace Font:** "JetBrains Mono", "Cascadia Code", monospace
- **Voice:** Bold, Technical, Player-First

---

## 1. Color Palette

### Primary Colors
| Name | Hex | RGB | Usage |
|------|-----|-----|-------|
| Ember | #FF6B1A | rgb(255,107,26) | Primary CTAs, active states, brand highlights |
| Ember Deep | #C24E0E | rgb(194,78,14) | Hover states, pressed buttons |
| Ember Soft | rgba(255,107,26,0.14) | - | Focus rings, subtle backgrounds |
| Mint | #3EF2B6 | rgb(62,242,182) | Success states, online indicators, secondary accent |
| Mint Deep | #12BD85 | rgb(18,189,133) | Hover for mint elements |

### Accent Themes (Launcher Themes)
| Theme | Primary | Secondary | Use Case |
|-------|---------|-----------|----------|
| Ember (Default) | #FF6B1A | #FFD479 | Default Cinder branding |
| Mint | #3EF2B6 | #7DFFD2 | Fresh, clean alternative |
| Violet | #B49BFF | #D3C4FF | Creative, modding-focused |
| Glacier | #5CD6FF | #B8ECFF | Cool, technical feel |
| Rose | #FF7AB8 | #FFB3D6 | Warm, community-focused |

### Neutral Palette
| Name | Hex | RGB | Usage |
|------|-----|-----|-------|
| Deep Charcoal | #06060F | rgb(6,6,15) | Page/app background |
| Charcoal | #0A0A18 | rgb(10,10,24) | Cards, raised surfaces |
| Dark Charcoal | #101024 | rgb(16,16,36) | Inputs, sunken areas |
| Rail | #12122B | rgb(18,18,43) | Sidebar, navigation |
| Border Subtle | rgba(140,140,255,0.09) | - | Hairline borders |
| Border | rgba(140,140,255,0.16) | - | Standard borders |
| Border Strong | rgba(140,140,255,0.32) | - | Focus, active borders |

### Text Colors
| Name | Hex | Usage |
|------|-----|-------|
| Text Primary | #E8E8F5 | Body text, headings |
| Text Dim | #A9A9C6 | Secondary text, descriptions |
| Text Faint | #7C7C9C | Captions, placeholders |
| On Ember | #1A0D02 | Text on ember backgrounds |

### Semantic Colors
| Name | Hex | Usage |
|------|-----|-------|
| Success | #7DFF9A | Online, installed, complete |
| Warning | #FFD479 | Pending, caution |
| Error | #FF8F8F | Offline, failed, destructive |

### Accessibility
- Text/Background Contrast: 14.2:1 (WCAG AAA)
- CTA Contrast (Ember on Deep Charcoal): 8.7:1 (WCAG AAA)
- All interactive elements meet WCAG 2.1 AA minimum

---

## 2. Typography

### Font Stack
```css
--font-display: 'Space Grotesk', system-ui, -apple-system, sans-serif;
--font-body: 'Space Grotesk', system-ui, -apple-system, sans-serif;
--font-mono: 'JetBrains Mono', 'Cascadia Code', 'SFMono-Regular', Consolas, monospace;
```

### Type Scale
| Element | Font | Weight | Size (Desktop/Mobile) | Line Height | Letter Spacing |
|---------|------|--------|----------------------|-------------|----------------|
| Display | Display | 700 | 56px / 40px | 1.1 | -0.02em |
| H1 | Display | 700 | 44px / 32px | 1.15 | -0.01em |
| H2 | Display | 600 | 32px / 26px | 1.2 | -0.01em |
| H3 | Display | 600 | 24px / 20px | 1.25 | 0 |
| H4 | Body | 600 | 20px / 18px | 1.3 | 0 |
| Body Large | Body | 400 | 18px / 16px | 1.6 | 0 |
| Body | Body | 400 | 16px / 15px | 1.65 | 0 |
| Small | Body | 400 | 14px / 14px | 1.5 | 0 |
| Caption | Body | 400 | 12px / 12px | 1.4 | 0.02em |
| Button | Body | 700 | 14px / 13px | 1 | 0.02em |
| Label | Body | 700 | 11px / 11px | 1 | 0.1em |

### Code/Monospace Scale
| Element | Size | Line Height |
|---------|------|-------------|
| Code Inline | 0.875em | 1.5 |
| Code Block | 13px | 1.6 |
| Terminal | 12px | 1.7 |

---

## 3. Logo Usage

### Variants
- **Primary:** Full horizontal logo with wordmark (logo.svg)
- **Stacked:** Vertical arrangement for square spaces (logo-dark.svg)
- **Icon:** Symbol only for favicons, app icons (icon-64.png)
- **Monochrome:** Single color for limited palettes (logo-mono-white.svg)

### Clear Space
Minimum clear space = height of the "C" in Cinder wordmark

### Minimum Size
- Digital: 120px width minimum (wordmark)
- App Icon: 64×64px minimum
- Favicon: 32×32px

### Don'ts
- Don't rotate or skew
- Don't change colors outside approved palette
- Don't add effects (shadows, gradients) unless specified
- Don't crop or modify proportions
- Don't place on busy backgrounds without safe zone

---

## 4. Voice & Tone

### Brand Personality
**Bold:** We speak directly about what Cinder does. No hedging.
**Technical:** We respect our users' intelligence. Accurate terminology, no dumbing down.
**Player-First:** Every decision serves the player experience. Not the launcher, not the ecosystem.

### Voice Chart
| Trait | We Are | We Are Not |
|-------|--------|------------|
| Bold | Direct, confident, clear | Arrogant, aggressive, pushy |
| Technical | Precise, specific, honest | Jargon-heavy, gatekeeping, obscure |
| Player-First | Empathetic, practical, generous | Pandering, apologetic, soft |

### Tone by Context
| Context | Tone | Example |
|---------|------|---------|
| Marketing | Bold + Player-First | "Launch Minecraft your way — offline or premium." |
| Documentation | Technical + Player-First | "Add an account. Pick a version. Hit Launch. That's it." |
| Error Messages | Technical + Direct | "Invalid session. Premium servers require Microsoft login." |
| Success | Warm + Direct | "Fabric 0.16.10 installed for 1.21.1. Mods will load on next launch." |
| Settings | Technical + Organized | "JVM preset: Balanced. Auto-picks Java 21 for 1.21+." |

### Prohibited Terms
- "Best launcher ever" (unverifiable)
- "No setup required" (misleading — Java/Microsoft login needed)
- "Works on all servers" (false — offline accounts can't join premium)

---

## 5. Visual Effects & Motion

### Shadows
| Token | Value | Usage |
|-------|-------|-------|
| --shadow-xs | 0 1px 2px rgba(0,0,0,0.3) | Subtle separators |
| --shadow-sm | 0 4px 16px rgba(0,0,0,0.4) | Cards, dropdowns |
| --shadow-md | 0 12px 36px rgba(0,0,0,0.5) | Modals, elevated cards |
| --shadow-lg | 0 24px 60px rgba(0,0,0,0.6) | Major overlays |
| --shadow-glow | 0 0 24px rgba(255,107,26,0.35) | Primary button hover |
| --shadow-glow-soft | 0 0 16px rgba(255,107,26,0.18) | Primary button idle |

### Border Radius
| Token | Value | Usage |
|-------|-------|-------|
| --radius-xs | 4px | Badges, chips |
| --radius-sm | 6px | Inputs, buttons |
| --radius-md | 8px | Cards, panels |
| --radius-lg | 12px | Modals, major sections |
| --radius-xl | 16px | Hero containers |
| --radius-full | 9999px | Pills, avatars |

### Transitions
| Token | Value | Usage |
|-------|-------|-------|
| --transition-fast | 100ms cubic-bezier(0.22, 1, 0.36, 1) | Hover states |
| --transition-base | 160ms cubic-bezier(0.22, 1, 0.36, 1) | Standard interactions |
| --transition-slow | 240ms cubic-bezier(0.22, 1, 0.36, 1) | Panel animations |
| --transition-spring | 300ms cubic-bezier(0.34, 1.4, 0.4, 1) | Playful elements |

### Motion Principles
- **Reduced motion** respected via `prefers-reduced-motion` and launcher setting
- **Staggered entrance** for lists (60ms delay per item)
- **Meaningful motion** — every animation communicates state change
- **No decorative animation** that doesn't convey information

---

## 6. Component Specifications

### Buttons
| State | Primary | Secondary | Ghost | Destructive |
|-------|---------|-----------|-------|-------------|
| Default | Ember bg, On Ember text | Charcoal bg, Text border | Transparent, Text | Transparent, Error text |
| Hover | Ember Deep, lift -2px | Border Strong, lift -2px | Ember Soft bg | Error Soft bg |
| Active | Ember Deep, scale 0.98 | Border Strong, scale 0.98 | Ember Soft bg | Error Soft bg |
| Disabled | 50% opacity, no shadow | 50% opacity | 50% opacity | 50% opacity |
| Focus | 2px Ember outline | 2px Ember outline | 2px Ember outline | 2px Error outline |

### Inputs
| State | Background | Border | Text | Placeholder |
|-------|------------|--------|------|-------------|
| Default | Dark Charcoal | Border | Text Primary | Text Faint |
| Hover | Dark Charcoal | Border Strong | Text Primary | Text Faint |
| Focus | Dark Charcoal | Ember (2px) | Text Primary | Text Dim |
| Error | Dark Charcoal | Error | Text Primary | Text Faint |
| Disabled | Rail | Border Subtle | Text Faint | Text Faint |

### Cards
| Variant | Background | Border | Shadow | Hover |
|---------|------------|--------|--------|-------|
| Default | Charcoal | Border | --shadow-sm | Lift -4px, Border Strong, --shadow-md |
| Elevated | Charcoal | Border | --shadow-md | Lift -6px, --shadow-lg |
| Glass | rgba(16,16,38,0.7) | Border Subtle | --shadow-sm | Border Strong |
| Interactive | Charcoal | Border | --shadow-sm | Cursor pointer, lift -2px |

### Navigation (Sidebar)
| State | Background | Text | Icon |
|-------|------------|------|------|
| Default | Transparent | Text Dim | Text Dim |
| Hover | Ember Soft | Text Primary | Mint |
| Active | Ember Soft + 2px Ember left border | White | Mint + glow |
| Active Hover | Ember Soft + glow | White | Mint + glow |

---

## 7. Layout & Spacing

### Spacing Scale (4px base)
| Token | Value | Usage |
|-------|-------|-------|
| --space-1 | 4px | Micro gaps |
| --space-2 | 8px | Tight gaps |
| --space-3 | 12px | Standard gaps |
| --space-4 | 16px | Medium gaps |
| --space-5 | 20px | Large gaps |
| --space-6 | 24px | Section gaps |
| --space-8 | 32px | Major sections |
| --space-10 | 40px | Page sections |
| --space-12 | 48px | Hero spacing |
| --space-16 | 64px | Large page gaps |

### Container Widths
| Token | Value | Usage |
|-------|-------|-------|
| --container-sm | 640px | Narrow content |
| --container-md | 880px | Standard content |
| --container-lg | 1080px | Wide content (website) |
| --container-xl | 1280px | Full-width sections |
| --container-full | 100% | Edge-to-edge |

### Grid
- 12-column grid for desktop
- 8-column grid for tablet
- 4-column grid for mobile
- Gap: --space-4 (16px) default

---

## 8. Iconography

### Style
- **Primary:** Outlined (2px stroke) — UI interfaces
- **Secondary:** Filled — Active states, emphasis
- **Size Base:** 24×24px (1.5rem)
- **Stroke Width:** 2px (outlined), N/A (filled)
- **Corner Radius:** 2px (consistent with UI)

### Icon Set
Use Lucide icons as base, customize for Cinder-specific:
- Minecraft block → Custom cube icon
- Flame/Ember → Custom ember icon
- Server → Custom server rack
- Mod → Custom puzzle piece

---

## 9. Imagery Guidelines

### Screenshots
- **Aspect Ratio:** 16:9 (launcher), 4:3 (website hero)
- **Background:** Charcoal with subtle grid
- **Window Chrome:** Custom shot bar (not OS native)
- **Annotation:** Ember accent for callouts

### 3D Renders (Skins)
- **Lighting:** Three-point, cool fill + warm key
- **Background:** Radial gradient (ember glow)
- **Animation:** Subtle float + auto-rotate (respect reduced motion)

---

## 10. Implementation Notes

### CSS Variables (Design Tokens)
All values exposed as CSS custom properties in `brand/tokens.css` and synced to:
- `website/styles.css` (via @import)
- `styles.css` (launcher)

### Theme Switching
Launcher supports 6 accent themes via `body[data-accent="..."]` — all use same semantic tokens, only primitive accent values change.

### Website Build
Static HTML/CSS/JS — no build step. Tokens imported via `<link rel="stylesheet" href="brand/tokens.css">`

### Launcher Build
Electron + vanilla JS — tokens used directly in `styles.css` (single source of truth)

---

## 11. Changelog

| Version | Date | Changes |
|---------|------|---------|
| 2.0 | 2026-11-16 | Complete redesign: new brand identity, design system, token architecture |
| 1.0 | 2026-01-15 | Initial brand guidelines (basic colors, logo) |