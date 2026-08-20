# Release verdict

Decide whether a specific story may go out on a specific surface, and say so in
one of the six verdicts rather than in a percentage.

## Trigger

A story object is proposed for release: handed to Content {OS} for publishing,
used by Sales {OS} as proof, given to Affiliate {OS} for a partner surface,
told from a stage, or sent to a customer.

## Steps

1. **Requester** names the story object and the exact surface. Consent granted
   for one surface never transfers to another, so the surface is part of the
   question.
2. **Storyteller {OS}** re-runs `/truthcheck` against the current version of the
   object and produces the truth class of every load bearing element as it
   stands today, not as it stood when the object was created.
3. **Storyteller {OS}** reads the consent records and produces, per named
   person, whether this surface is inside the consented scope and whether the
   consent has expired.
4. **Positioning {OS}** supplies the ledger status of any claim the story leans
   on. Storyteller {OS} produces the claim check: live, contested or expired. A
   contested or expired claim may not be dramatised.
5. **Brand {OS}** supplies the voice rules. Storyteller {OS} runs `/voice` and
   produces the conformance report and any conflict.
6. **Storyteller {OS}** runs `/score` and produces the structural completeness
   report, kept separate from the gates and explicitly labelled as neither a
   truth signal nor a virality prediction.
7. **Storyteller {OS}** runs `/rehearse` when the surface is spoken, and
   produces the delivery notes and the beats that lose the room.
8. **Storyteller {OS}** issues the verdict with its reason: READY, READY WITH
   CUTS, NEEDS TRUTH CHECK, NEEDS DEEPENING, WRONG STORY FOR THIS JOB, or DO
   NOT PUBLISH.
9. **Human** approves the exact text on the exact surface. A verdict of READY
   is a clearance, not a publication decision, and publication belongs to
   Content {OS}.
10. **Storyteller {OS}** records the verdict, the surface and the approver on
    the story object with `omega-story update`, so the next release request
    starts from what was decided last time.

## Completion test

The story object carries a verdict for this surface, dated, with the truth check
and the consent check both recorded as pass at that date, and the claim check
recorded as live. A verdict of READY exists only when both gates passed
independently of the score. A verdict recorded without a named surface, or a
READY issued while any load bearing fact or any consent is unresolved, fails.

## Failure and abort

- A load bearing fact is uncertain at step 2: return NEEDS TRUTH CHECK, name
  the fact, and stop. The score is irrelevant here, because VERIFY is a gate and
  the score is not.
- Consent is out of scope or expired at step 3: return DO NOT PUBLISH, and
  offer the abstracted or anonymised version as a separate object rather than
  quietly editing this one.
- The claim the story supports is contested or expired at step 4: return DO NOT
  PUBLISH for claim bearing surfaces, and route the claim question to
  Positioning {OS}. Storyteller does not adjudicate a claim.
- The story is true, deep and well shaped but will not do the job asked of it:
  return WRONG STORY FOR THIS JOB with the reason, and offer `/repurpose` to
  find a story that will. A well made story that misses the job is still a
  failure.
- A human overrides a DO NOT PUBLISH: record the override with the approver's
  name, the reason and the date on the story object, and never restate the
  verdict as READY. The override is a decision, not a re-verification.
