# OS — the AgentikOS operative-systems suite

This directory holds the AgentikOS product line of operative systems (OS) shipped
with OmegaOS. Each subdirectory is ONE operative system. The suite is surfaced in
the TUI under the **OS** tab (`omega menu`) and installed to `~/.omega/os/` by
`install.sh`.

## The suite (in order)

| # | OS | Slug | Focus | Status |
|---|----|------|-------|--------|
| 1 | Mindset OS | `mindset-os` | Mental models, mindset engineering | awaiting drop |
| 2 | Habits OS | `habits-os` | Habit design, tracking, consistency | awaiting drop |
| 3 | Brainstorm OS | `brainstorm-os` | Idea generation and capture | awaiting drop |
| 4 | Blueprint OS | `blueprint-os` | Product blueprints and design | awaiting drop |
| 5 | Stepper OS | `stepper-os` | Step-by-step execution of a blueprint | **integrated** |
| 6 | Builder OS | `builder-os` | Building and shipping the product | awaiting drop |

Registry source of truth: `crates/omega-core/src/os_products.rs`
(`OsProduct::all()`). The TUI tab, statuses and paths all derive from it -
add or reorder an OS THERE, never in the UI code. The full integration
playbook (anatomy of an OS, the three-commands convention, the add/complete
processes) is `docs/OS-SUITE.md`.

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
