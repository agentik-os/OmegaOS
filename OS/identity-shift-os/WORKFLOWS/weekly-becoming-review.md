# Identity Shift {OS}: Weekly becoming review

**Produces:** a dated review record for the period: the evidence balance, the drift finding, one adjustment, and an explicit verdict of continue, amend or close.
**Trigger:** the review cadence set in the charter fires (weekly by default), or the user asks where the shift stands, or the close-by date arrives.
**Runs in:** `REVIEW`, and `CLOSE` when the verdict is close.
**Takes:** the charter, the evidence ledger entries since the last review, behaviour contract evidence from Habit Tracker {OS}, and any capacity report from Health & Energy {OS}.

## Steps

1. Read the charter first, not the ledger. Restate the target identity and the
   exit test in one line each, so the period is judged against what was agreed
   rather than against what happened.
2. List every ledger entry since the last review with its date and class. Count
   confirming, disconfirming and ambiguous separately. Report the counts before
   interpreting them.
3. Pull the behaviour contract evidence from Habit Tracker {OS} for the same
   dates. Where the contract log and the ledger disagree, report both and treat
   the contract log as the harder evidence.
4. Check for drift: is the shift still moving toward the chartered target, or
   has the target quietly changed during the period? A changed target is an
   amendment and must be recorded as one, with the reason and the date.
5. Read the disconfirming entries out on their own. If the period produced only
   confirming entries, say so and treat it as a warning about the recording
   habit, not as proof.
6. If the period produced no entries at all in either direction, record it as
   empty. Two consecutive empty periods force a close decision at this review.
7. Check capacity. If Health & Energy {OS} reports insufficient capacity, or a
   safety signal is present, move the shift to `HOLD`, write the resume
   condition, and end the review there.
8. Make exactly one adjustment: change the carrying behaviour, change its floor,
   change the environment, or change nothing and say why nothing is the right
   move this week.
9. Write the verdict: continue, amend (with the reason recorded), or close. If
   the close-by date has arrived, the verdict cannot be continue.
10. Persist the review record through Context & Memory {OS} and send a copy to
    Journal {OS} as reflective material, not as a conclusion about the person.

## Completion test

A dated review record exists for the period; it names the counts of confirming,
disconfirming and ambiguous entries; it states whether the shift drifted; it
carries exactly one adjustment; it ends with continue, amend or close; and no
review dated on or after the close-by date carries the verdict continue.

## Failure

If there are no ledger entries and no contract evidence, the review reports an
empty period rather than inferring progress. If Habit Tracker {OS} evidence is
unavailable, the review runs on ledger entries alone and labels its conclusion
self-reported. If the charter and the evidence describe different targets, the
review stops at step 4 and asks the user which one is real before any adjustment
is made. If the exit test turns out not to be checkable in practice, the review
halts, the charter is amended with a checkable test and the amendment date, and
the record states that earlier reviews ran against a weaker test.
