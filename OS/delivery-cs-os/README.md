# Delivery & Customer Success {OS}

<!-- agentik:scaffold -->

> Fulfil the promise, drive adoption, earn the renewal.

**Suite position:** `39` in **04 · GROW** (Turn value into distribution and revenue).

## What this OS is for

Delivery & Customer Success {OS} is one operative system of the AGENTIK {OS} suite. It is not a
prompt and not a document: it is a system with its own operating specification,
workflows, commands, memory rules, evaluations and adapters, installed and run
by the Agentik Runtime.

Use it when your problem is: fulfil the promise, drive adoption, earn the renewal.

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
agentik install delivery-cs-os          # install this OS
agentik configure delivery-cs-os        # answer the minimum setup questions
agentik run delivery-cs-os              # start using it
```

Inside OmegaOS it is also reachable from `omega menu`, **OS** tab, entry
`39. Delivery & Customer Success {OS}`.

## Structure

```
delivery-cs-os/
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
