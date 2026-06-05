# OmegaOS — Getting Started

> Re-read this anytime: `omega guide` (or `cat ~/.omega/GETTING-STARTED.md`).
> Everything below is also checkable with one command: `omega doctor`.

The installer just set up the full stack: the `rmux` session daemon, the `omega`
CLI + TUI, 7 self-healing cron jobs, the quality arsenal (23 audits), and the
Telegram command bot service (dormant until you give it a token). **Only the
personal pieces are left — they take ~5 minutes.**

---

## Step 1 — Connect your Claude account (required)

OmegaOS drives [Claude Code](https://claude.com/claude-code). Link your account:

```
claude          # opens Claude Code …
/login          # … then type /login and follow the URL it prints
```

On a headless VPS: open the printed URL on your laptop, sign in, paste the code
back in the terminal. Verify: `claude auth status` → `"loggedIn": true`.
OmegaOS then shares this credential with every agent it spawns, refreshes the
token proactively (cron, every 30 min), and self-heals expiries.

## Step 2 — Telegram remote control (recommended)

Drive everything from your phone: dispatch missions, get reports, briefings
and alerts.

1. **Create a bot**: message [@BotFather](https://t.me/BotFather) → `/newbot`
   → copy the token (`123456:ABC…`).
2. **Get your user id**: message [@userinfobot](https://t.me/userinfobot) —
   it replies with your numeric id.
3. **Wire it** (the bot only ever answers YOU — allow-list enforced):
   ```
   omega telegram setup <BOT_TOKEN> <YOUR_USER_ID> --user-id <YOUR_USER_ID>
   ```
4. *(optional but the full experience)* **Project hub with one topic per
   project**: create a Telegram group → enable **Topics** in its settings →
   add your bot as **admin** with **Manage Topics** → type `/setupgroup` in
   the group, then `/sync`. You get: one topic per project (message it =
   dispatch a mission), an **Atlas** topic (talk to the orchestrator), and an
   **Alerts** topic (operational alerts; auto-recreated if deleted).

DM the bot `/start` to see the full button menu.

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
| Token refresh | cron 30 min — keeps Claude OAuth fresh, syncs Mission Control |
| Patrol | cron 1 min — restarts the rmux daemon, resurrects crashed oracles |
| Self-heal | cron 3 h — `omega doctor --fix` automatically |
| Daily briefing | cron 08:00 — portfolio health digest in the Atlas topic |
| Stuck-oracle alerts | cron 1 min — pings the Alerts topic if an oracle stalls |

## Optional extras

- **Mission Control dashboard** (web UI, one container per agent — needs
  Docker): `omega-mc-up`, then open `http://<host>:8080`.
- **More CLI agents**: `omega install codex|gemini|pi|hermes|glm` (or
  Settings → Install agents in the TUI). All install user-space, no root.
- **Global keybindings**: `omega install-bindings` (Ctrl+Space popup).
- **Themes**: 15 TUI palettes with live preview (Settings → Theme) — the full
  gallery, slugs, and Termius guidance live in [docs/THEMES.md](THEMES.md).
- **Remote use from a laptop**: `mosh user@host -- omega` (installed; instant
  typing over any latency). In Termius: enable the mosh toggle.

Stuck? `omega doctor` first — it names the broken piece and `omega doctor
--fix` repairs the mechanical ones.
