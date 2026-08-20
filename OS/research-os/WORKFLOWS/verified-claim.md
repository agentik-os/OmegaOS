# Workflow: Verified claim

**Produces:** one claim with its full provenance chain, verified against the
primary source, classed, dated, and marked with its support type.

## Trigger

A single claim is load-bearing: a decision, a model, a memo or a public document
rests on it. Or a number is circulating and nobody can say where it came from.

## Steps

1. **Restate the claim exactly.** Word for word as it is being used, not as you
   would prefer to phrase it. A claim rewritten before verification verifies a
   different claim.
2. **Make it checkable.** Name the subject, the magnitude, the population and the
   time window. "Adoption is growing fast" is not checkable. "Reported enterprise
   deployments grew from X to Y between 2024 and 2026, among firms above 500
   employees" is.
3. **Find the origin, not the nearest repetition.** Follow the citation chain
   backwards. A blog citing an article citing a press release is one source, and
   the press release is the origin.
4. **Retrieve the primary.** The filing, the dataset, the standard, the paper,
   the court record, the official statistic, the vendor's own documentation.
   Record its locator and the date accessed.
5. **Check that the primary says it.** Compare the claim to what the primary
   actually states. Watch for the four common drifts: the population changed, the
   time window changed, a modelled projection became a measurement, or a range
   became its most favourable endpoint.
6. **Record the method behind any number.** How it was measured, over what
   population, with what sample, and what it excludes. A measurement without a
   method is an assertion with a decimal point.
7. **Date it twice.** The date of the evidence and the date you accessed it. Then
   check whether a newer version supersedes it.
8. **Disclose the interest.** Who produced this, who paid for it, what they gain
   if it is believed. A funded source is usable; a funded source presented as
   neutral is not.
9. **Look for independent support.** Find at least one source that reached the
   same result by a different route. If every path leads back to the same origin,
   mark the claim `circular` and treat it as single-sourced.
10. **Actively look for the refutation.** Search for the counter-evidence
    specifically, not incidentally. A claim that was never argued against was
    never checked.
11. **Assign the class.** `fact`, `measurement`, `inference`, `assumption`,
    `unknown` or `contested`. If the primary could not be reached, the claim
    keeps its secondary citation and carries the note that the primary was not
    verified.
12. **Assign confidence with its reason.** Independent source count, tier of the
    origin, recency, method quality, and what remains unknown.
13. **Record the verdict on the claim as evidence, not as a decision.** Verified,
    verified against a secondary only, unreopenable, or contested. If it is
    contested and a decision rests on it, emit `research.claim.contested` and
    hand it to Validation {OS}. Research does not settle it here.
14. **Write it to the claim set** with the full chain, so nobody has to redo this
    work when the claim appears again in another document.

## Completion test

- The claim as verified is the claim as used, word for word.
- The citation chain is recorded back to its origin, and the origin is named.
- The primary was retrieved, or the record states plainly that it could not be
  and why.
- Population, time window and method are recorded for any number.
- Both dates are present: date of evidence, date of access.
- The producing interest is disclosed where one exists.
- Independent support is named, or the claim is explicitly marked
  single-sourced or `circular`.
- A specific search for the refutation was run and its result recorded, even when
  it found nothing.
- The claim carries exactly one class and a confidence level with its reason.
- No verdict language appears anywhere in the record.
