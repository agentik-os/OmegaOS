# Changelog

All notable changes to OmegaOS are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project aims
for [semantic versioning](https://semver.org) once it reaches 1.0. Until then,
`main` is the only supported line.

## [Unreleased]

### Added
- Guided Telegram setup in the npx installer (`omega-os` 1.5.0): before the
  Matrix animation takes the screen, an interactive wizard walks through
  BotFather bot creation, validates the token live (`getMe`), auto-detects
  your chat id from your first message to the bot (`getUpdates`), then wires
  everything via `omega telegram setup` once install.sh succeeds. Skipped
  when non-interactive (no TTY / CI), with `--no-telegram`, or when
  `~/.omega/telegram.toml` already exists (re-installs keep the config).
  Queue-based prompt input — multi-line pastes and scripted stdin are never
  dropped; stdin EOF degrades to "skip", never a hang.
- WCAG-AA contrast contract in the TUI theme engine, enforced by unit tests:
  every text-bearing role (text/dim/info/error/warn/bright and the accent
  family) ≥ 4.5:1 vs background, selection text ≥ 4.5:1 on both accent and
  accent2 bars, a contrast-vs-background hierarchy `dim2 < dim < text` (the
  raw luminance order inverts on light themes), and a role-vs-role
  distinctness floor — warn and error vs the accent at CIE76 ΔE ≥ 30, so an
  alert never blends into active text (Noir and Paper exempt, mono by
  design) — palettes can no longer regress below readable. New semantic
  `warn` role (the blocked badge is now themed; it was hardcoded orange);
  orange-accent themes (Amber, Gruvbox) move warn to the alert-red family;
  per-theme dim/dim2 retuned to meet AA while staying visually quieter than
  body text. Omega stays 100% named ANSI — warn included, an adaptive light
  red — so it keeps inheriting the terminal's own palette. Documented in
  `docs/THEMES.md`.
- TUI theme selector (Settings → Theme): 17 selectable palettes — Omega
  (default), Matrix, Terminal, Amber, Noir, Paper, Monogram, Dracula, Nord,
  Gruvbox, Solarized Dark, Tokyo Night, Synthwave, Ocean, Crimson, plus
  Transparent Dark/Light (no painted background — the terminal's own bg and
  transparency show through, with fixed white/black ink). Every
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
- Pasting very long text (≳8 KB, e.g. 10k characters) into an attached rmux
  client corrupted the paste: rmux pin bumped `726d9e7` → `4455da0`, whose
  stateful `PasteFilter` keeps a paste spanning several client `read()`
  bursts ONE bracketed block. Previously the per-burst heuristic re-wrapped
  the middle bursts of a host-bracketed paste (the synthetic `201~` closed
  the paste early — the rest went in raw and every embedded newline submitted
  as Enter), and split an unbracketed (SSH/Termius) paste into several
  `[Pasted text]` blocks. Runtime A/B proven with a 10,200-byte paste
  recorded off the pane PTY: old = 2-3 corrupted blocks with an injected
  `\r` at the read boundary, new = 1 block, body intact byte-for-byte.
- Telegram bot never ran on macOS: the service install was systemd-only, so
  `omega telegram setup` wrote the config but every message went unanswered.
  `install.sh` now installs a launchd LaunchAgent on Darwin
  (`~/Library/LaunchAgents/os.omega.tg-bot.plist`, RunAtLoad + KeepAlive,
  logs in `~/.omega/logs/tg-bot.log`) with the same semantics as the Linux
  unit: always running, waits for the token, auto-restarts.
- macOS install hung at ~20% behind the npx Matrix animation: Phase 2 wrote
  the Debian-ism `/etc/default/locale` via `sudo`, whose password prompt was
  invisible behind the full-screen animation. `ensure_utf8_locale` now no-ops
  on Darwin (natively UTF-8), every Phase-2 sudo is non-interactive
  (`sudo -n` — fail loud, never prompt), and `bootstrap_os_packages` gains a
  Homebrew branch plus a Darwin-without-brew soft path (missing rsync/jq
  warn-and-continue; only git/curl are fatal).
- Mouse-wheel scroll dead in every rmux pane (regression in rmux `0e4abb2`):
  the client-side paste heuristic wrapped a batched wheel burst (3+ SGR mouse
  reports ≥ 32 bytes, no newline) as a bracketed paste, so the server pasted
  the sequences into the PTY instead of decoding scroll. rmux pin bumped to
  `726d9e7`, which exempts ESC-initiated bursts from paste synthesis.
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
