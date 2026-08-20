# OS Builder {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, its ordered steps, and the completion test
that decides whether it is done.

| Workflow | Trigger | Produces |
|---|---|---|
| [FULL_BUILD.md](FULL_BUILD.md) | a capability request that clears the viability tree, and no unit exists for it | a registered, authored, scored, released unit that passes `verify.py --full` |
| [FAST_BUILD.md](FAST_BUILD.md) | the same request, but the capability is low risk, narrow and already understood | a wave 1 unit that passes `verify.py` on the five core files, honestly labelled incomplete |
| [REPAIR_AN_EXISTING_OS.md](REPAIR_AN_EXISTING_OS.md) | a unit already on disk fails a gate, drifted from the contract, or was scaffolded and never authored | the same slug, repaired in place, passing the tier it claims |

A workflow is finished when its completion test passes, not when its last step
has been performed.

## Choosing between them

Read the viability tree first (`PROMPTS/00-intake.md`, step 4). It answers
whether to build anything at all. Only once the answer is BUILD does the choice
below apply.

| Condition | Path |
|---|---|
| The capability touches money, legal rights, health, employment, compliance, production systems or regulated data | FULL, always. Fast is not available for these. |
| The capability is net new to the suite and nobody can state its evidence base yet | FULL. Research is the phase that gets skipped, and it is the phase that decides whether the unit is true. |
| The unit will be depended on by another unit (it appears in someone's `requires`) | FULL. A dependency that fails a gate fails everyone downstream. |
| The capability is narrow, low risk, well understood, and the operator wants it usable today | FAST, then FULL later. Fast never releases. |
| A slug already exists on disk | REPAIR. Never scaffold over an authored unit. |

The honest default is FULL. FAST exists so that a small capability is not
strangled by ceremony, not so that a large one can skip its gates.

## What every workflow inherits

Three rules bind all three paths, and no phase may waive them.

1. **The contract is machine checked, never asserted.** A unit is complete when
   `python3 OS/_tools/verify.py <slug>` exits zero, not when the build says so.
   The 23 files in `verify.py: CONTRACT_FILES` are the contract; the five in
   `CORE_FILES` are wave 1.
2. **Registration is upstream of authoring.** A slug that is not in the `SUITE`
   tuple of `OS/_tools/suite.py` does not exist, whatever is on disk. Generated
   files (`OS/_registry.json`, `OS/README.md`,
   `crates/omega-core/src/os_products.rs`) are emitted from it and are never
   hand edited.
3. **No placeholder survives a gate.** Any file still carrying the scaffold
   marker comment that `scaffold.py` writes fails `AUTHORED`, and the string
   "to be authored" in `OS.md`, `SKILL.md` or `COMMANDS/README.md` fails
   `SUBSTANCE`. Both are checked, so neither is a matter of taste.
