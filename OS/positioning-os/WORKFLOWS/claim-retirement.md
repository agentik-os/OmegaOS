# Claim retirement

Take a claim off the market on purpose, once evidence stops supporting it, and
make sure every surface that repeats it stops repeating it.

## Trigger

A `/position-review` returns expired or contested, a competitor demonstrably
matches the claim, a benchmark is rerun and lost, a customer disputes the claim
in a sale, or the expiry condition written into the ledger entry fires.

## Steps

1. **Positioning {OS}** presents the claim, its original evidence, its expiry
   condition and the event that fired it, side by side.
2. **Positioning {OS}** runs `/position-test` once more and produces a current
   verdict, so the retirement rests on a retest rather than on an impression.
3. **Positioning {OS}** produces the consumer list: every downstream unit
   holding the claim and, through Content {OS}, every published asset that
   repeats it.
4. **Positioning {OS}** classifies the retirement: replaceable (a narrower
   claim survives the evidence), degraded (true under stated conditions only),
   or dead (no version survives).
5. **Positioning {OS}** drafts the replacement claim when the class is
   replaceable or degraded, and routes it through the claim ledger workflow
   rather than shortcutting it.
6. **Human** approves the retirement, the classification and the replacement
   text exactly as written.
7. **Positioning {OS}** writes the retirement record: what was claimed, what
   killed it, the date, the replacement if any, and the assets to correct.
8. **Positioning {OS}** notifies Content {OS}, Sales {OS}, Storyteller {OS},
   Growth {OS}, Brand {OS}, Offer {OS} and Affiliate {OS}, each with the
   surfaces it owns that need correcting.
9. **Content {OS}** and **Sales {OS}** confirm the correction of the surfaces
   they own, and Positioning {OS} records each confirmation against the
   retirement record.
10. **Positioning {OS}** closes the record only when every listed surface is
    confirmed corrected or explicitly waived by a human with a reason.

## Completion test

The retirement record exists with the killing evidence and its date, the claim
no longer appears as live anywhere in the ledger, and every surface on the
step 3 consumer list is marked either corrected with a confirmation from its
owning unit or waived with a named human and a written reason. An open surface
with neither state means the retirement is incomplete, regardless of how long
ago it was approved.

## Failure and abort

- The retest at step 2 passes: do not retire. Record the scare, reset the
  expiry condition, and say plainly that the claim survived.
- No consumer list can be produced because Content {OS} has no asset inventory:
  proceed with the retirement, mark the surface list incomplete, and escalate,
  because an unknown surface still repeating a dead claim is the real exposure.
- Human approval withheld at step 6: the claim stays in the ledger marked
  contested, never live, and downstream units are told it may not be published
  while contested.
- A replacement claim fails its own test: retire the original anyway. A gap in
  the ledger is honest; a claim kept alive because nothing better exists is not.
