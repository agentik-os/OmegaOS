//! TUI theme engine — semantic color roles + selectable palettes.
//!
//! Every chrome color in `ui.rs` goes through a semantic accessor here
//! (`th::accent()`, `th::success()`, …) instead of a hardcoded
//! `Color::Cyan`. The active theme is a process-global (lock-free
//! `AtomicU8`) so switching it in Settings re-skins the whole TUI on the
//! very next frame — no plumbing through every render function.
//!
//! NOT themed on purpose: the session-pane preview passthrough
//! (`preview_to_color` in ui.rs) — that is the agent's own output and must
//! keep the terminal's real palette.

use ratatui::style::Color;
use std::sync::atomic::{AtomicU8, Ordering};

/// Semantic color roles for the TUI chrome. Mapping from the legacy
/// hardcoded palette: Cyan→accent, Yellow→accent2, Green→success,
/// Red→error, Blue→info, Magenta→special, Gray→dim, DarkGray→dim2,
/// White→bright, Black→sel_fg (text on accent-colored selection bars).
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub accent: Color,
    pub accent2: Color,
    pub success: Color,
    pub error: Color,
    pub info: Color,
    pub special: Color,
    pub dim: Color,
    pub dim2: Color,
    pub bright: Color,
    pub sel_fg: Color,
    // Bright variants (legacy Light* colors — rare call sites).
    pub accent_hi: Color,
    pub accent2_hi: Color,
    pub success_hi: Color,
    pub error_hi: Color,
    pub info_hi: Color,
    pub special_hi: Color,
}

/// Compact const constructor: the `*_hi` variants default to their base
/// role; themes that want distinct brights override via struct update.
#[allow(clippy::too_many_arguments)]
const fn mk(
    accent: Color,
    accent2: Color,
    success: Color,
    error: Color,
    info: Color,
    special: Color,
    dim: Color,
    dim2: Color,
    bright: Color,
    sel_fg: Color,
) -> Theme {
    Theme {
        accent,
        accent2,
        success,
        error,
        info,
        special,
        dim,
        dim2,
        bright,
        sel_fg,
        accent_hi: accent,
        accent2_hi: accent2,
        success_hi: success,
        error_hi: error,
        info_hi: info,
        special_hi: special,
    }
}

// ── Palettes ────────────────────────────────────────────────────────────────

/// Omega (default) — the classic cyan/yellow chrome, 16-color named ANSI so
/// it adapts to the terminal's own scheme (light and dark).
const OMEGA: Theme = Theme {
    accent_hi: Color::LightCyan,
    accent2_hi: Color::LightYellow,
    success_hi: Color::LightGreen,
    error_hi: Color::LightRed,
    info_hi: Color::LightBlue,
    special_hi: Color::LightMagenta,
    ..mk(
        Color::Cyan,
        Color::Yellow,
        Color::Green,
        Color::Red,
        Color::Blue,
        Color::Magenta,
        Color::Gray,
        Color::DarkGray,
        Color::White,
        Color::Black,
    )
};

/// Matrix — neon digital-rain green.
const MATRIX: Theme = mk(
    Color::Rgb(0, 255, 65),
    Color::Rgb(160, 255, 160),
    Color::Rgb(0, 230, 90),
    Color::Rgb(255, 110, 110),
    Color::Rgb(0, 200, 120),
    Color::Rgb(120, 255, 170),
    Color::Rgb(0, 140, 40),
    Color::Rgb(0, 95, 30),
    Color::Rgb(200, 255, 200),
    Color::Black,
);

/// Terminal — soft VT220 green phosphor, calmer than Matrix.
const TERMINAL: Theme = mk(
    Color::Rgb(102, 255, 102),
    Color::Rgb(170, 255, 170),
    Color::Rgb(102, 255, 102),
    Color::Rgb(255, 140, 110),
    Color::Rgb(120, 220, 140),
    Color::Rgb(150, 255, 180),
    Color::Rgb(70, 150, 70),
    Color::Rgb(45, 100, 45),
    Color::Rgb(210, 255, 210),
    Color::Black,
);

/// Amber — retro amber phosphor (P3 CRT).
const AMBER: Theme = mk(
    Color::Rgb(255, 176, 0),
    Color::Rgb(255, 200, 80),
    Color::Rgb(255, 190, 60),
    Color::Rgb(255, 110, 40),
    Color::Rgb(230, 160, 30),
    Color::Rgb(255, 210, 130),
    Color::Rgb(165, 115, 20),
    Color::Rgb(110, 75, 10),
    Color::Rgb(255, 230, 160),
    Color::Black,
);

/// Noir — full black & white, pure grayscale on dark terminals.
const NOIR: Theme = mk(
    Color::White,
    Color::Rgb(200, 200, 200),
    Color::Rgb(190, 190, 190),
    Color::Rgb(255, 255, 255),
    Color::Rgb(170, 170, 170),
    Color::Rgb(215, 215, 215),
    Color::Rgb(120, 120, 120),
    Color::Rgb(75, 75, 75),
    Color::White,
    Color::Black,
);

/// Paper — full white: ink-on-paper for LIGHT terminal backgrounds.
const PAPER: Theme = mk(
    Color::Rgb(20, 20, 20),
    Color::Rgb(70, 70, 70),
    Color::Rgb(50, 50, 50),
    Color::Rgb(0, 0, 0),
    Color::Rgb(90, 90, 90),
    Color::Rgb(60, 60, 60),
    Color::Rgb(150, 150, 150),
    Color::Rgb(185, 185, 185),
    Color::Rgb(0, 0, 0),
    Color::Rgb(255, 255, 255),
);

/// Monogram — monochrome chrome with a single cyan accent.
const MONOGRAM: Theme = mk(
    Color::Cyan,
    Color::Rgb(220, 220, 220),
    Color::Rgb(200, 200, 200),
    Color::White,
    Color::Rgb(180, 180, 180),
    Color::Rgb(190, 190, 190),
    Color::Rgb(115, 115, 115),
    Color::Rgb(70, 70, 70),
    Color::White,
    Color::Black,
);

/// Dracula — the classic purple/pink/cyan dark palette.
const DRACULA: Theme = mk(
    Color::Rgb(189, 147, 249),
    Color::Rgb(241, 250, 140),
    Color::Rgb(80, 250, 123),
    Color::Rgb(255, 85, 85),
    Color::Rgb(139, 233, 253),
    Color::Rgb(255, 121, 198),
    Color::Rgb(98, 114, 164),
    Color::Rgb(68, 71, 90),
    Color::Rgb(248, 248, 242),
    Color::Rgb(40, 42, 54),
);

/// Nord — cool arctic blues.
const NORD: Theme = mk(
    Color::Rgb(136, 192, 208),
    Color::Rgb(235, 203, 139),
    Color::Rgb(163, 190, 140),
    Color::Rgb(191, 97, 106),
    Color::Rgb(129, 161, 193),
    Color::Rgb(180, 142, 173),
    Color::Rgb(124, 135, 156),
    Color::Rgb(76, 86, 106),
    Color::Rgb(236, 239, 244),
    Color::Rgb(46, 52, 64),
);

/// Gruvbox — warm retro earth tones.
const GRUVBOX: Theme = mk(
    Color::Rgb(254, 128, 25),
    Color::Rgb(250, 189, 47),
    Color::Rgb(184, 187, 38),
    Color::Rgb(251, 73, 52),
    Color::Rgb(131, 165, 152),
    Color::Rgb(211, 134, 155),
    Color::Rgb(146, 131, 116),
    Color::Rgb(102, 92, 84),
    Color::Rgb(235, 219, 178),
    Color::Rgb(40, 40, 40),
);

/// Solarized Dark — the Ethan Schoonover classic.
const SOLARIZED: Theme = mk(
    Color::Rgb(38, 139, 210),
    Color::Rgb(181, 137, 0),
    Color::Rgb(133, 153, 0),
    Color::Rgb(220, 50, 47),
    Color::Rgb(42, 161, 152),
    Color::Rgb(211, 54, 130),
    Color::Rgb(101, 123, 131),
    Color::Rgb(88, 110, 117),
    Color::Rgb(238, 232, 213),
    Color::Rgb(0, 43, 54),
);

/// Tokyo Night — neon-soft blues and purples.
const TOKYO_NIGHT: Theme = mk(
    Color::Rgb(122, 162, 247),
    Color::Rgb(224, 175, 104),
    Color::Rgb(158, 206, 106),
    Color::Rgb(247, 118, 142),
    Color::Rgb(125, 207, 255),
    Color::Rgb(187, 154, 247),
    Color::Rgb(86, 95, 137),
    Color::Rgb(59, 66, 97),
    Color::Rgb(192, 202, 245),
    Color::Rgb(26, 27, 38),
);

/// Synthwave — cyberpunk neon pink/cyan.
const SYNTHWAVE: Theme = mk(
    Color::Rgb(255, 113, 206),
    Color::Rgb(255, 251, 150),
    Color::Rgb(5, 255, 161),
    Color::Rgb(255, 71, 87),
    Color::Rgb(1, 205, 254),
    Color::Rgb(185, 103, 255),
    Color::Rgb(140, 100, 160),
    Color::Rgb(90, 60, 110),
    Color::Rgb(240, 230, 255),
    Color::Rgb(20, 10, 35),
);

/// Ocean — deep sea blues.
const OCEAN: Theme = mk(
    Color::Rgb(0, 170, 255),
    Color::Rgb(137, 221, 255),
    Color::Rgb(92, 207, 230),
    Color::Rgb(255, 107, 107),
    Color::Rgb(72, 140, 255),
    Color::Rgb(130, 170, 255),
    Color::Rgb(84, 110, 140),
    Color::Rgb(52, 70, 90),
    Color::Rgb(214, 233, 252),
    Color::Rgb(10, 25, 40),
);

/// Crimson — red-alert command deck.
const CRIMSON: Theme = mk(
    Color::Rgb(255, 70, 85),
    Color::Rgb(255, 160, 110),
    Color::Rgb(255, 200, 120),
    Color::Rgb(255, 40, 55),
    Color::Rgb(255, 120, 140),
    Color::Rgb(255, 90, 160),
    Color::Rgb(160, 85, 90),
    Color::Rgb(100, 52, 58),
    Color::Rgb(255, 222, 222),
    Color::Black,
);

// ── Registry ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ThemeId {
    Omega = 0,
    Matrix = 1,
    Terminal = 2,
    Amber = 3,
    Noir = 4,
    Paper = 5,
    Monogram = 6,
    Dracula = 7,
    Nord = 8,
    Gruvbox = 9,
    Solarized = 10,
    TokyoNight = 11,
    Synthwave = 12,
    Ocean = 13,
    Crimson = 14,
}

impl ThemeId {
    pub fn all() -> &'static [ThemeId] {
        &[
            ThemeId::Omega,
            ThemeId::Matrix,
            ThemeId::Terminal,
            ThemeId::Amber,
            ThemeId::Noir,
            ThemeId::Paper,
            ThemeId::Monogram,
            ThemeId::Dracula,
            ThemeId::Nord,
            ThemeId::Gruvbox,
            ThemeId::Solarized,
            ThemeId::TokyoNight,
            ThemeId::Synthwave,
            ThemeId::Ocean,
            ThemeId::Crimson,
        ]
    }

    /// Stable slug persisted in config.toml (`theme = "..."`).
    pub fn slug(self) -> &'static str {
        match self {
            ThemeId::Omega => "omega",
            ThemeId::Matrix => "matrix",
            ThemeId::Terminal => "terminal",
            ThemeId::Amber => "amber",
            ThemeId::Noir => "noir",
            ThemeId::Paper => "paper",
            ThemeId::Monogram => "monogram",
            ThemeId::Dracula => "dracula",
            ThemeId::Nord => "nord",
            ThemeId::Gruvbox => "gruvbox",
            ThemeId::Solarized => "solarized",
            ThemeId::TokyoNight => "tokyo-night",
            ThemeId::Synthwave => "synthwave",
            ThemeId::Ocean => "ocean",
            ThemeId::Crimson => "crimson",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ThemeId::Omega => "Omega (default)",
            ThemeId::Matrix => "Matrix",
            ThemeId::Terminal => "Terminal (green phosphor)",
            ThemeId::Amber => "Amber (retro phosphor)",
            ThemeId::Noir => "Noir (full black)",
            ThemeId::Paper => "Paper (full white)",
            ThemeId::Monogram => "Monogram (mono + cyan)",
            ThemeId::Dracula => "Dracula",
            ThemeId::Nord => "Nord",
            ThemeId::Gruvbox => "Gruvbox",
            ThemeId::Solarized => "Solarized Dark",
            ThemeId::TokyoNight => "Tokyo Night",
            ThemeId::Synthwave => "Synthwave",
            ThemeId::Ocean => "Ocean",
            ThemeId::Crimson => "Crimson",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            ThemeId::Omega => "Classic cyan/yellow — adapts to your terminal's own scheme",
            ThemeId::Matrix => "Neon digital-rain green",
            ThemeId::Terminal => "Soft VT220 green phosphor",
            ThemeId::Amber => "Warm amber CRT phosphor",
            ThemeId::Noir => "Pure grayscale for dark terminals",
            ThemeId::Paper => "Ink on paper — for LIGHT terminal backgrounds",
            ThemeId::Monogram => "Monochrome chrome, one cyan accent",
            ThemeId::Dracula => "Purple, pink & cyan dark classic",
            ThemeId::Nord => "Cool arctic blues",
            ThemeId::Gruvbox => "Warm retro earth tones",
            ThemeId::Solarized => "The Solarized Dark classic",
            ThemeId::TokyoNight => "Neon-soft blues and purples",
            ThemeId::Synthwave => "Cyberpunk neon pink & cyan",
            ThemeId::Ocean => "Deep sea blues",
            ThemeId::Crimson => "Red-alert command deck",
        }
    }

    pub fn palette(self) -> &'static Theme {
        match self {
            ThemeId::Omega => &OMEGA,
            ThemeId::Matrix => &MATRIX,
            ThemeId::Terminal => &TERMINAL,
            ThemeId::Amber => &AMBER,
            ThemeId::Noir => &NOIR,
            ThemeId::Paper => &PAPER,
            ThemeId::Monogram => &MONOGRAM,
            ThemeId::Dracula => &DRACULA,
            ThemeId::Nord => &NORD,
            ThemeId::Gruvbox => &GRUVBOX,
            ThemeId::Solarized => &SOLARIZED,
            ThemeId::TokyoNight => &TOKYO_NIGHT,
            ThemeId::Synthwave => &SYNTHWAVE,
            ThemeId::Ocean => &OCEAN,
            ThemeId::Crimson => &CRIMSON,
        }
    }

    pub fn from_slug(s: &str) -> Option<ThemeId> {
        ThemeId::all().iter().copied().find(|t| t.slug() == s)
    }

    fn from_u8(n: u8) -> ThemeId {
        ThemeId::all()
            .get(n as usize)
            .copied()
            .unwrap_or(ThemeId::Omega)
    }
}

// ── Global active theme ─────────────────────────────────────────────────────

static ACTIVE: AtomicU8 = AtomicU8::new(ThemeId::Omega as u8);

/// Switch the active theme — takes effect on the next rendered frame.
pub fn set_active(id: ThemeId) {
    ACTIVE.store(id as u8, Ordering::Relaxed);
}

/// Set the active theme from a config slug; unknown slugs fall back to Omega.
pub fn set_active_slug(slug: &str) {
    set_active(ThemeId::from_slug(slug).unwrap_or(ThemeId::Omega));
}

pub fn active() -> ThemeId {
    ThemeId::from_u8(ACTIVE.load(Ordering::Relaxed))
}

fn cur() -> &'static Theme {
    active().palette()
}

// Semantic accessors — the only color API ui.rs should use for chrome.
pub fn accent() -> Color {
    cur().accent
}
pub fn accent2() -> Color {
    cur().accent2
}
pub fn success() -> Color {
    cur().success
}
pub fn error() -> Color {
    cur().error
}
pub fn info() -> Color {
    cur().info
}
pub fn special() -> Color {
    cur().special
}
pub fn dim() -> Color {
    cur().dim
}
pub fn dim2() -> Color {
    cur().dim2
}
pub fn bright() -> Color {
    cur().bright
}
pub fn sel_fg() -> Color {
    cur().sel_fg
}
pub fn accent_hi() -> Color {
    cur().accent_hi
}
pub fn accent2_hi() -> Color {
    cur().accent2_hi
}
pub fn success_hi() -> Color {
    cur().success_hi
}
pub fn error_hi() -> Color {
    cur().error_hi
}
pub fn info_hi() -> Color {
    cur().info_hi
}
pub fn special_hi() -> Color {
    cur().special_hi
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_roundtrip() {
        for id in ThemeId::all() {
            assert_eq!(ThemeId::from_slug(id.slug()), Some(*id), "slug {}", id.slug());
        }
        assert_eq!(ThemeId::from_slug("nope"), None);
    }

    #[test]
    fn set_active_slug_falls_back_to_omega() {
        set_active(ThemeId::Matrix);
        set_active_slug("does-not-exist");
        assert_eq!(active(), ThemeId::Omega);
    }

    #[test]
    fn fifteen_themes() {
        assert_eq!(ThemeId::all().len(), 15);
    }
}
