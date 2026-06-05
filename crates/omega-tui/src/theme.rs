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
    /// Full-screen background. `None` = keep the terminal's own background
    /// (the Omega default). `Some(..)` is what makes a theme unmistakably
    /// different at a glance — Dracula's purple, Paper's white, etc.
    pub bg: Option<Color>,
    /// Body text color. `Color::Reset` (terminal default) for bg-less themes;
    /// themes that paint a background MUST set an explicit readable text
    /// color (the terminal's default fg may be invisible on the painted bg).
    pub text: Color,
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
        bg: None,
        text: Color::Reset,
        accent_hi: accent,
        accent2_hi: accent2,
        success_hi: success,
        error_hi: error,
        info_hi: info,
        special_hi: special,
    }
}

/// The "Monogram model" (user-validated): a quiet grayscale chrome + ONE
/// signature accent color. Every theme except Omega (terminal-native),
/// Noir and Paper (pure mono by design) is built from this template — the
/// palettes differ only by their accent, background and text tint.
const fn mono(accent: Color, bg: Color, text: Color) -> Theme {
    Theme {
        bg: Some(bg),
        text,
        ..mk(
            accent,
            Color::Rgb(220, 220, 220), // accent2 — soft near-white highlight
            Color::Rgb(200, 200, 200), // success
            Color::White,              // error
            Color::Rgb(180, 180, 180), // info
            Color::Rgb(190, 190, 190), // special
            Color::Rgb(115, 115, 115), // dim
            Color::Rgb(70, 70, 70),    // dim2
            Color::White,              // bright
            Color::Black,              // sel_fg
        )
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

/// Matrix — mono chrome + neon Matrix green.
const MATRIX: Theme = mono(
    Color::Rgb(0, 255, 65),
    Color::Rgb(5, 15, 5),
    Color::Rgb(210, 230, 210),
);

/// Terminal — mono chrome + soft phosphor green.
const TERMINAL: Theme = mono(
    Color::Rgb(102, 255, 102),
    Color::Rgb(0, 18, 0),
    Color::Rgb(200, 225, 200),
);

/// Amber — mono chrome + retro amber phosphor.
const AMBER: Theme = mono(
    Color::Rgb(255, 176, 0),
    Color::Rgb(22, 13, 0),
    Color::Rgb(235, 220, 200),
);

/// Noir — full black & white, pure grayscale on dark terminals.
const NOIR: Theme = Theme {
    bg: Some(Color::Rgb(0, 0, 0)),
    text: Color::Rgb(220, 220, 220),
    ..mk(
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
)
};

/// Paper — full white: ink-on-paper for LIGHT terminal backgrounds.
const PAPER: Theme = Theme {
    bg: Some(Color::Rgb(245, 245, 240)),
    text: Color::Rgb(20, 20, 20),
    ..mk(
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
)
};

/// Monogram — the original: mono chrome + cyan.
const MONOGRAM: Theme = mono(
    Color::Cyan,
    Color::Rgb(10, 10, 12),
    Color::Rgb(225, 225, 225),
);

/// Dracula — mono chrome + Dracula purple.
const DRACULA: Theme = mono(
    Color::Rgb(189, 147, 249),
    Color::Rgb(40, 42, 54),
    Color::Rgb(235, 235, 240),
);

/// Nord — mono chrome + arctic ice blue.
const NORD: Theme = mono(
    Color::Rgb(136, 192, 208),
    Color::Rgb(46, 52, 64),
    Color::Rgb(222, 228, 235),
);

/// Gruvbox — mono chrome + warm gruvbox orange.
const GRUVBOX: Theme = mono(
    Color::Rgb(254, 128, 25),
    Color::Rgb(40, 40, 40),
    Color::Rgb(230, 220, 200),
);

/// Solarized — mono chrome + solarized teal.
const SOLARIZED: Theme = mono(
    Color::Rgb(42, 161, 152),
    Color::Rgb(0, 43, 54),
    Color::Rgb(200, 210, 205),
);

/// Tokyo Night — mono chrome + Tokyo Night blue.
const TOKYO_NIGHT: Theme = mono(
    Color::Rgb(122, 162, 247),
    Color::Rgb(26, 27, 38),
    Color::Rgb(215, 220, 240),
);

/// Synthwave — mono chrome + neon pink.
const SYNTHWAVE: Theme = mono(
    Color::Rgb(255, 113, 206),
    Color::Rgb(26, 11, 46),
    Color::Rgb(235, 225, 245),
);

/// Ocean — mono chrome + deep sea blue.
const OCEAN: Theme = mono(
    Color::Rgb(0, 170, 255),
    Color::Rgb(10, 25, 40),
    Color::Rgb(210, 225, 240),
);

/// Crimson — mono chrome + alert red.
const CRIMSON: Theme = mono(
    Color::Rgb(255, 70, 85),
    Color::Rgb(26, 5, 8),
    Color::Rgb(240, 215, 215),
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
            ThemeId::Matrix => "Mono chrome + neon Matrix green",
            ThemeId::Terminal => "Mono chrome + soft phosphor green",
            ThemeId::Amber => "Mono chrome + retro amber",
            ThemeId::Noir => "Pure grayscale for dark terminals",
            ThemeId::Paper => "Ink on paper — for LIGHT terminal backgrounds",
            ThemeId::Monogram => "Monochrome chrome, one cyan accent",
            ThemeId::Dracula => "Mono chrome + Dracula purple",
            ThemeId::Nord => "Mono chrome + arctic ice blue",
            ThemeId::Gruvbox => "Mono chrome + warm gruvbox orange",
            ThemeId::Solarized => "Mono chrome + solarized teal",
            ThemeId::TokyoNight => "Mono chrome + Tokyo Night blue",
            ThemeId::Synthwave => "Mono chrome + neon pink",
            ThemeId::Ocean => "Mono chrome + deep sea blue",
            ThemeId::Crimson => "Mono chrome + alert red",
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
/// Full-screen background override; `None` keeps the terminal's own bg.
pub fn bg() -> Option<Color> {
    cur().bg
}
/// Body text color (Reset for bg-less themes, explicit otherwise).
pub fn text() -> Color {
    cur().text
}
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
