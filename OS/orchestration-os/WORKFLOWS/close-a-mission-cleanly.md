# Workflow: Close a mission cleanly

End a mission with an honest signal and nothing still running.

## Trigger

- Every ledger entry is done and verified.
- The mission must end without that being true.
- The requester stopped the mission.

## Steps

1. **Re-read the ledger from its file** and list every entry with its state. Do
   not close from a recollection of what was completed.
2. **Confirm independent verification per done entry.** The verification command,
   who ran it, and its output. An entry whose only evidence is a delegate's claim
   is not done, and it is moved back rather than argued about.
3. **Recompute the live worker set.** Every worker this mission started, whether
   or not the ledger mentions it. Missions leak workers, and a leaked worker
   keeps writing after the mission believes it ended.
4. **Refuse a clean signal while any worker is live and unfinished.** Wait for
   it, or close it deliberately, and record which.
5. **Release every scope claim,** including those held by workers that died. A
   claim that outlives its worker rejects the next legitimate step on the same
   files, and the next mission then fails for a reason that has nothing to do
   with it.
6. **Leave committed work alone.** Closing a worker never destroys what it
   produced; the closure ends the process, not the output.
7. **Assemble the synthesis** if it has not been produced already. Closing
   without one throws away the reason the fan out happened.
8. **Choose the signal honestly.**
   - *Clean:* every entry done and independently verified.
   - *Pending:* naming exactly which entries remain and what each needs.
   - *Failed:* with the evidence, not the interpretation.
9. **Report both halves:** what shipped, and what did not. The second half is the
   one the requester cannot reconstruct on their own.
10. **Stage the ledger, the verifications and the closure** to Context & Memory
    {OS}, and send the mission to Evaluation {OS} for scoring.
11. **Make the closure repeatable.** Running it a second time recomputes the live
    set, closes nothing that is already closed, and re-kills nothing.

## Completion test

- Every ledger entry has a state, and every done entry has an independent
  verification record.
- The live worker set was recomputed at closure time, not read from the ledger.
- No worker started by this mission is still running unaccounted for.
- Every scope claim is released, including those of dead workers.
- No committed work was destroyed by the closure.
- A synthesis exists.
- The signal is clean only when every entry is done and verified; otherwise it is
  pending with the remainder named, or failed with evidence.
- The report names both what shipped and what did not.
- The ledger, verifications and closure are staged to Context & Memory {OS}.
- Running the closure a second time changes nothing and errors on nothing.

An incomplete mission reported as clean is worse than an honest pending, because
it ends the work for everyone downstream who believed it.
