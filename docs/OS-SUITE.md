# The AgentikOS OS Suite - integration playbook

This document is the standard for EVERY operative system (OS) of the AgentikOS
suite: what an integrated OS looks like, the three commands it must expose, and
the exact process when the operator says "add this OS" or "complete this OS".

The suite lives in `OS/` (installed to `~/.omega/os/`), is surfaced in the TUI
**OS** tab, and its registry is compiled into
`crates/omega-core/src/os_products.rs` (`OsProduct::all()` - the single source
of truth for names, slugs, taglines and order).

## The suite

The BUILD CHAIN is its own group, in pipeline order:
`01 Ideation -> 02 Researcher -> 03 Blueprint -> 04 Designer (UX/UI) -> 05 Stepper -> 06 Builder`.

| # | OS | Slug | Focus | Status |
|---|----|------|-------|--------|
| 01 | Ideation OS | `ideation-os` | Brainstorm {OS} v3: multi-agent imagination + decision council, lineage, Surface Lab, frozen concept handoff | **integrated** |
| 02 | Researcher OS | `researcher-os` | Market Research {OS}: evidence + validation compiler, bounded GO/PIVOT decisions, frozen Blueprint input manifest | **integrated** |
| 03 | Blueprint OS | `blueprint-os` | The product-definition compiler (v3): 38 sections, stable IDs, 20 gates, frozen handoff | **integrated** |
| 04 | Designer OS (UX/UI) | `designer-os` | Contracts turned into screens, flows and a design system | awaiting drop |
| 05 | Stepper OS | `stepper-os` | Step-by-step execution of a blueprint | **integrated** |
| 06 | Builder OS | `builder-os` | The implementation runtime: the Stepper roadmap executed into release-ready code | **integrated** |

Personal OSes:

| OS | Slug | Focus | Status |
|----|------|-------|--------|
| Mindset OS | `mindset-os` | Jim Rohn identity/wellbeing/wealth OS: evidence-labeled coaching + 90-day program | **integrated** |
| Habits OS | `habits-os` | Habit design, tracking, consistency | awaiting drop |
| Books OS | `books-os` | Your library as an OS: reading, retention, living knowledge | **integrated** |

Status is derived from the filesystem (TUI + `os_products::dir_status`): a
directory holding only its scaffold (placeholder README, `MASTER.md`,
`ledger/`) is `awaiting drop`; a real payload makes it `integrated`.

## Anatomy of an integrated OS

```text
OS/<slug>/
├── README.md                    what it is, layout, how to run it, honest
│                                divergences from the pack spec
├── MASTER.md                    the OS's MASTER AGENT system prompt (every
│                                OS has one, even pre-integration)
├── pack/                        the operator-provided spec documents, verbatim
├── engine/                      the runnable implementation (when the OS has one)
├── bin/omega-<name>             the OmegaOS command (thin launcher)
├── commands/codex-<slug>.md     the OpenAI/Codex slash command
└── ledger/                      runtime-only: the master agent / Telegram
                                 bot's persistent memory (never committed)
```

Plus, outside `OS/`:

- `skills/<slug>/SKILL.md` - the Claude command (skill + `/<slug>` and
  `/omg-<slug>` stubs). When a canonical skill already covers the OS (Books
  OS -> alexandria), the stubs point at the canon - never fork it.
- An `install.sh` block keeping Law 0 parity (see below).

## The master agent

`MASTER.md` is the OS's one brain: the system prompt of the agent that runs
the OS. It is the SAME prompt on every surface - the TUI Enter session, the
Telegram bot, and whatever the CLI launches. Pre-integration OSes ship a
pre-integration master (explain the OS, collect operator intent in
`ledger/INTENT.md`, guide the drop); integrated OSes ship the real operating
brain (Stepper: drive the execution protocol; Books: the full librarian -
whose canon lives in `agents/librarian.md`, MASTER.md bootstraps to it).

## The four-surfaces convention

Every OS exposes the SAME capability on four surfaces:

1. **Claude** - a skill in `skills/<slug>/SKILL.md`. Installed to
   `~/.omega/skills/<slug>/` (then `omega sync` symlinks it into
   `~/.claude/skills/`), plus `/​<slug>` and `/omg-<slug>` stubs in
   `~/.claude/commands/`. The skill TEACHES the loop and points at the CLI.
2. **OpenAI / Codex** - a flat markdown prompt in
   `OS/<slug>/commands/codex-<slug>.md`, installed by install.sh to
   `~/.codex/prompts/<slug>.md` (Codex custom slash command). Same protocol,
   condensed.
3. **OmegaOS** - a `bin/omega-<name>` wrapper symlinked into `~/.local/bin`.
   Heavy runtimes (Python venv, node_modules) are a LAZY first-run opt-in:
   install.sh never pip-installs (R-ENV boundary, like pixelrag/browser-use).
   The TUI OS tab's Enter opens a Claude session running the MASTER AGENT.
4. **Telegram** - `omega-os-bot <slug> [token]` (TUI: OS tab -> `T`) links a
   dedicated bot per OS: an `agent-bots.json` entry of kind `persona` whose
   system prompt is the OS's master agent and whose working dir is the
   installed OS folder (`ledger/` persists). Operator-only (allow-list,
   R-TGSEC); the token lives in agent-bots.json (mode 600) and nowhere else.

All four surfaces drive ONE brain. Never fork the logic per surface.

## Process - "add this OS" (a zip landed in Deposit)

1. **Locate + glance.** The zip is in `~/Deposit/` (Telegram DEPOSIT bot).
   Unpack to scratch first and READ what it runs before executing anything
   (R-REPO-INSTALL: one safety glance - install scripts, manifests, anything
   curl|sh or obfuscated). Clean -> proceed.
2. **Vendor the pack.** Copy the spec documents verbatim into
   `OS/<slug>/pack/`. The pack is the operator's canon: never edit it, write
   divergences in the OS README instead.
3. **Build the runtime** the pack describes (`engine/` or equivalent), with
   tests, honoring the pack's non-negotiables. Prove it end to end at runtime
   (L1): init -> loop -> terminal gate, captured output.
4. **Write the real MASTER.md** (replacing the pre-integration one) and wire
   the four surfaces (convention above).
5. **Parity (Law 0).** install.sh: the generic `OS/` copy block already ships
   payloads + bin wrappers + codex commands; add a skill-stub block for the
   Claude command (see the Stepper OS block as the template). Run
   `./scripts/verify-install.sh`.
6. **Update the docs**: the OS README (status, layout, run), this file's
   status table, and the root `README.md` suite table if it changed shape.
7. **Verify the TUI** shows the OS 🟢 integrated (rmux capture of the OS tab).
8. **Commit + push** OmegaOS; publish the skill to the Agentik-Skills library
   (R-SKILLPUB - both SSOTs).

## Process - "complete this OS" (payload exists, extend it)

1. Read `OS/<slug>/README.md` (current state + declared divergences) and the
   pack docs the change touches.
2. Extend the engine + tests; the pack stays untouched (new operator specs go
   to `pack/` as new files, versioned).
3. Update the README's divergences section - it must stay honest.
4. Re-run the engine test suite + the runtime smoke, keep install.sh parity,
   push, republish the skill if it changed.

## Future OS notes (what each will need at integration time)

- **Mindset OS** - INTEGRATED (Jim Rohn Extended v2): an evidence-aware
  identity/wellbeing/wealth coaching OS (prompt-pack + two stdlib workspace
  scripts, `omega-mindset new`/`score`). safety.md is always honored.
- **Habits OS** - likely a personal-OS runtime (LifeStyle lane); expect a
  Convex/Next.js app or prompt-pack payload. If it ships as an APP, the
  `engine/` slot holds it and `bin/omega-<name>` launches dev/deploy.
- **Ideation OS** - INTEGRATED (Brainstorm {OS} v3 ULTIMATE): a multi-agent
  imagination + decision council with lineage, the Surface Lab, quality gates
  and a frozen concept handoff to Market Research / Blueprint. The skill is
  brainstorm-os; the superpowers brainstorming skill remains a lighter live
  helper, not a fork of this OS.
- **Researcher OS** - INTEGRATED (Market Research {OS} v1.0.0): evidence
  compiler with depths SIGNAL/VALIDATION/INVESTMENT_GRADE and a frozen
  Blueprint input manifest; the marketing suite's market-research skill is a
  source lane inside it, never a replacement.
- **Designer OS (UX/UI)** - the design router (R-DESIGN, 130+ skills) and
  Open Design are the live surfaces; the payload will organize them into the
  chain, never duplicate them.
- **Blueprint OS** - INTEGRATED (v3, the definition compiler). The v1
  14-phase designer is archived at `skills/blueprint-os/legacy/`; its scripts
  (blueprint-check.sh, stax_derive, runner) stay live for the `/stack` chain
  (R-BLUEPRINT-STACK). Extend the v3 pack, never fork the surface.
- **Builder OS** - INTEGRATED (v1, the implementation runtime): consumes the
  frozen Blueprint handoff, executes the Stepper roadmap, gates BG01-BG20,
  finalize = frozen engineering/operations handoff.
- **Books OS** - already integrated as the wrapper around the canonical
  librarian system (`agents/librarian.md` + the alexandria skill); future
  payloads extend that canon, never fork it.

An OS is DONE when: pack vendored, runtime tested + proven live, MASTER.md
real, four surfaces wired, install parity green, TUI 🟢, docs updated,
pushed.
