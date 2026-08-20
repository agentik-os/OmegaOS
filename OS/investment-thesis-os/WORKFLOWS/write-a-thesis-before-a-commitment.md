# Write a thesis before a commitment

Produces a timestamped thesis with falsifiable claims, kill criteria and a
checkpoint date, stored before any money moves.

## Trigger

An opportunity has been qualified in Deal Flow {OS}, or the user has named a
bet directly, and a commitment is being contemplated. Also triggered whenever
the user asks Capital {OS} for an allocation and no thesis reference exists.

## Inputs

- The opportunity name and stage.
- The user's reasoning in their own words, unedited.
- The intended commitment size and the cost of exiting it, from Capital {OS}.
- Any Due Diligence {OS} findings already registered, with sources and dates.
- The user's pattern library from prior retirements, from Context & Memory {OS}.

## Steps

1. Check whether the commitment has already been made. If it has, stamp the
   thesis `retrospective`, tell the user it will not count in the hit rate, and
   continue. Do not silently proceed as if it were written in time.
2. Capture the user's reasoning verbatim before structuring anything. Store the
   raw version alongside the structured one.
3. Structure the reasoning into the four questions: what must become true, why
   now, why us, what are we paid for taking which risk. Leave a question blank
   and named rather than filled with a plausible sentence.
4. Separate facts from claims. Every present-day fact must point to a Due
   Diligence {OS} finding with a source and a date. Anything else becomes a
   claim requiring verification, and is sent to Due Diligence {OS} as a scoped
   question.
5. Rewrite each claim with its disproof condition and the date by which the
   disproof should be observable. Strike anything unfalsifiable and record why.
6. Read the exit cost from Capital {OS} and set kill criteria against it, while
   the position is still cheap to abandon. Emit `thesis.kill_criteria.set`.
7. Run the pre-mortem: two years on, the money is gone, what most likely
   happened. Rank the causes, map each to a claim, and convert unmonitored gaps
   into claims or record them as accepted blind spots.
8. Review the pattern library for the user's recurring failure modes and check
   the thesis against them by name.
9. Store and timestamp the thesis, emit `thesis.drafted`, and set the first
   checkpoint date and its evidence source.
10. Hand the thesis reference to Capital {OS}. **Human approval gate:** the
    allocation decision and any transfer of money is made by a person in
    Capital {OS}. This workflow ends at the reference and never at an approval,
    an instruction or a payment.

## Completion test

A stored thesis exists with a write timestamp earlier than the commitment
timestamp in Capital {OS}, every claim in the register carries a disproof
condition or a strike reason, a kill criteria sheet exists with its set date,
and a checkpoint date is on the calendar with a named evidence source.

## Failure modes

| Failure mode | What it looks like | Response |
|---|---|---|
| Written after the cheque | the commitment timestamp precedes the thesis | label `retrospective`, exclude from hit rate, state it in the output |
| Unfalsifiable thesis | every claim survives every possible world | report the unfalsifiable share before anything else, do not proceed to kill criteria until at least the core claims are testable |
| Facts smuggled in as claims | management assertions written as established truth | move them to Due Diligence {OS} as scoped questions and mark them unverified |
| Kill criteria set to fit the position | criteria that could never be met | rewrite against the real exit cost, and record if they were set after entry |
| Checkpoint with no evidence source | a date with nothing behind it | name the source or the claim is untestable by construction, and say so now rather than at the checkpoint |
| Time pressure | a commitment scheduled before drafting finishes | ship three falsifiable claims plus kill criteria and mark the thesis partial, rather than delivering nothing |
