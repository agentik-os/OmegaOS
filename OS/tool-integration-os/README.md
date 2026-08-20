# Tool & Integration {OS}

<!-- agentik:scaffold -->

> Connect external tools safely, with typed contracts.

**Suite position:** `70` in **08 · AI & SYSTEMS** (Intelligence infrastructure for everything).

## What this OS is for

Tool & Integration {OS} is one operative system of the AGENTIK {OS} suite. It is not a
prompt and not a document: it is a system with its own operating specification,
workflows, commands, memory rules, evaluations and adapters, installed and run
by the Agentik Runtime.

Use it when your problem is: connect external tools safely, with typed contracts.

## Start here

| You want to | Read |
|---|---|
| Understand what it does | this file |
| Understand how it operates | [`OS.md`](OS.md) |
| See the AI behaviour contract | [`SYSTEM.md`](SYSTEM.md) |
| See what it can do | [`SKILL.md`](SKILL.md) |
| Configure it for yourself | [`SETUP.md`](SETUP.md) |
| See every command | [`COMMANDS/`](COMMANDS/) |
| See it in use | [`EXAMPLES/`](EXAMPLES/) |

## Install and run

```bash
agentik install tool-integration-os          # install this OS
agentik configure tool-integration-os        # answer the minimum setup questions
agentik run tool-integration-os              # start using it
```

Inside OmegaOS it is also reachable from `omega menu`, **OS** tab, entry
`70. Tool & Integration {OS}`.

## Structure

```
tool-integration-os/
├── README.md      this file, the human entry point
├── OS.md          the complete operating specification
├── SYSTEM.md      AI and system instructions
├── SKILL.md       capabilities and procedures
├── SETUP.md       initial configuration
├── manifest.json  machine-readable metadata
├── CHANGELOG.md   what changed between versions
├── WORKFLOWS/     repeatable processes
├── COMMANDS/      every command, explained
├── PROMPTS/       reusable prompt units
├── REFERENCES/    knowledge this OS needs
├── MEMORY/        what may be remembered, updated, forgotten
├── TOOLS/         external capabilities it may use
├── EVALS/         tests that prove it behaves correctly
├── EXAMPLES/      worked examples
├── INTERFACES/    chat, artifact, dashboard, generative UI
└── ADAPTERS/      ChatGPT, Claude, Gemini, Codex
```

## Status

Scaffolded against the AGENTIK {OS} contract. Sections still carrying the
scaffold marker are structure, not authored content.
