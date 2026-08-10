# OS — the AgentikOS operative-systems suite

This directory holds the AgentikOS product line of operative systems (OS) shipped
with OmegaOS. Each subdirectory is ONE operative system. The suite is surfaced in
the TUI under the **OS** tab (`omega menu`) and installed to `~/.omega/os/` by
`install.sh`.

## The suite (in order)

The BUILD CHAIN is its own group in the TUI, in pipeline order, then the
personal OSes.

| # | OS | Slug | Focus | Status |
|---|----|------|-------|--------|
| 01 | Ideation OS | `ideation-os` | Idea generation and capture | awaiting drop |
| 02 | Researcher OS | `researcher-os` | Market and user research: validate before blueprinting | awaiting drop |
| 03 | Blueprint OS | `blueprint-os` | The product-definition compiler (v3): 38 sections, stable IDs, 20 gates, frozen handoff | **integrated** |
| 04 | Designer OS (UX/UI) | `designer-os` | Contracts turned into screens, flows and a design system | awaiting drop |
| 05 | Stepper OS | `stepper-os` | Step-by-step execution of a blueprint | **integrated** |
| 06 | Builder OS | `builder-os` | The implementation runtime: the Stepper roadmap executed into release-ready code | **integrated** |

Personal OSes:

| OS | Slug | Focus | Status |
|----|------|-------|--------|
| Mindset OS | `mindset-os` | Mental models, mindset engineering | awaiting drop |
| Habits OS | `habits-os` | Habit design, tracking, consistency | awaiting drop |
| Books OS | `books-os` | Your library as an OS: reading, retention, living knowledge | **integrated** |

Registry source of truth: `crates/omega-core/src/os_products.rs`
(`OsProduct::all()`). The TUI tab, statuses and paths all derive from it -
add or reorder an OS THERE, never in the UI code. The full integration
playbook (anatomy of an OS, the surfaces convention, the add/complete
processes) is `docs/OS-SUITE.md`.

## Master agent + Telegram bot

Every OS carries a `MASTER.md` - its MASTER AGENT system prompt. The TUI's
Enter opens a Claude session running that agent, and `T` (or
`omega-os-bot <slug> [token]`) links a dedicated Telegram bot whose brain is
the SAME master agent (agent-bots.json kind `persona`, ledger under the OS
folder). One brain, every surface.

## Integration pipeline (how an OS lands here)

1. The operator drops the OS payload (zip) in the Deposit box
   (`~/Deposit`, via the Telegram DEPOSIT bot).
2. Unpack it into `OS/<slug>/` (this repo), next to the placeholder README.
3. Document how it runs in `OS/<slug>/README.md` (entrypoint, deps, config).
4. Keep `install.sh` parity (Law 0): the `OS/` payload is copied to
   `~/.omega/os/` on install - a fresh clone + install must reproduce it.
5. Commit + push.

An OS whose directory only contains its placeholder README is shown as
`awaiting drop` in the TUI; anything more marks it `integrated`.

Secrets never live here (R-ENV): keys go to `~/.omega/secrets/`, the payload
references them by name.
