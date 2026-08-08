# OmegaOS — Getting Started

> Re-read this anytime: `omega guide` (or `cat ~/.omega/GETTING-STARTED.md`).
> Everything below is also checkable with one command: `omega doctor`.

The installer just set up the full stack: the `rmux` session daemon, the `omega`
CLI + TUI, 7 self-healing cron jobs, the quality arsenal (23 audits), and the
Telegram command bot service (dormant until you give it a token). **Only the
personal pieces are left — they take ~5 minutes.**

---

## Step 1: Connect OpenAI Codex (default runtime)

New OmegaOS and Oracle sessions use Codex by default. Link it once:

```
codex login
codex login status
```

On a headless VPS, use the displayed device flow. A successful status prints
the active login method. Claude Code remains available as an explicit provider:
run `claude`, enter `/login`, then select Claude per session or mission.

## Step 2 — Telegram remote control (recommended)

Drive everything from your phone: dispatch missions, get reports, briefings
and alerts.

1. **Create a bot**: message [@BotFather](https://t.me/BotFather) → `/newbot`
   → copy the token (`123456:ABC…`).
2. **Get your user id**: message [@userinfobot](https://t.me/userinfobot) —
   it replies with your numeric id.
3. **Wire it** (the bot only ever answers YOU — allow-list enforced):
   ```
   OMEGA_TG_TOKEN=<BOT_TOKEN> omega telegram setup <YOUR_USER_ID> --user-id <YOUR_USER_ID>
   ```
   (the env prefix keeps the token out of the process list and shell history)
4. *(optional but the full experience)* **Project hub with one topic per
   project**: create a Telegram group → enable **Topics** in its settings →
   add your bot as **admin** with **Manage Topics** → type `/setupgroup` in
   the group, then `/sync`. You get: one topic per project (message it =
   dispatch a mission), an **Atlas** topic (talk to the orchestrator), and an
   **Alerts** topic (operational alerts; auto-recreated if deleted).

DM the bot `/start` to see the full button menu.

### Per-project agent bots (optional)

Beyond the single command bot, each project can get its **own** Telegram bot —
a dedicated chat where that project's agent answers directly. To register one:
create a bot with [@BotFather](https://t.me/BotFather), then open the command
bot's **project menu** (Projects → your project → dedicated bot) and paste the
token. OmegaOS stores it in `~/.omega/agent-bots.json` and runs the bot as a
per-project service (`omega-tg-agent-<project>.service`), resurrected
automatically at startup.

`~/.omega/agent-bots.json` and the tokens inside it are **local secret state**:
recreated on each machine, never committed to any repo (R-TGSEC — each bot
answers only its allow-listed users).

### The Librarian bot (Alexandria) + its open-source media stack

From the command bot's **Menu → 🤖 Agents → 📚 Link your librarian (Alexandria)**,
paste a @BotFather token and you get a personal librarian / learning engine on its
own Telegram bot (English by default, `/setup` to calibrate, `/best`, `/book`,
`/chapter`, `/challenge`, …). It uses **only open-source, on-device media tooling**,
all installed by `./install.sh`:

- **Voice in (transcription)** — `omega-transcribe` (faster-whisper, `tools/transcription/`).
  On-device, no API key, 99 languages. The bot transcribes voice notes and audio files
  with it first; the OpenAI Whisper API is only a fallback if you set `OPENAI_API_KEY`.
- **Voice out (TTS)** — the `omega-ttsd` gateway (Piper/Kokoro, `tools/tts/`). `/voice on`
  makes the librarian also answer as a spoken note, rendered locally.
- **Clean diagrams** — `beautiful-mermaid` (MIT), installed into `~/.omega/telegram-bot`
  by `bun install` during setup. It renders Mermaid to a self-contained SVG offline, so the
  librarian delivers crisp diagram files (a PNG preview + a full-screen HTML) instead of
  ASCII that breaks in chat.

None of these need a paid key. If a piece is missing on your box the bot degrades
gracefully (text instead of voice, the diagram skill instead of beautiful-mermaid).

## Step 3 — Service keys for auto-provisioning (optional)

`/omg-new-project` can provision Vercel + Convex + GitHub + Clerk + Stripe for
a new app automatically. Give it the keys once:

```
$EDITOR ~/.omega/provisioning/services.env
```

```bash
export VERCEL_TOKEN=""        # vercel.com → Settings → Tokens
export GITHUB_TOKEN=""        # github.com → Settings → Developer settings → PAT
export CONVEX_TEAM_TOKEN=""   # dashboard.convex.dev → Team settings
export STRIPE_SECRET_KEY=""   # dashboard.stripe.com → Developers → API keys
export OPENAI_API_KEY=""      # platform.openai.com — used for Telegram VOICE messages (Whisper)
```

All optional — leave blank what you don't use. Secrets live in `~/.omega/`
only (gitignored, chmod 600); never in a repo.

## Step 4 — Add your projects

Pick any of:

- **TUI**: `omega` → Projects → **[N] New Project** (guided: vision → PRD →
  plan → build) or add an existing folder.
- **Telegram**: Projects menu → **Import from GitHub** (clones + wires
  dashboard agent + topic + `/project` command in one go).
- **CLI**: drop a repo under `~/Station/<Category>/<Name>` — it's
  auto-discovered; `omega dispatch <Name> "<mission>"` just works.

## Step 5 — Verify, then fly

```
omega doctor     # full-stack health check — everything should be [+]
omega           # the TUI: sessions, monitor, settings, audits
omega dispatch <Project> "fix the signup flow"   # your first mission
omega audit list # the 23-audit quality arsenal
```

Daily drivers: `omega` (TUI) · the Telegram bot · `omega master` (AISB chat) ·
`omega attach -t <session>` (jump into any live agent).

---

## What runs by itself (no action needed)

| Thing | How |
|---|---|
| Telegram bots | systemd user services, enabled + linger → survive reboots |
| Credential maintenance | reconciles provider-native credentials into OmegaOS state without printing secrets |
| Patrol | cron 1 min — restarts the rmux daemon, resurrects crashed oracles |
| Self-heal | cron 3 h — `omega doctor --fix` automatically |
| Daily briefing | cron 08:00 — portfolio health digest in the Atlas topic |
| Stuck-oracle alerts | cron 1 min — pings the Alerts topic if an oracle stalls |

## Optional extras

- **Mission Control dashboard** (web UI, one container per agent — needs
  Docker): `omega-mc-up`, then open `http://<host>:8080`.
- **More CLI agents**: `omega install claude|gemini|pi|hermes|glm` (or
  Settings → Install agents in the TUI). All install user-space, no root.
- **Global keybindings**: `omega install-bindings` (Ctrl+Space popup).
- **Themes**: a gallery of TUI palettes with live preview (Settings → Theme) —
  the full list, slugs, and Termius guidance live in [docs/THEMES.md](THEMES.md).
- **Remote use from a laptop**: `mosh user@host -- omega` (installed; instant
  typing over any latency). In Termius: enable the mosh toggle.

Stuck? `omega doctor` first — it names the broken piece and `omega doctor
--fix` repairs the mechanical ones.
