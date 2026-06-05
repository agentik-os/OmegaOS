# Changelog

All notable changes to OmegaOS are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project aims
for [semantic versioning](https://semver.org) once it reaches 1.0. Until then,
`main` is the only supported line.

## [Unreleased]

### Added
- WCAG-AA contrast contract in the TUI theme engine, enforced by unit tests:
  text/dim/info/error/warn vs background ≥ 4.5:1, accent/graphic roles ≥ 3.0:1,
  selection text on the accent bar ≥ 4.5:1, and a `dim2 < dim < text` luminance
  hierarchy — palettes can no longer regress below readable. New semantic
  `warn` role (the blocked badge is now themed; it was hardcoded orange), a
  distinct alert red in the mono themes, and per-theme dim/dim2 retuned to meet
  AA while staying visually quieter than body text. Omega (terminal-native
  ANSI) is exempt by design. Documented in `docs/THEMES.md`.
- TUI theme selector (Settings → Theme): 15 selectable palettes — Omega
  (default), Matrix, Terminal, Amber, Noir, Paper, Monogram, Dracula, Nord,
  Gruvbox, Solarized Dark, Tokyo Night, Synthwave, Ocean, Crimson. Every
  theme except Omega paints its own full-screen background and text color
  and follows the "Monogram model": a quiet grayscale chrome plus ONE
  signature accent per theme (Matrix green, Dracula purple, neon pink, …);
  Omega keeps the terminal's own background and classic multicolor chrome. The selector live-previews each theme while arrowing through
  it, the gallery renders each row on that theme's background, the choice
  persists in `~/.omega/config.toml` (`theme = "..."`), and every chrome
  color in the TUI goes through semantic theme roles. The session-pane
  preview keeps the agent's own colors untouched.
- GitHub Actions CI: build the workspace with `-D warnings` and run the test
  suite as hard gates; clippy and rustfmt run as advisory steps.
- Hand-written, human-voiced README with French, Russian, and Chinese
  translations, plus a "How a mission runs" section explaining the
  Master → oracle → worker → workflow flow.
- Contributor docs: this changelog, `CONTRIBUTING.md`, `SECURITY.md`,
  `CODE_OF_CONDUCT.md`, and issue/PR templates.

### Changed
- Terminal output (TUI and CLI) is now emoji-free, using the `[+]/[~]/[x]`
  ASCII convention. Telegram messages keep their emoji.

### Fixed
- `credentials` test that was flaky under parallel runs (it mutated the global
  `HOME`); the HOME-touching tests are now serialized.
- Dead code removed across the orchestration and TUI crates so the workspace
  builds with zero warnings.
- Oracle respawn no longer trusts a stale registry entry, and the patrol daemon
  re-checks session liveness before auto-marking a worker done.

## [0.1.0]

Initial public cut. The `omega` CLI and TUI, the rmux-backed session model, the
typed doctrine (6 Laws and 20 Rules) injected into every dispatched agent, the
oracle/worker orchestration layer, the Quality Arsenal audits, and the optional
Telegram bridge.
