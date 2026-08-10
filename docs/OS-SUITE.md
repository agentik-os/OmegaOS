# The AgentikOS OS Suite - integration playbook

This document is the standard for EVERY operative system (OS) of the AgentikOS
suite: what an integrated OS looks like, the three commands it must expose, and
the exact process when the operator says "add this OS" or "complete this OS".

The suite lives in `OS/` (installed to `~/.omega/os/`), is surfaced in the TUI
**OS** tab, and its registry is compiled into
`crates/omega-core/src/os_products.rs` (`OsProduct::all()` - the single source
of truth for names, slugs, taglines and order).

## The suite

| # | OS | Slug | Focus | Status |
|---|----|------|-------|--------|
| 1 | Mindset OS | `mindset-os` | Mental models, mindset engineering | awaiting drop |
| 2 | Habits OS | `habits-os` | Habit design, tracking, consistency | awaiting drop |
| 3 | Brainstorm OS | `brainstorm-os` | Idea generation and capture | awaiting drop |
| 4 | Blueprint OS | `blueprint-os` | Product blueprints and design | awaiting drop (the `/blueprint-os` skill already covers the design flow) |
| 5 | Stepper OS | `stepper-os` | Step-by-step execution of a blueprint | **integrated** |
| 6 | Builder OS | `builder-os` | Building and shipping the product | awaiting drop |

Status is derived from the filesystem (TUI + `os_products::dir_status`): a
directory holding only its placeholder README is `awaiting drop`; anything more
is `integrated`.

## Anatomy of an integrated OS

```text
OS/<slug>/
├── README.md                    what it is, layout, how to run it, honest
│                                divergences from the pack spec
├── pack/                        the operator-provided spec documents, verbatim
├── engine/                      the runnable implementation (when the OS has one)
├── bin/omega-<name>             the OmegaOS command (thin launcher)
└── commands/codex-<slug>.md     the OpenAI/Codex slash command
```

Plus, outside `OS/`:

- `skills/<slug>/SKILL.md` - the Claude command (skill + `/<slug>` and
  `/omg-<slug>` stubs).
- An `install.sh` block keeping Law 0 parity (see below).

## The three-commands convention

Every OS that gains a runtime exposes the SAME capability on three surfaces:

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
   The TUI OS tab's Enter opens a Claude session scoped to the OS folder.

All three surfaces drive ONE engine. Never fork the logic per surface.

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
4. **Wire the three commands** (convention above).
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

- **Mindset OS / Habits OS** - likely personal-OS runtimes (LifeStyle lane);
  expect Convex/Next.js app payloads or prompt-pack payloads. If an OS ships
  as an APP, the `engine/` slot holds the app and `bin/omega-<name>` launches
  dev/deploy; the R-BLUEPRINT-STACK chain governs any new app build.
- **Brainstorm OS** - pairs with the existing brainstorming skill; the OS
  payload should absorb it as its Claude surface, not duplicate it.
- **Blueprint OS** - the `/blueprint-os` skill (14 phases, 3 gates) is already
  the design flow; the OS payload will vendor its pack and the two must not
  fork: the skill becomes the Claude command of the OS.
- **Builder OS** - downstream of Stepper: expect it to consume Stepper's
  release gate as its input contract.

An OS is DONE when: pack vendored, runtime tested + proven live, three
commands wired, install parity green, TUI 🟢, docs updated, pushed.
