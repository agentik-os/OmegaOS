# Stax Design Spec — the pixel-level transformation contract

This is the authoritative UI/UX spec of the Stax panel system. Every value below is the
one shipped in `frameword/apps/crm-specimen/src/styles.css` + `tokens.css`. During a
migration, **every visual element of the old app is converted against this contract** —
icons, cards, badges, buttons, tables, spacing, type, color, states. Nothing is restyled
"by eye": each element maps to a row of the element matrix and each row lands on the
values in this file.

## 1 · Panel anatomy — interior margins are law

```
┌─ .panel ──────────────────────────────── radius 14 · 1px border · shadow-2xs ─┐
│ .panel-bar     h 56 · padding 0 10px 0 16px · gap 10 · 1px bottom border      │
│   .eyebrow     mono var(--fz-mono) · tracking 0.14em · uppercase · muted      │
│ .panel-body    padding 18px 18px 16px · flex column · scrolls (thin bar)      │
│   .panel-title serif var(--fz-title 27) · lh 1.08 · tracking -0.02em         │
│                margin 4px 0 8px                                               │
│   .panel-sub   0.96 × fz-body · muted · margin 0 0 18px · max 54ch           │
│   content …                                                                   │
│ .panel-foot    padding 11px 14px · gap 8 · 1px top border · bg secondary      │
└───────────────────────────────────────────────────────────────────────────────┘
```

- Panel widths come from the registry only: **S 380 · M 480 · L 640 · XL 800** (px).
  Never a hardcoded width on a panel.
- Stage: gap `var(--stage-gap, 14px)`, gutter `var(--stage-pad, 18px)` on BOTH sides
  (left padding + end spacer). A flexing panel (canvas/kanban) keeps the right gutter.
- Inside the body, blocks stack with these exact rhythms:
  - `.card` — padding **14px 16px**, radius 12, 1px border, **margin-bottom 16px**;
    its `.lab` mono label gets margin-bottom 6.
  - `.stat` (KPI) — padding **12px 14px**, radius 12; stats sit in a flex row, gap 10.
  - `.drills` — column gap **8px**; `.drill` row — padding **12px 14px**, radius 10,
    internal gap 12, serif lead tile 34×34 radius 9.
  - Popover labels `.pop-sub` — margin-bottom 6.
- The foot is the ONLY action zone: primary CTA = accent bg + **accent border** +
  `--accent-hover` on hover; destructive = `--destructive` text. Never object-state
  toggles (pin/unpin) in the foot; never metadata lines.

## 2 · Type & numbers

| Role | Font | Size | Rules |
|---|---|---|---|
| Display / panel titles | `--font-serif` (Newsreader) | `--fz-title` 27 | weight 400, tracking −0.02em |
| Body | `--font-sans` (Inter) | `--fz-body` 13.5 | derived sizes via `calc(var(--fz-body) * k)` |
| Labels / eyebrows / data | `--font-mono` (Geist Mono) | `--fz-mono` 10 | uppercase, tracking 0.08–0.14em |
| **Every numeric value** | `--font-mono` | context | `tabular-nums`, tracking −0.02em — **numbers are never serif** |

All sizes derive from the three `--fz-*` user preferences. A migrated element with a
pixel font-size that ignores them is a defect.

## 3 · Color — one accent, everything derived

- Base tokens only: `--background --card --secondary --border --rule-1 --foreground
  --muted-foreground --ink-2 --ink-4`.
- ONE accent (`--accent`, default Supreme red oklch(0.578 0.245 27)) with derived ramp:
  `--accent-soft` (pills), `--accent-hover` (hover — **mix in OKLCH, never oklab**:
  oklab browns reds), `--accent-2/3/4` (72/45/22% mixes) for multi-series data.
- Semantic green/amber exist ONLY for do/don't dots and notification dots. Charts,
  statuses, kanban dots, funnels: **accent ramp + neutral inks**.
- Text on accent = `--accent-foreground` (computed by luminance for custom accents).
- Every theme block must define every surface token (light AND dark — a token missing
  from one theme is a defect).

## 4 · Shape, depth, motion

- Radius scale: panels 14 · cards/stats 12 · drills/menus 10 · buttons 7-8 · inputs 7 ·
  pills 9999.
- Shadows: resting `--shadow-2xs`, floating menus `--shadow-md`, drawers `--shadow-lg`.
  The last panel carries the reading-order edge shadow; non-focused panel bars tint
  `--secondary`; pinned panels get `inset 0 2px 0 var(--accent)` on the bar.
- Motion: one ease `var(--ease)` (cubic-bezier(0.32,0.72,0,1)); panels 260ms in,
  menus 100-160ms fade/rise. No springs, no bounces.

## 5 · Icons

- Stroke icons only (Lucide-style): viewBox 24, **strokeWidth 1.6–1.7**, round caps and
  joins, `currentColor`. Sizes: topbar 14, toolbars 15, inline 12-13.
- No emoji in UI chrome — editorial glyphs only (§ ¶ ✶ → ⌄ ✓ ⋯).
- Every icon of the old app is inventoried and either mapped to a stroke equivalent or
  dropped with a decision-log entry.

## 6 · Controls — conversion table (old world → Stax)

| Old element | Stax target |
|---|---|
| Modal / dialog | Panel (size by content) opened to the right — parent stays |
| Tabs | Sibling drills, or in-panel sections (never a tab strip) |
| Dropdown `<select>` for a status-like choice | Segmented `d-btn sm` buttons (filled = active) |
| Native date/time input | Popover calendar (`.dp-pop`, Monday-first) + 30-min time list |
| Inline rename / boxed edit-in-place | `.inline-edit` — text-like, zero border, zero ring |
| Focus ring on inputs | ONE accent-tinted border; outline reserved for buttons (keyboard) |
| Toolbar of 10+ flat buttons | Grouped smart menus (trigger shows active state) |
| Toast / snackbar stacks | `.toast` mono pill, bottom-center |
| Hover affordances | Reveal on row/element hover (edit pencils, open arrows at opacity 0 → 1) |
| Data grid | `.dt` table — sticky mono headers, hairline `--rule-1` grid, borderless cell
  inputs (focus = 5% accent tint), typed cells (mono numbers right-aligned, accent select
  pills, accent checks), row opens as the NEXT panel (a page) |
| Free-form board / diagram | Canvas panel — WhitePaper nodes, 4-way handles, accent edges |
| Wizard | Chained drills with foot CTAs |
| Global search | ⌘K palette |
| Chat / AI helper | Full-height drawer (⌘J) |

## 7 · States — all six, for every element

Every migrated element ships all of: default · hover · focus/active · empty ·
loading/skeleton · error. Empty states are sentences with a next action, not blank
space. A migrated element missing a state keeps its matrix row open.
