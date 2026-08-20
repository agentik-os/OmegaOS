# Workflow: Recover a failed mission

Resume what can be resumed, close honestly what cannot, and turn the failure
into a change to the shape.

## Trigger

- A step failed terminally.
- The session was interrupted, restarted, or its context was compacted.
- The budget ceiling was reached.
- A loop is not converging.
- A step has been blocked with nobody watching.

## Steps

1. **Read the ledger from the file.** Not from memory, and not from the
   transcript. The memory of a plan is exactly what an interruption destroys.
2. **Establish the true state of every entry.** An entry marked in progress by a
   process that no longer exists is not in progress; it is unknown, and it is
   re-established by inspection rather than by assumption.
3. **Account for every worker the mission started.** Some are still running, some
   are dead and still hold a scope claim. Release the claims of dead workers,
   because a stale claim rejects the next legitimate step on the same files.
4. **Resume at the first entry that is not done,** in ledger order.
5. **For a terminally failed node, decide once:** retry within the bound, use the
   declared fallback, or mark it failed and report which downstream entries
   became unreachable. Do not retry an unbounded number of times because the
   failure is intermittent.
6. **For a non converging loop, stop it at its bound** and report what it kept
   rediscovering. Check the deduplication key: a loop that deduplicates only
   against accepted results will rediscover every rejected one forever.
7. **For a budget overrun, escalate rather than continue.** State what has been
   completed, what remains, and what the remainder is expected to cost.
8. **For a blocked step, escalate to a human** and never re-dispatch into the
   same block. Re-dispatching a blocked step is not persistence, it manufactures
   thrash.
9. **Never quietly drop an ask.** An ask that cannot be completed stays in the
   ledger as not done, and appears in the report as not done.
10. **Synthesise whatever did complete,** so a partial mission still returns
    usable work rather than nothing.
11. **Close honestly** with pending or failed, naming exactly what remains and
    the evidence for the failure.
12. **Postmortem into a shape change:** the barrier that should not have existed,
    the missing scope isolation, the loop bound that was wrong, the verification
    that was never attached. A resolution to be more careful is not an output.

## Completion test

- The ledger was read from its file and every entry has a re-established state.
- Every worker started by this mission is accounted for, and every dead worker's
  scope claim is released.
- Retries respected their bound, and fallbacks were used where declared.
- A non converging loop was stopped at its bound with its rediscovery reported.
- A budget overrun produced an escalation before continuing.
- No blocked step was re-dispatched into the same block.
- No ask was removed from the ledger; incomplete asks are marked not done.
- Completed work was synthesised even though the mission is partial.
- The closure signal is pending or failed, with what remains named.
- The postmortem names a specific change to the topology or the ledger
  discipline.

A recovered mission that closes clean because the remaining asks were deleted
from the ledger is worse than one that closes pending, and it is the exact
failure this workflow exists to make impossible.
