# Provider compatibility

OmegaOS validates installed agent CLIs with `omega doctor`. Reinstall or update
an existing provider with:

```bash
omega install <provider> --force
```

## Supported contracts

| Provider | Minimum tested CLI | Omega launch contract |
|---|---:|---|
| Claude Code | 2.1.219 | interactive TTY, `--permission-mode auto` |
| Codex | 0.147.0 | `--approve-for-me`, hook-trust bypass, no conflicting `--sandbox` |
| Gemini CLI | 0.31.0 | `--prompt-interactive`, Enterprise/API-key accounts |
| Antigravity (`agy`) | 1.1.8 | native Google auth, prompt-interactive |
| Pi / OpenRouter | 0.84.3 | explicit provider/model and `--` prompt delimiter |
| Hermes | 0.20.0 | `hermes chat`; `-q` for a dispatched one-shot |
| Kimi Code | 0.38.0 | `--prompt` without the incompatible `--auto` flag |
| GLM | Claude Code 2.1.219 | Claude adapter pointed at Z.AI Anthropic endpoint |

Current catalog defaults are `gpt-5.6` for Codex, `auto` for Gemini CLI,
`glm-5.3` for direct GLM, and `anthropic/claude-opus-5` for OpenRouter-backed
Pi/Hermes sessions. Account-scoped products may resolve an alias to a different
eligible model; use the provider's own model-list command to verify the actual
selection.

Omega-managed Codex sessions default `codex.bypass_hook_trust = true` so
installer-managed hooks cannot block a detached pane. This applies to every
enabled Codex hook; set it to `false` if you keep third-party hooks that must be
reviewed interactively.

## Google migration

Google stopped serving Gemini CLI requests for free, AI Pro, and AI Ultra
individual accounts on June 18, 2026. Those users should install Antigravity:

```bash
omega install antigravity
agy                         # authenticate once
omega config activate antigravity
```

Gemini CLI remains supported for Gemini Code Assist Standard/Enterprise and
paid Gemini or Enterprise Agent Platform API keys. Gemini and Antigravity keep
OAuth credentials in their native hybrid/keyring stores; OmegaOS does not copy
or symlink those credentials.

## Selecting a provider

```bash
omega config activate codex gpt-5.6
omega config activate claude opus
omega config activate antigravity       # native account default
omega dispatch MyProject "mission" --agent hermes
```

The active global selection is mirrored in
`~/.omega/state/active-model.json`. A mission-level `--agent` override takes
precedence without changing the global default.

## Upgrade checks

After updating OmegaOS:

```bash
omega reconcile
omega doctor
omega doctor --deep
```

`doctor` checks the executable version before a detached pane can fail on an
obsolete flag. `--deep` additionally performs provider authentication probes
where supported.
