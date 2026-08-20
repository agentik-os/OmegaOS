# Verify a management claim

Produces an evidence entry with a named source, a date, a confidence level and
a seller or independent classification, and where the evidence contradicts the
claim, a registered finding with a severity and a consequence.

## Trigger

Management asserts something material in a meeting, a presentation or a data
room document, or a thesis claim depends on a present-day fact that has only
ever been stated by the seller.

## Inputs

- The claim, quoted as it was stated, with the speaker and the date.
- The document, extract or recording it came from, classified as seller sourced.
- The list of independent sources available: statutory filings, registries,
  system extracts observable directly, customers, suppliers, former employees,
  third party data.
- The decision relevance of the claim, from the diligence plan.

## Steps

1. Log the claim as an assertion first: who said it, when, and in what setting.
   An assertion is a valid evidence entry, at the confidence level `asserted`.
2. Restate the claim as something checkable. "Revenue is growing strongly"
   becomes "FY revenue grew from X to Y", which a source can settle.
3. Name the independent source that could settle it, and what it would show if
   the claim is true and if it is false. If no independent source exists, say
   so now rather than after two weeks of looking.
4. Obtain the evidence. Prefer direct observation, then independent
   corroboration, then a single independent source, then the assertion alone.
5. Log the evidence entry with source, date, confidence and the independent
   flag. Reject undated or unattributed material. A second seller document is
   not corroboration.
6. Compare the evidence with the claim and record one of four outcomes:
   confirmed, contradicted, partially supported, or unverifiable.
7. If contradicted or partially supported, register a finding with a severity
   and an explicit consequence class: price, structure, condition or walk. Do
   not propose a clause, a number or an instrument here. Emit
   `diligence.finding.registered`.
8. If the outcome is unverifiable, record it as unverified with its severity
   and put it on the gaps list. It is never scored as a pass and never
   described as "no issues identified".
9. If the contradiction meets the red flag threshold, stop and escalate with
   the evidence attached. **Human approval gate:** the calendar is paused and
   only a person decides to continue, restructure or stop.
10. If the claim requires a legal, tax or accounting conclusion to settle,
    record the question, name the profession that must answer it, and keep the
    item open until that written answer is attached. This OS does not supply
    the opinion.
11. Update the thesis claim in Investment Thesis {OS} that depended on this
    fact, so a prediction is not left resting on an assertion.

## Completion test

The evidence log contains an entry for the claim with a named source, a date, a
confidence level and an independent flag, the outcome is recorded as confirmed,
contradicted, partially supported or unverifiable, and any contradiction has a
register entry carrying a severity and a consequence class. If the claim rests
on the seller alone, it appears on the gaps list.

## Failure modes

| Failure mode | What it looks like | Response |
|---|---|---|
| Assertion promoted by repetition | the same management statement appears in three documents and starts reading as fact | keep the confidence at `asserted` regardless of how many seller documents repeat it |
| Corroboration from the same side | a seller spreadsheet used to confirm a seller claim | reject as corroboration, keep the item on the gaps list |
| Confidence inflation | a single seller document logged as high confidence | refuse the entry, since the confidence level and the source class must agree |
| Contested evidence | two credible sources disagree | log both, mark the item contested, escalate, and never resolve it by preferring the convenient one |
| Finding filed as a note | a contradiction recorded without a consequence | hold it out of the register as an open observation and escalate for a relevance decision |
| Opinion improvised | this OS writing what amounts to a tax or legal conclusion | decline, name the profession, keep the item open until their written answer arrives |
| Chased into the ground | weeks spent on a claim whose answer changes nothing | recheck the decision relevance from the plan and drop it if it fails |
