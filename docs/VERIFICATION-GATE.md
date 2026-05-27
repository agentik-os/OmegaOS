# OmegaOS — Verification Gate Protocol

> "Code lies. Only runtime tells the truth." — Rule L1
> Every oracle's output passes through this gate. No exceptions.

## Gate 1: Build (automated, mandatory)

```bash
cargo build --release 2>&1  # 0 errors = PASS
cargo test 2>&1             # all tests pass = PASS
```

If ANY oracle's code breaks the build → revert their changes, re-dispatch.

## Gate 2: Module Coherence (manual review)

For each new .rs file, verify:
- [ ] Types match across module boundaries (no orphan structs)
- [ ] All `pub fn` have at least 1 caller
- [ ] No duplicate functionality between oracles
- [ ] lib.rs properly declares all new modules

## Gate 3: Architecture Compliance

- [ ] All orchestration uses rmux SDK (NEVER tmux commands)
- [ ] All Telegram uses reqwest HTTP (NEVER python-telegram-bot)
- [ ] All state uses JSONL/JSON files (NEVER SQLite for now)
- [ ] All config reads from ~/.omega/ (NEVER hardcoded paths)
- [ ] No Python, no bash scripts in the Rust pipeline

## Gate 4: Rule Enforcement in Code

- [ ] R-19: rubric.rs generates criteria BEFORE worker dispatch
- [ ] R-21: gate.rs implements 3-lens verification
- [ ] R-30: gate.rs implements ≥12 Popper challenges
- [ ] R-14: ship.rs verifies deploy returns 200
- [ ] R-28: mission.rs tracks token budget
- [ ] R-35: verifier.rs rejects uncited claims
- [ ] L3: oracle_lifecycle.rs never waits for user input
- [ ] SCOPE-CLAIM: dispatch.rs rejects overlapping file claims

## Gate 5: E2E Flow Test

Simulate the full chain:
```
1. Create a test project via omega projects add
2. Send a message via Telegram bridge
3. Intent parser classifies it
4. Router dispatches to correct oracle
5. Oracle spawns worker with fresh context
6. Worker completes, writes done.json
7. Quality gate runs (rubric + multi-grader)
8. Ship pipeline (build + commit + push)
9. Report sent back via Telegram
10. Reply to report → auto-routes to same oracle
```

Each step must produce verifiable evidence (log line, file, pane capture).

## Gate 6: Regression Check

Compare before/after:
- All existing TUI features still work (scroll, tabs, Enter/Esc, Ctrl+L)
- Telegram bridge still sends/receives messages
- AISB Master still auto-restarts on crash
- PDF generator still works from any directory
- omega rules list/export/sync still function
- All 15 rules still render in Info tab

## Execution

After all 5 oracles write .done.json:
1. Merge all changes (resolve lib.rs conflicts)
2. Run Gate 1 (build + test)
3. Run Gate 2 (module coherence)
4. Run Gate 3 (architecture compliance)
5. Run Gate 4 (rule enforcement)
6. Fix any failures, re-run gates
7. Run Gate 5 (E2E)
8. Run Gate 6 (regression)
9. Only then: git push
