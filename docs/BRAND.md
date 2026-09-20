# Cinder Design System — extracted from code

> Source of truth: styles.css (795 lines, :root lines 4-51, accent themes lines 691-716) and index.html (612 lines, body data-accent=indigo line 10). Every token below is copied verbatim from styles.css. Do NOT edit styles.css, index.html, or any existing file to change the brand.

## 1. Color tokens

Base (:root, styles.css 4-51). Invariant across themes unless overridden in section 2.

| Token | Value | Used as |
|---|---|---|
| --bg0 | #06060f | body background, .bg gradient end |
| --bg1 | #0a0a18 | .bg gradient start |
| --bg2 | #101024 | reserved bg step (defined) |
| --rail | #12122b | .sidebar background, .modal/.palette background #12122b literal |
| --glass | rgba(16, 16, 38, 0.7) | glass tint (defined) |
| --card | linear-gradient(180deg, rgba(255,255,255,0.045), rgba(255,255,255,0.01)) | .card, .server-card layered over --card-solid |
| --card-solid | #101024 | .card, .server-card, .news-card base |
| --card-2 | #181836 | secondary card step (defined) |
| --inset | rgba(0, 0, 0, 0.5) | inset fill (defined), .input background rgba(0,0,0,0.5) family |
| --stroke | rgba(140, 140, 255, 0.16) | .input/.select/.btn-secondary/.chip borders |
| --stroke-soft | rgba(140, 140, 255, 0.09) | .card/.server-card/.list-panel/.logs-panel borders |
| --stroke-hi | rgba(140, 140, 255, 0.32) | hover/focus borders, .modal/.palette borders |
| --text | #e8e8f5 | body text, input text |
| --dim | #8f8fb0 | .tab-sub, .card-hint, .muted, .server-line |
| --faint | #55556f | label, .tb-ver, .stat-l, .logs-panel .line-dim #55556f literal |
| --violet | #a78bff | invariant violet (defined, never overridden) |
| --gold | #ffd479 | .nav-dot, .badge.compat-warn, .logs-panel .line-warn |
| --red | #ff8f8f | .toast-err bar, .ping-off text, compat-bad rgba(255,110,110) family |
| --green | #7dff9a | .top-presence, .installed-dot, .badge.compat-ok, .ping-on |
| --mint-ink | see section 2 | text on accent fills |

Notes:
- ::selection is rgba(106,106,245,0.45) + #fff (styles.css 63).
- Scrollbar thumbs rgba(140,140,255,0.2), hover rgba(106,106,245,0.5) (styles.css 685-689).
- .bg .grid lines rgba(140,140,255,0.04) 44px; vignette rgba(0,0,0,0.45) (styles.css 80-99).

## 2. Accent themes (--mint family + --cyan + --glow)

Default indigo = :root values (no override block; index.html line 10 body data-accent=indigo; #set-accent option indigo — NoRisk blue default). Overrides are body[data-accent] (styles.css 692-716).

| Theme | --mint | --mint-2 | --mint-deep | --cyan | --mint-ink | --glow | --glow-soft |
|---|---|---|---|---|---|---|---|
| indigo (default, :root) | #6a6af5 | #9d9dff | #4343d8 | #5cc8ff | #e8e8ff | rgba(106, 106, 245, 0.55) | rgba(106, 106, 245, 0.16) |
| mint | #3ef2b6 | #7dffd2 | #12bd85 | #4cc9ff | #03241a | rgba(62, 242, 182, 0.5) | rgba(62, 242, 182, 0.16) |
| ember | #ffa14d | #ffd479 | #e06a1f | #ff7a59 | #ffd9ad | rgba(255, 161, 77, 0.5) | rgba(255, 161, 77, 0.16) |
| violet | #b49bff | #d3c4ff | #7c5cf0 | #6fd3ff | #e2d8ff | rgba(180, 155, 255, 0.5) | rgba(180, 155, 255, 0.16) |
| glacier | #5cd6ff | #b8ecff | #1f9fe0 | #7dffd2 | #c8efff | rgba(92, 214, 255, 0.5) | rgba(92, 214, 255, 0.16) |
| rose | #ff7ab8 | #ffb3d6 | #e03c83 | #ff9a6b | #ffd3e6 | rgba(255, 122, 184, 0.5) | rgba(255, 122, 184, 0.16) |

Only --mint, --mint-2, --mint-deep, --cyan, --mint-ink, --glow, --glow-soft change per theme. All of --bg0/--bg1/--bg2, --rail, --card*, --stroke*, --text/--dim/--faint, --violet, --gold, --red, --green stay invariant. Theme-following surfaces: .nav.active, .btn-primary, .btn-launch/.launch-arrow, .progress-fill (linear-gradient 90deg var(--mint-deep), var(--mint) 60 percent, var(--cyan)), range track/thumb, .toggle checked, .eyebrow, .chip-on/.srv-play/.badge-active, focus outline 2px solid var(--mint).

## 3. Spacing scale

No --space tokens exist. Spacing is literal px in styles.css. Reuse these steps, do not invent new ones:

| Value | Where |
|---|---|
| 4px | .eyebrow vertical pad, .ping vertical pad |
| 6px | .tb-nav gap, --r-md radius, .list-panel child gap |
| 7px | button/input/avatar/ram-col radius, .term-bar dot gap |
| 8px | --r-lg, gaps (.toolbar, .nav-list, .srv-actions, .item-actions), .list-panel pad, .skin-canvas-wrap radius |
| 10px | --r-xl, pads (.stat, .ram-col, .palette-item, .logs-panel .line horiz), gaps (.chips, .toolbar, .field-row, .modal-actions, .stats-strip) |
| 12px | gaps (titlebar, stage-controls, servers-grid, updates-row, list-panel items), pads (.input 12px 14px, .btn-primary 12px 20px, .item 12px 14px, .srv-body 12px 14px) |
| 14px | .sidebar top pad, .settings-footer gap |
| 16px | gaps (.logs-header, .mini-card pad 16px), .toolbar bottom 16px, .card-hint bottom 16px |
| 18px | gaps (.card-grid, .settings-grid, .skins-layout 18px), .content mobile pad, .news-rail pad 18px, .brand-logo glow 0 0 18px |
| 20px | .tab-header bottom 20px, .btn-primary horiz 20px, .settings-grid col gap 20px |
| 22px-26px-30px | .content pad 22px 26px 26px, .stage pad 26px 30px 24px, .modal pad 26px |
| Shell | .titlebar 56px tall, .credit-bar 26px tall, .app calc(100vh - 56px - 26px) min-width 720px, .sidebar 112px (76px at max-width 760px), .tab max-width 1220px |

## 4. Radius scale

| Token | Value | Use |
|---|---|---|
| --r-xl | 10px | .play-layout, .modal, .palette, .brand-logo 10px |
| --r-lg | 8px | .nav, .card, .server-card, .term-bar top, .toast, .skin-canvas-wrap 8px literal |
| --r-md | 6px | .stat, .srv-actions button, .toast-ico, .palette-ico |
| --r-sm | 5px | kbd, .eyebrow, .badge, .srv-tag, .toggle .track 5px, .top-avatar 5px |
| ad-hoc 7px | 7px | .tb-icon-btn, .top-account, .instance-pill, .input/.select, .btn-primary/.btn-secondary, .list-panel .item, .avatar, .ram-col, .palette-item |
| ad-hoc 99px | 99px | .chip, .progress-track/.progress-fill, .ping |
| ad-hoc 50 percent | 50 percent | .top-presence, .nav-dot, .installed-dot, range thumbs, .term-bar i |

## 5. Shadow scale

| Token | Value | Use |
|---|---|---|
| --sh-lg | 0 24px 60px rgba(0, 0, 0, 0.6) | .modal, .palette (palette adds 0 0 60px rgba(106,106,245,0.2)) |
| --sh-md | 0 12px 36px rgba(0, 0, 0, 0.5) | .card, .news-card hover, .server-card hover, .toast |
| --sh-sm | 0 4px 16px rgba(0, 0, 0, 0.4) | small lift (defined) |
| glow family | 0 0 18px var(--glow) (.brand-logo), 0 0 18px rgba(106,106,245,0.35) (.nav.active), 0 0 24px rgba(106,106,245,0.45) (.btn-launch), 0 0 12px var(--glow) (.progress-fill), 0 0 10px var(--glow) (range thumb, toggle knob) | accent glow — follows section 2 |

## 6. Type scale

Family (styles.css 45-46): --font = --mono = Cascadia Code, JetBrains Mono, SFMono-Regular, Consolas, Menlo, monospace. Base body 14px. Weights used: 700 and 800 only.

| Size | Weight | Where |
|---|---|---|
| 30px | 800 | .tab-header h1 |
| 22px | 800 | .nametag |
| 21px | 800 | .launch-main |
| 20px | 800 | .news-title, .worlds-row emoji 20px |
| 17px | 800 | .tb-title, .modal h3 (tb-icon-btn 15px / svg 17px family) |
| 16px | 800 | .card h2, .stat-n |
| 14px | 400-800 | body base, .news-head, .srv-body b, .palette-input, .ram-col b |
| 13px-13.5px | 400-800 | .top-name 13px/800, .tab-sub/.muted/.card-hint 13px, .input 13.5px, .list-panel .item 13.5px, .btn 13px/800, .palette-item 13px, .server-line 13px |
| 12px-12.5px | 700-800 | .instance-pill 12.5px/700, .status-text 12.5px, .mono 12.5px, .logs-panel 12px/mono/1.7, .chip 12px/700, .srv-desc 12px, .badge 11px/800 family, .toast 12.5px |
| 11px-11.5px | 400-800 | .hint-inline 11px, .progress-text/.news-sub/.news-list/.item-sub/.term-bar 11-11.5px, .ping 11px/700/mono, .palette-item small 11px |
| 10px-10.5px | 700-800 | label 10px/800/1.5px tracking, kbd 10px/700, .tb-ver 10px/700, .top-sub 10px/700, .launch-sub 10px/700, .ram-cap 10px/800, .nav 10.5px/700, .eyebrow 10.5px/800/2px tracking, .credit-bar 10.5px |
| 9px-9.5px | 700-800 | .stat-l 9px/700/1.2px uppercase, .srv-tag 9.5px/800/1.2px uppercase, .nav 9px at max-width 760px |
| mono contexts | 400-800 | .mono, .srv-ip 11.5px, .ping 11px, .term-bar 11px, .logs-panel 12px/1.7, play-server/skin inputs |

Line-height: 1.3 (.top-account-text), 1.5-1.7 (.card-hint 1.6, .muted 1.65, .logs-panel 1.7, .srv-desc 1.5, .toast 1.55), 1 (.tb-ver, .ping).

## 7. Motion rules

Easings: --ease cubic-bezier(0.22, 1, 0.36, 1); --spring cubic-bezier(0.34, 1.4, 0.4, 1); plus ease-in-out (float, pulse, breathe) and linear (snowfall, shimmer).

| Duration | What |
|---|---|
| var(--t) 160ms | base for all on .nav, .btn, .chip, .input/.select, .server-card, .list-panel .item, .toggle .track |
| 200ms | .modal-backdrop fade-in |
| 240ms | .palette modal-pop, .toast.out |
| 250ms | .tab-in, .row-in list rows |
| 260ms | .modal-pop |
| 280ms | .toast-in, .progress-fill width |
| 320ms | .tab.active child row-in stagger (+60/120/180/240ms delays) |
| 350ms | .news-card row-in stagger (+70/140ms) |
| 400ms | .news-banner img / .srv-img scale |
| 600ms | .skin-canvas canvas-in |
| 1.3s linear | .skeleton shimmer |
| 1.6s / 2.4s ease-in-out | .nav-dot pulse 1.6s, .top-presence pulse 2.4s |
| 2.8s | .bolt bolt-flicker |
| 3.2s | .btn-launch launch-idle (disabled stops) |
| 5s ease-in-out | .skin-body float-y |
| 7s ease-in-out | .stage stage-breathe |
| 60s linear / 90s linear reverse | .stars.s1 / .s2 snowfall |

Reduce-motion (mirror on web):
- body.reduce-motion all elements: animation none !important, transition none !important (styles.css 717-719), toggled by #set-motion (Reduce motion, no floating/glowing animations).
- prefers-reduced-motion reduce media: animation-duration 0.01ms !important, transition-duration 0.01ms !important (styles.css 751-753).
- body[data-snow=off] .stars display none (styles.css 786), toggled by #set-showsnow.
- .stage.still .skin-body animation none + #skin-anim checkbox (styles.css 266).
- Focus stays visible: button/input/select/range focus-visible outline 2px solid var(--mint), offset 2px (styles.css 722-725).

## 8. Logo usage rules

Assets: assets/logo.svg (titlebar .brand-logo, favicon), assets/mark.svg (footer .credit-bar), assets/icon.ico (Windows), assets/icon.png (Linux), assets/logo-512.png (Discord Art Asset). Observed sizes: .brand-logo 44px CSS (46x46 attrs index.html line 50), .credit-bar img 13px (styles.css 169).

- Clear space = height of the ember square (orange rounded square behind the flame) on all four sides. Nothing enters it.
- Min size 16px (favicon/presence floor). Below that use mark.svg, never the full lockup.
- Never recolor (keep ember square as shipped, do not tint to theme --mint), never stretch (preserve aspect; .brand-logo is square 44px), never add effects (no extra drop-shadow or stroke beyond 0 0 18px var(--glow) on .brand-logo).
- Favicon stays assets/logo.svg (index.html line 7); sidebar stays assets/logo.svg (index.html line 50); footer stays assets/mark.svg (index.html line 578) until the website-phase swap lands.

## 9. Per-tab checklist (tab id to tokens to keep)

index.html sections: tab-play, tab-servers, tab-versions, tab-accounts, tab-mods, tab-skins, tab-worlds, tab-settings, tab-logs. Shell shared by all: --bg0/--bg1, --rail sidebar, --stroke-soft dividers, --text/--dim/--faint, --r-lg/--r-md, --sh-md, --t/--ease, --font/--mono, focus var(--mint).

| Tab (index.html id) | Must use (spot-check against styles.css) |
|---|---|
| play - #tab-play | --bg1 to --bg0 stage gradient + rgba(106,106,245,0.08) radial; --card-solid news cards; --stroke-soft/--stroke/--stroke-hi; --text/--dim/--faint; accent --mint/--mint-2/--mint-deep/--cyan/--glow/--glow-soft (launch, progress mint-deep to mint to cyan, eyebrow, stats mint-2); --r-xl/--r-md/--r-lg; --sh-md; --t/--ease/--spring; --font/--mono; kbd/chip/badge |
| servers - #tab-servers | --card + --card-solid cards; --stroke-soft/--stroke/--stroke-hi; --text/--dim + --mint-2 IP; accent mint (.srv-play, .chip-on); --green/--red/--dim .ping-on/off/wait; .srv banner gradients stay literal; --r-lg/--r-md/5px tag; --sh-md hover; --t/--ease; --mono IPs |
| versions - #tab-versions | rgba(0,0,0,0.35-0.4) .list-panel; --stroke-soft/--stroke/--mint focus; --text/--dim/--faint; accent rgba(106,106,245,0.1/0.6) active row; --green/--gold/--red compat badges; --r-lg/7px/6px; --t/--ease; .skeleton shimmer; .installed-dot var(--green) |
| accounts - #tab-accounts | --card + --card-solid two-col grid; --stroke/--stroke-soft; --text/--dim/--faint; accent focus/glow; .chip 99px suggestions; .avatar 40px pixelated; --r-lg/7px; --sh-md; --t/--ease; .status-text 12.5px |
| mods - #tab-mods | --card + --card-solid; --stroke/--stroke-hi; --text/--dim/--faint; accent loader/progress/toggle (--mint/--mint-2/--glow); --green/--red/--gold badges; .mod-icon 48px; .toggle 42x22 + --spring knob; .ver-picker; --r-lg/8px/7px; --sh-md; --t/--ease |
| skins - #tab-skins | .skins-layout 380px+1fr; --card-solid; --stroke-soft canvas frame; --glow-soft radial + drop-shadow(var(--glow)) canvas; --text/--dim/--faint; accent buttons/focus; .input.mono username; --r-lg/8px/7px; --sh-md; --t/--ease |
| worlds - #tab-worlds | .list-panel + .worlds-row (20px emoji, anywhere wrap); --stroke-soft/--stroke-hi; --text/--faint; accent hover only; --green backup hover; --r-lg/6px/7px; --t/--ease |
| settings - #tab-settings | .settings-grid 1fr+1fr/18px 20px; --card + --card-solid; --stroke/--stroke-hi focus + var(--glow-soft) ring; --text/--dim/--faint; accent --mint/--mint-2/--mint-deep/--cyan/--glow (range mint-deep to mint to cyan, toggle, chips); --r-lg/7px/99px; --sh-md; --t/--ease/--spring; --mono paths/args; #set-accent 6 options; #set-motion/#set-showsnow switches |
| logs - #tab-logs | rgba(2,2,8,0.9) panel + rgba(0,0,0,0.5) term bar; --stroke-soft; base #b9b9d9 + --faint/#55556f; --gold/--red/--green .line-warn/error/ok; --cyan default toast bar; --mono 12px/1.7 pre-wrap; 8px radii; hover rgba(140,140,255,0.05) |

> Website phase: import ../brand/tokens.css (single source of truth) and verify each tab row above still passes with every data-accent value.

