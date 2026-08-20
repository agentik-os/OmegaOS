# Workflow: Resume after interruption

**Mode:** `RESUME`, then `PREFLIGHT`
**Produces:** reconciled Builder and Stepper state, and work continued from
evidence rather than from memory.

## Trigger

Any session that is not the first: a crash, a context compaction, a machine
restart, a handover to another person or another agent, or simply the next day.

## Preconditions

- The repository is reachable and its remote state can be inspected.
- Builder state and Stepper state exist on disk.

## Steps

1. **Read state before reading the transcript.** `omega-builder status` and
   `omega-stepper status`. What a session remembers about its own progress is
   the least reliable source available.
2. **Verify fingerprints.** Blueprint version and checksum, Stepper graph
   revision. Upstream may have cut a new version while this session was gone. A
   mismatch stops work before any edit.
3. **Reconcile Stepper.** `omega-stepper resume`. Interrupted RUNNING or
   VERIFYING attempts drop back to FAILED so the planner re-offers them.
4. **Inspect the working tree.** `git status`, worktrees, branches, locks.
   Uncommitted work is reconciled and understood, never reset and never
   destructively stashed. It is usually the only record of what the interrupted
   attempt actually did.
5. **Match tree to attempt.** For each open or recently failed attempt, decide:
   does the working tree contain part of that step's work, someone else's work,
   or something unrelated. Say which, out loud, in the session report.
6. **Reconcile Builder attempts.** Transition each open attempt to a legal
   state, recording why. An attempt left half open is counted as live by the
   ceiling and will distort the next repair loop.
7. **Re-read prior failure evidence.** Before re-claiming the interrupted step,
   read what already failed on it. This is the cheapest defence against
   repeating a dead end.
8. **Checkpoint.** `omega-builder checkpoint`, so the reconciliation itself
   survives the next interruption.
9. **Continue from the plan.** `omega-stepper plan`, and take the wave the
   planner offers. Not the step the transcript was talking about.

## Completion test

```bash
omega-builder validate <state.json>     # no open illegal attempt
omega-stepper status                    # no step stuck in RUNNING or VERIFYING
git status                              # every uncommitted change accounted for in the report
```

Passes when: fingerprints match or the mismatch has been reported and resolved,
no attempt is left in an interrupted state, every uncommitted change has been
attributed to a step or explicitly flagged as unattributed, and the next claim
comes from the planner rather than from the transcript.

## Failure paths

| What happens | What the workflow does |
|---|---|
| the working tree contains changes nobody can attribute | do not delete them; preserve them on a branch, report them, and ask before discarding |
| the Blueprint fingerprint moved | stop, read the delta, re-plan the affected steps before implementing anything |
| the Stepper graph moved under an in-flight step | abandon the attempt cleanly, re-read the new contract, re-claim |
| two sessions were running on the same files | serialise, integrate both with a real merge, and report the overlap as a scope failure |
| state files are corrupt or absent | rebuild from the append-only event log and the repository, and say plainly what could not be recovered |
