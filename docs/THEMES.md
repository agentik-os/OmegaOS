# OmegaOS TUI — Themes

> Pick a theme in 5 seconds: `omega` → Settings → **Theme**. Arrow through the
> list to live-preview each palette on the full screen; **Enter** commits,
> **Esc** reverts to what you had.

The TUI ships 15 palettes. Your choice persists in `~/.omega/config.toml`:

```toml
[general]
theme = "tokyo-night"   # any slug from the table below
```

The theme engine lives in `crates/omega-tui/src/theme.rs` — every chrome color
in the TUI goes through semantic roles, never raw color literals.

---

## The 15 themes

| Slug | Label | Identity | Best for |
|---|---|---|---|
| `omega` | Omega *(default)* | Terminal-native ANSI multicolor — inherits YOUR terminal's palette and background | Any terminal, light or dark |
| `matrix` | Matrix | `#00FF41` green on `#050F05` | Dark terminals |
| `terminal` | Terminal | `#66FF66` green on `#001200` | Dark terminals |
| `amber` | Amber | `#FFB000` amber on `#160D00` | Dark terminals |
| `noir` | Noir | White-on-`#000000` grayscale | Dark terminals |
| `paper` | Paper | `#141414` ink on `#F5F5F0` | **Light terminals** |
| `monogram` | Monogram | `#00FFFF` cyan on `#0A0A0C` | Dark terminals |
| `dracula` | Dracula | `#BD93F9` purple on `#282A36` | Dark terminals |
| `nord` | Nord | `#88C0D0` frost blue on `#2E3440` | Dark terminals |
| `gruvbox` | Gruvbox | `#FE8019` orange on `#282828` | Dark terminals |
| `solarized` | Solarized Dark | `#2AA198` teal on `#002B36` | Dark terminals |
| `tokyo-night` | Tokyo Night | `#7AA2F7` blue on `#1A1B26` | Dark terminals |
| `synthwave` | Synthwave | `#FF71CE` neon pink on `#1A0B2E` | Dark terminals |
| `ocean` | Ocean | `#00AAFF` azure on `#0A1928` | Dark terminals |
| `crimson` | Crimson | `#FF4655` red on `#1A0508` | Dark terminals |

Every theme except Omega paints its own full-screen background and text color,
so the TUI looks the same regardless of what your terminal is set to. Omega is
deliberately the opposite — see [Omega adapts to you](#omega-adapts-to-you).

---

## The semantic state model

Every theme expresses the same four visual states, so once you learn one theme
you've learned them all:

| State | Rendering | Meaning |
|---|---|---|
| **Active** | accent + bold | The thing that's running / alive right now |
| **Passive** | dim | Background info, idle items, secondary labels |
| **Focus / selection** | inverted bar — `sel_fg` text on an accent background | Where your cursor is |
| **Alert** | warn / error colors, distinct from both the background and the active accent | Something needs you: blocked, failed, stuck |

### The Monogram model

All themes follow one design rule: **gray chrome + ONE signature accent.**
Borders, labels, and structure stay quiet grayscale; the single accent color
(Matrix green, Dracula purple, neon pink, …) is reserved for what matters —
active state and your selection. Two mono themes push this to the limit:
Noir and Paper stay grayscale by design, using weight and inversion instead
of hue for hierarchy.

---

## The contrast contract

Readability isn't a vibe — it's a unit-tested invariant. Tests in `theme.rs`
enforce WCAG-AA contrast ratios for **every** palette:

- **Text roles** (`text`, `dim`, `info`, `error`, `warn`) vs the theme
  background: **≥ 4.5:1** (AA for normal text).
- **Accent/graphic roles** (`accent`, `accent2`, `success`, `special`,
  `dim2`) vs the background: **≥ 3.0:1** (AA for large text / UI components).
- **Selection bar**: `sel_fg` on the accent background: **≥ 4.5:1** — the
  inverted cursor bar is always readable.
- **Hierarchy**: `dim2 < dim < text` in luminance — quieter roles are
  measurably quieter, but never below the AA floor.

A palette that regresses below any threshold fails the test suite, so no theme
can ship unreadable. Omega is exempt by design: it uses ANSI named colors, so
its actual contrast is whatever your terminal's palette delivers (and adapting
to that palette is its whole point).

---

## Terminal & Termius guidance

### Truecolor themes render identically everywhere

Every theme except Omega uses 24-bit RGB (truecolor). Truecolor bytes go
straight to the screen — they never pass through your terminal's 16-color
palette — so Matrix is the exact same `#00FF41` in iTerm, Alacritty, kitty,
Windows Terminal, and **Termius**. If a truecolor theme looks wrong, your
emulator or multiplexer is likely downgrading truecolor; verify support with:

```
msgcat --color=test
```

Smooth gradients = truecolor works. Banded blocks = your terminal (or a hop in
between, like an old tmux/screen) is quantizing to 256 colors.

### Omega adapts to you

Omega, the default, is built from ANSI *named* colors (red, green, blue, …),
so it inherits your terminal/Termius color profile — background included.
That's its feature, not a limitation: it follows your light/dark preference
and matches whatever scheme you've tuned your terminal to. If you want the
TUI to look like *your* terminal, use Omega; if you want it to look the same
on every machine, pick any other theme.

### Agent panes keep the terminal's real palette

The session-pane preview passes agent output through untouched — agent panes
are **never re-themed**, on any theme. Claude (and any CLI agent) emits ANSI
named colors, and those resolve through your terminal's palette. So if dark
blue agent output is unreadable, the fix is your terminal/Termius color
profile, not the OmegaOS theme. In Termius: **Settings → Appearance → Color
Scheme** — pick (or edit) a scheme whose blue is legible on your background.
