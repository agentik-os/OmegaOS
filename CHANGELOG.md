# Changelog

All notable changes to OmegaOS are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project aims
for [semantic versioning](https://semver.org) once it reaches 1.0. Until then,
`main` is the only supported line.

## [Unreleased]

### Added
- **llm-council** — convene four different Claude models on one question:
  independent answers in parallel, anonymous peer review, an Opus president
  synthesizes the verdict and surfaces the dissent. Ships as the
  `/omg-llm-council` skill, the `@council` Matrix agent (the 14th), and the
  R-COUNCIL rule. Runs 100% on the Workflow primitive inside your existing
  Claude Code session — no API keys, no extra cost.
- **browser-use** — agentic cloud-browser skill (`/omg-browser-use`) plus the
  R-BROWSER rule (when to use agentic browsing vs scripted Playwright), with
  least-privilege key handling and error redaction.
- **Marketing + visual-identity pack** — 10 vendored skills (market-research,
  marketing-strategist, product-marketing-context, content-strategy,
  social-content, cold-email, ad-creative, launch-strategy, and the Higgsfield
  soul-id/generate pair), governed by the new R-MARKETING and R-VISUAL-ID rules.
- **Worktree isolation for parallel workers** — `omega spawn-worker --worktree`
  gives each parallel worker its own git worktree on top of the scope-claim
  file locks, with a clean merge back when workers finish.
- **Mission PDF reports** — every mission ends with a branded PDF report
  (including a linked "steps to verify" section) delivered to the project's
  Telegram topic.
- **Live Telegram progress card** — one card per oracle, edited in place as
  plan tasks complete (`omega progress`), with a hard flood ceiling so a
  mission never spams the topic.
- **Maintenance skills** — `cleanup` (disk/session/cache hygiene),
  `project-tidy` (de-sprawl a repo polluted by agent output), and `ramflush`
  (kernel cache purge + perf report), wired into the Telegram bot's Clean hub.
  New R-SKILLPUB rule: every new skill ships to the library + OmegaOS.
- **Companion agent bots** — a "Cowork" bot kind: an instant Haiku co-worker on
  its own Telegram bot, evolved into the self-improving personal companion
  (inline `/menu`, Composio app connections).
- **Deposit bot** (`omega-os` 1.5.4) — a private Telegram inbox so the operator
  can send photos/notes from their phone that an agent reads in `~/.omega/inbox/`
  (timestamped, captioned, indexed). Ships + auto-starts like the command bot
  (`telegram-bot/inbox-bot.ts`, `omega-inbox-bot.service`); connect with
  `inbox-bot-up <BOT_TOKEN>` (or `OMEGA_DEPOSIT_TOKEN=<TOKEN> inbox-bot-up`).
  Token in `~/.omega/deposit.toml` (0600, R-TGSEC); the bot self-locks to the
  first chat that messages it. The command bot now ingests **any** file type the
  operator sends, not just images.
- Claude **Fable 5** (`claude-fable-5`) wired into the provider catalog and
  alias resolver.

### Changed
- The canonical Telegram setup command is the env-prefix form
  (`OMEGA_TG_TOKEN=<TOKEN> omega telegram setup …`) everywhere — installer
  wizard, runtime echoes, and docs; the token never appears in argv.

### Fixed
- Doctor: robust Telegram-bot poller detection, no false "duplicate pollers"
  warning on a headless Mac, and the expected-rules counter follows the
  registry.
- macOS release builds: the retired `macos-13` Intel runner is replaced by
  cross-compiling `x86_64-apple-darwin` from `macos-14`.

## [0.1.5] — 2026-06-06

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
- **`/omg-acceptance`** — autonomous browser-acceptance + self-heal gate:
  Playwright-sweeps every route, captures every console error and failed
  network request, walks the authenticated golden path with a real persisted
  write, then fixes what it finds and re-runs until green. Builds must prove
  they WORK at runtime, not just compile.
- Transparent Dark & Light TUI themes (no painted background — the terminal's
  own bg shows through), bringing the gallery to 17 palettes; the Omega chrome
  theme applied to rmux itself; a 1-row/2-col breathing-room margin around the
  whole TUI.

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
- Five adversarial hardening passes (fix4–fix7b) across the TUI, CLI/core,
  installer, and Telegram bot: state-driven confirm flows (no armed-confirm
  class), a11y/uiux remediation of the theme engine, tokens kept off argv,
  portable poller healing, runtime bun resolution in agent-bot units, and
  npm-wizard failure-path hardening.

## [0.1.4] — 2026-06-05

### Added
- TUI theme selector (Settings → Theme): 15 selectable palettes — Omega
  (default), Matrix, Terminal, Amber, Noir, Paper, Monogram, Dracula, Nord,
  Gruvbox, Solarized Dark, Tokyo Night, Synthwave, Ocean, Crimson. Every
  theme except Omega paints its own full-screen background and text color
  and follows the "Monogram model": a quiet grayscale chrome plus ONE
  signature accent per theme (Matrix green, Dracula purple, neon pink, …);
  Omega keeps the terminal's own background and classic multicolor chrome.
  The selector live-previews each theme while arrowing through it, the
  gallery renders each row on that theme's background, the choice persists
  in `~/.omega/config.toml` (`theme = "..."`), and every chrome color in the
  TUI goes through semantic theme roles. The session-pane preview keeps the
  agent's own colors untouched.
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
- `omega plan-run` strict pre-run validation: refuses skip-prone or
  fake-completing plans (trivial `verify_command`s rejected), and every
  worker brief gets the mandatory `omega done <session>` completion signal
  injected — the reason builds used to stall at 0%.

### Fixed
- Mouse-wheel scroll dead in every rmux pane (regression in rmux `0e4abb2`):
  the client-side paste heuristic wrapped a batched wheel burst (3+ SGR mouse
  reports ≥ 32 bytes, no newline) as a bracketed paste, so the server pasted
  the sequences into the PTY instead of decoding scroll. rmux pin bumped to
  `726d9e7`, which exempts ESC-initiated bursts from paste synthesis.
- TUI chat focus: session selection re-anchors by NAME across refreshes (the
  chat keystream can never silently retarget another session) and Esc in chat
  focus returns to the list — local, never forwarded to the agent.
- Per-project Telegram agent-bot units are resurrected at startup, closing
  the reinstall gap; album/caption-split Telegram fragments aggregate into
  ONE mission.

## [0.1.3] — 2026-06-05

### Added
- Full new-project pipeline in the product: vision → PRD → brand-identity
  (opt-in) → planner → build, with custom stack choice and a dedicated vision
  oracle; Claude Design import; optional Telegram step in the flow.
- Visible project actions in the TUI Agentic tab with a Telegram-parity
  3-tier delete menu (one Actions menu, line per action).
- Project folders are pre-trusted before every claude launch (no interactive
  trust dialog in dispatched sessions).

### Fixed
- Telegram photo/image messages are no longer silently dropped.
- Patrol stale-reap kill chain + lost gate-upgrade notification.
- `install.sh` reads `OMEGA_VERSION` from the workspace Cargo.toml (was
  hardcoded 0.1.0).

## [0.1.2] — 2026-06-05

### Added
- Guided first-run onboarding: `omega guide`, `~/.omega/GETTING-STARTED.md`,
  and an ordered install epilogue.
- R-PDF rule: all PDFs go through the OmegaOS pdfgen (single source of truth).
- Telegram: Projects → Import from GitHub workflow; provider API keys can be
  deleted from the Model menu.

### Fixed
- Workers died at login: `--bare` dropped from the Claude worker launch (bare
  mode skipped OAuth credential loading).
- Full bypass-permissions on every session + no reports in the Atlas topic
  (routing fix); oracles auto-close when finished (L4 gate-pending upgrade +
  deterministic patrol reap).
- Guard: no more false-blocking of files under `$HOME`; catastrophic alerts
  route through the canonical alert funnel to the Alerts topic.

## [0.1.1] — 2026-06-05

### Added
- Telegram hub maturity: free text routed to the AISB Master brain, one topic
  per project, a dedicated undeletable Alerts topic, designed Ω report cards
  with action buttons, live progress-bar card, conversation history +
  reply-to-message routing, voice input (Whisper), `/start` welcome + `/guide`,
  3-tier project delete, per-project dedicated-bot menu, morning briefing.
- Oracle engineering contract: git-sync, plan/100%, branch-per-worker,
  audit-on-code, L4 completeness gate (done_clean downgraded to pending if the
  plan isn't 100%), end-of-mission notifier (done.json → Telegram), and
  stuck-oracle alerts.
- `omega doctor --fix`, the self-heal daemon, token-budget alerts at
  80/85/90/95%, and a destructive-op audit tripwire (PreToolUse hook).
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
- TUI menu reorganized into Sessions · Menu · Agentic · Settings · Help, with
  the brain retired from the session menu (it lives in the Atlas Telegram
  topic).

### Fixed
- Atomic credential writes in the OAuth helper — no more 0-byte window — and
  fresh tokens are no longer discarded when healing the legacy symlink.
- `credentials` test that was flaky under parallel runs (it mutated the global
  `HOME`); the HOME-touching tests are now serialized.
- Dead code removed across the orchestration and TUI crates so the workspace
  builds with zero warnings.
- Oracle respawn no longer trusts a stale registry entry, and the patrol daemon
  re-checks session liveness before auto-marking a worker done.
- Dispatch: "Session ID already in use" — a persisted `--session-id` is never
  reused.

## [0.1.0] — 2026-06-03

Initial public cut. The `omega` CLI and TUI, the rmux-backed session model, the
typed doctrine (the 6 Laws and the named Rules) injected into every dispatched
agent, the oracle/worker orchestration layer, the Quality Arsenal audits, and
the optional Telegram bridge.
