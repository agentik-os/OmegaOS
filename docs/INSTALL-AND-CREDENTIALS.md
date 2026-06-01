# OmegaOS — Install Flow & Credentials (Single Source of Truth)

> Explains the difference between the GitHub repo, the installer, and the
> runtime directory. Plus the complete credentials/OAuth/email system.

## Part 1 — GitHub vs Installer vs Runtime

You were confused about "GitHub = install part, installer = outside GitHub".
Here's the truth:

### The installer IS in the GitHub repo

`install.sh` lives at the root of the GitHub repo. There is no separate
installer "outside GitHub". The flow is:

```
github.com/agentik-os/OmegaOS
        │
        │  (1) user clones OR curls install.sh
        ▼
┌──────────────────────────────────────────────────┐
│ install.sh (in the repo)                          │
│   reads the repo's source files                   │
│   compiles the Rust code                           │
│   creates ~/.omega/ on the user's machine         │
└──────────────────────────────────────────────────┘
        │
        ├──→ ~/.local/bin/omega        (compiled binary)
        └──→ ~/.omega/                  (runtime data, created fresh)
```

### Two ways a user installs

**Method A — one-liner (most common):**
```bash
curl -sSL https://raw.githubusercontent.com/agentik-os/OmegaOS/main/install.sh | bash
```
This downloads ONLY install.sh, which then `git clone`s the full repo to
`/tmp/omega-build`, compiles, installs, and cleans up.

**Method B — clone first:**
```bash
git clone https://github.com/agentik-os/OmegaOS
cd OmegaOS
./install.sh
```
install.sh detects it's running inside the repo (sees Cargo.toml) and
builds from the current directory.

### What install.sh creates on the user's machine

```
~/.local/bin/omega           ← the compiled Rust binary (8.9 MB)
~/.local/bin/rmux            ← the terminal multiplexer
~/.omega/                    ← the runtime master directory:
  ├── OMEGA.md               (copied from repo)
  ├── config.toml            (from repo's config/default.toml)
  ├── providers.toml         (provider catalog)
  ├── rules/                 (15 rules, via `omega rules export`)
  ├── agents/                (19 agent prompts, copied)
  ├── skills/pdfgen/         (PDF generator, rsynced)
  ├── skills/audits/         (23 audits, copied)
  ├── credentials/           (created empty, populated on first login)
  └── state/ logs/ locks/    (runtime dirs)
```

### Why the architecture rules ARE in place after install

The architecture (centralized config, symlinks, multi-provider) is enforced
by install.sh Phase 5:

```bash
# Phase 5a: Credential Migration (migrate_creds function)
migrate_creds "claude" "$HOME/.claude/.credentials.json"
migrate_creds "codex"  "$HOME/.codex/auth.json"
migrate_creds "gemini" "$HOME/.config/gemini/oauth_creds.json"
# → moves existing creds to ~/.omega/credentials/, creates symlinks back

# Then:
omega rules export    # writes 15 rules to ~/.omega/rules/
omega sync            # symlinks rules into ~/.claude/rules/omega-*.md
                      # appends @import to ~/.gemini/GEMINI.md
                      # symlinks ~/.codex/AGENTS.md → OMEGA.md
```

So when a fresh user installs, the architecture is auto-applied. They don't
have to do anything — `omega sync` runs at install time.

### The 3 locations, restated

| Location | What | Versioned? | Secret? |
|----------|------|-----------|---------|
| `~/VibeCoding/work/OmegaOS/` (the repo) | Source code + install.sh | Yes (git) | No |
| `~/.local/bin/omega` | Compiled binary | No (built locally) | No |
| `~/.omega/` | Runtime: creds, rules, agents, state | No (gitignored) | Yes (credentials) |

The repo is immutable + public. `~/.omega/` is the user's living, private state.

## Part 2 — Credentials System (SSOT)

### Where credentials live

```
~/.omega/credentials/
├── claude.json         OAuth tokens (accessToken, refreshToken, expiresAt)
├── codex.json          OpenAI API key
├── gemini.json         Google OAuth
├── glm.json            Z.AI key
├── openrouter.json     OpenRouter key
└── accounts/           Saved profiles for account switching
    ├── claude-gareth.json
    └── claude-work.json
```

### The symlink dance (and a gotcha)

Each LLM CLI expects its creds at a native path. OmegaOS symlinks them:

```
~/.claude/.credentials.json  → ~/.omega/credentials/claude.json
~/.codex/auth.json           → ~/.omega/credentials/codex.json
```

**GOTCHA:** Claude's `/login` does an **atomic write** (write to .tmp +
rename). The `rename()` REPLACES the symlink with a real file. So after a
login, `~/.claude/.credentials.json` is a fresh real file and the omega
copy is stale.

**Fix (in oauth.rs):**
1. `claude_native_path()` returns `~/.claude/.credentials.json` — the path
   Claude actually writes to.
2. `handle_code()` watches THAT path for the mtime change (not the omega path).
3. After login success, `sync_credentials_to_omega()` copies the fresh file
   to `~/.omega/credentials/claude.json` and re-creates the symlink.

### The OAuth login flow (step by step)

```
1. User: /account → taps [Login] button
2. oauth::request_reauth():
   - spawns rmux session "aisb-reauth" running `claude --dangerously-skip-permissions`
   - sends /login
   - captures the OAuth URL from the pane
3. Bot sends the URL to Telegram
4. User clicks URL → authorizes on claude.com → copies the code
5. User pastes the code into Telegram
6. oauth::handle_code():
   - detects the code (looks_like_oauth_code: 20+ chars [A-Za-z0-9_-])
   - pastes code into aisb-reauth (paste → sleep 1s → Enter)
   - polls ~/.claude/.credentials.json for mtime change (up to 20s)
   - DETECTS "Login successful. Press Enter to continue" → auto-sends Enter
   - on success: sync_credentials_to_omega() + reads email
7. Bot replies: "Authenticated\nEmail: user@example.com\nExpires: N min"
```

### The email gotcha

**credentials.json does NOT contain an email.** It only has:
accessToken, refreshToken, expiresAt, scopes, subscriptionType, rateLimitTier.

The email lives ONLY in `claude auth status` JSON output:
```json
{"loggedIn": true, "email": "user@example.com", "subscriptionType": "max", ...}
```

So `account::email_from_claude_auth_status()` runs `claude auth status`,
parses the JSON, and extracts the email. That's why the account card now
shows the real email instead of "unknown".

### State files

| File | Purpose |
|------|---------|
| `/tmp/omega-pending-reauth.json` | Pending login flag (prevents double-trigger), 30s cooldown |
| `~/.omega/state/telegram-active-model.json` | Per-chat active provider+model |
| `~/.omega/credentials/accounts/*.json` | Saved account profiles |

### Auto-detection of auth failures

`oauth::detect_auth_failure()` scans oracle pane output for markers:
`401`, `Unauthorized`, `rate_limit_error`, `Rate limit reached`,
`Please run /login`, `Invalid bearer token`, `Token expired`.

When detected, the bridge can auto-trigger the reauth flow.

## Part 3 — Telegram Commands (current)

| Command | What | Buttons |
|---------|------|---------|
| /help | List commands | — |
| /account | Account card | [Login] [Logout] [Billing] [Switch] |
| /model | Switch provider/model | Provider buttons → model buttons |
| /projects | Project list | per-project + [+ New] [Scan & add existing] |
| /sessions | Active sessions | per-session (tap to target) |

Plain text (no slash) → relayed to AISB Master automatically.
Reply to an oracle report → routed back to that project's oracle.
/cancel → clears any session/project target.
