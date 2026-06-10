# OmegaOS — Centralized Configuration Architecture v3

> **Scope:** authoritative for the `~/.omega/` centralized runtime layout
> (credentials, models, settings). For the full system architecture
> (crates, orchestration, agent levels, CLI), see [ARCHITECTURE.md](ARCHITECTURE.md).
>
> Single source of truth for ALL LLM coding agents.
> `~/.omega/` is the master. Each LLM reads from it.
> Credentials, models, and settings live ONLY in ~/.omega/ — never elsewhere.

## Directory Structure

```
~/.omega/                              MASTER — all LLMs reference this
│
├── OMEGA.md                           Universal system prompt (every agent)
│
├── rules/                             The typed doctrine — 6 Laws + named Rules (.md files; `omega rules list` prints the set)
├── agents/                            14 Matrix agents + oracle/worker prompts
├── skills/                            Cross-LLM skills (pdfgen, audits, llm-council, browser-use, marketing pack, ...)
├── docs/                              Reference documentation
├── projects.json                      Project registry (paths, topics, oracle sessions)
│
├── config.toml                        General OmegaOS settings
├── providers.toml                     Per-provider config (model, base_url, default)
├── telegram.toml                      Telegram bridge (gitignored)
├── deposit.toml                       Deposit/inbox bot token (0600, gitignored)
├── agent-bots.json                    Per-project Telegram agent bots (tokens — local secret, never in the repo)
│
├── telegram-bot/                      Installed bot runtimes (omega-tg-bot.ts command bot, inbox-bot.ts deposit bot)
├── inbox/                             Operator deposit inbox — files/photos/notes sent from the phone (timestamped, indexed)
│
├── credentials/                       ALL provider credentials live HERE
│   ├── claude.json                    Claude OAuth tokens (was ~/.claude/.credentials.json)
│   ├── codex.json                     OpenAI/Codex API key
│   ├── gemini.json                    Google Gemini API key
│   ├── glm.json                       Z.AI/Zhipu key
│   ├── openrouter.json                OpenRouter key (multi-model)
│   ├── pi.json                        earendil/Pi config
│   ├── hermes.json                    Nous Research key
│   └── accounts/                      Multiple accounts per provider
│       ├── claude-gareth.json
│       ├── claude-work.json
│       └── ...
│
├── state/                             Runtime state (sessions, locks, done.json)
├── logs/                              Session logs
└── audit/                             Audit results
```

## Symlinks for LLM Compatibility

OmegaOS creates symlinks so existing LLM CLIs find their credentials in the
expected location, but the canonical files are ALWAYS in `~/.omega/credentials/`:

```
~/.claude/.credentials.json    → ~/.omega/credentials/claude.json
~/.codex/auth.json             → ~/.omega/credentials/codex.json
~/.config/gemini/oauth_creds.json → ~/.omega/credentials/gemini.json
```

This way:
- `omega telegram setup` writes only to `~/.omega/`
- `omega install <provider>` writes only to `~/.omega/`
- LLM CLIs still find their creds because of the symlinks
- One backup directory covers everything
- Account switching = updating the symlink target

## Provider Catalog

Stored in `~/.omega/providers.toml`:

```toml
default_provider = "claude"
default_model    = "opus"

[claude]
type      = "oauth"
cred_file = "credentials/claude.json"
models    = ["opus", "sonnet", "haiku"]
default_model = "opus"

[codex]
type      = "api_key"
cred_file = "credentials/codex.json"
models    = ["gpt-5", "gpt-5-codex", "o3"]
default_model = "gpt-5-codex"

[gemini]
type      = "oauth"
cred_file = "credentials/gemini.json"
models    = ["gemini-2.5-pro", "gemini-2.5-flash"]
default_model = "gemini-2.5-pro"

[glm]
type      = "api_key"
cred_file = "credentials/glm.json"
models    = ["glm-4.6", "glm-4.5"]

[openrouter]
type      = "api_key"
cred_file = "credentials/openrouter.json"
base_url  = "https://openrouter.ai/api/v1"
models    = ["anthropic/claude-sonnet-4.6", "openai/gpt-5", "google/gemini-2.5-pro"]

[pi]
type      = "config"
cred_file = "credentials/pi.json"
default_provider = "openrouter"
default_model    = "anthropic/claude-sonnet-4.6"

[hermes]
type      = "api_key"
cred_file = "credentials/hermes.json"
```

## Telegram Model Selection

Users can switch the active provider/model per session:

```
/model              → show current + list available
/model claude       → switch to Claude (uses default model)
/model claude opus  → switch to Claude with opus model
/model codex gpt-5  → switch to Codex with gpt-5
/model openrouter   → switch to OpenRouter (with its default model)
```

Each Telegram chat tracks its own `active_provider` and `active_model` —
stored in `~/.omega/state/telegram-active-model.json`.

## Account Management

```
/account              → show current account for each provider
/account claude       → list Claude accounts
/account claude gareth → switch to claude-gareth account
/account add claude work → save current Claude creds as "work" profile
```

Behind the scenes:
- Each account = a file in `~/.omega/credentials/accounts/`
- Switching updates the symlink: `~/.omega/credentials/claude.json → accounts/claude-gareth.json`
- Plus updates `~/.claude/.credentials.json` symlink (which points to omega)

## CLI Commands

```
omega rules list / export       Operational rules
omega sync                      Symlink omega config into all LLM dirs
omega accounts                  List all accounts across providers
omega accounts switch <provider> <name>
omega model show / set <provider> [model]
omega install <agent>           Install LLM CLI + auto-sync credentials
omega init                      Full setup: dirs + export + sync + symlinks
```

## Key Principles

1. **~/.omega/ is the master** — credentials, config, rules, agents, skills
2. **Symlinks for compatibility** — LLM CLIs see their expected paths, but real files are in omega
3. **Multi-provider first** — Claude / Codex / Gemini / GLM / OpenRouter / Pi / Hermes
4. **Multi-account per provider** — Switch between work/personal/client accounts
5. **Per-chat model selection** — Choose provider+model from Telegram on the fly
6. **No secrets outside omega** — credentials NEVER live in ~/.claude/, ~/.codex/, etc.
7. **install.sh handles symlinks** — moves existing creds into omega, creates the links

## Migration (existing users)

`omega init` (or first install) migrates existing credentials:

```bash
# For each detected provider:
if [ -f ~/.claude/.credentials.json ] && [ ! -L ~/.claude/.credentials.json ]; then
    mv ~/.claude/.credentials.json ~/.omega/credentials/claude.json
    ln -s ~/.omega/credentials/claude.json ~/.claude/.credentials.json
fi
# Same for Codex (~/.codex/auth.json), Gemini, etc.
```

Result: existing setups keep working, but all data is now centralized.
