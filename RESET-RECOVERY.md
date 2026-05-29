# RESET-RECOVERY — after a full VPS reset

OmegaOS reinstalls with one command:

```bash
git clone https://github.com/agentik-os/OmegaOS && cd OmegaOS && ./install.sh
```

That **reproduces everything OmegaOS needs to function**: `omega` + `rmux` binaries
(built from source), agent prompts (`~/.omega/agents`), the Laws+Rules registry
(`omega rules export` → `~/.omega/rules` → `~/.claude/rules` symlinks), slash commands
(`omega-*`, `/dynamic`, audit stubs), skills (`audits`, `pdfgen`), docs, the
**persistent Telegram systemd service**, the **tracking + verify hooks**, a **SOUL.md**
identity baseline, **native billing** (`omega usage --check`), and the `omega patrol`/`usage` crons.

It does **NOT** restore secrets or external plugins (by design — secrets never live in the repo).
Do these **manually after install**:

## 🔑 Secrets to re-add
| What | How |
|---|---|
| Telegram bot | `omega telegram setup <BOT_TOKEN> <CHAT_ID> --user-id <YOUR_USER_ID>` then `systemctl --user start omega-telegram` |
| Claude / Codex / Gemini auth | run `claude` (and `codex`/`gemini`) and log in — writes through to `~/.omega/credentials/` |
| Vercel tokens | recreate `~/.omega/config/vercel-tokens.json` → `{"teams":{"<orgId>":{"token":"…"}}}` |
| GitHub | `gh auth login` |
| Tella (if used) | `~/.omega/secrets/integrations.env` → `TELLA_API_KEY=…` (chmod 600) |

## 📦 External (reinstall separately — not part of OmegaOS core)
| What | How |
|---|---|
| claude-mem (memory system) | `/plugin marketplace add thedotmack/claude-mem` → `/plugin install claude-mem` |
| superpowers / stripe / frontend-design plugins | `/plugin marketplace add anthropics/claude-plugins-official` → `/plugin install …` |
| Personal global `~/.claude/CLAUDE.md` + agents | optional: re-clone `agentik-os/claude-config` into `~/.claude` |
| Personal `MEMORY.md` (user profile) | optional: re-clone agentik-monitor (kept out of the public repo for privacy) |
| Other projects (agentik-os-site, clients) | re-clone their repos; their crons live in those repos |

## ✅ Verify after install
```bash
omega rules list                       # 6 Laws + operational rules render
systemctl --user status omega-telegram # bridge service present (running once token set)
omega usage --check && cat ~/.omega/state/usage.json   # native billing
ls ~/.omega/hooks/ ~/.omega/SOUL.md    # hooks + identity present
./scripts/verify-install.sh            # INSTALL PARITY OK
```

## What is intentionally NOT restored (legacy, retired)
The `~/.aisb` bash/tmux orchestration layer (37 watchdog crons, dispatch-to-session, the
python aisb-bot) is **replaced** by omega/rmux + `omega patrol` + the systemd Telegram
service. A fresh install does not — and should not — recreate it.
