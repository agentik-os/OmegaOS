# Workflow: the freshness sweep

Produces advanced verified dates on documents that are still true, corrections
on those that are not, and drift reports for the ones nobody can confirm.

## Trigger

The review cadence fires, or something in reality changed under a set of
documents: a process was simplified, a price moved, a tool was replaced, a
project landed.

## Inputs

- The index, filtered to documents at or past their review date.
- Recent changes from Operations {OS}, Project {OS}, Process & SOP {OS} and
  Meeting {OS} decision records.
- The owners.

## Steps

1. **List the stale set** by owner, ordered by traffic multiplied by the
   consequence of being wrong.
2. **Check each document against a real source.** A person who does the work, a
   live system, a signed contract, a measurement. Not against another document.
3. **Classify each one:**
   - True: advance the verified date and the next review date.
   - Wrong in detail: the owner corrects it, and the verified date advances only
     after the correction.
   - Wrong in substance: the document is drifted. Report it, mark it, do not
     rewrite it unilaterally.
   - Dead: nothing uses it and nothing it describes still exists. Route to
     retirement, which asks a human first.
4. **Never advance a verified date on an edit alone.** Editing is not verifying.
5. **Cross-check the change feeds.** Anything Operations {OS} simplified or
   Process & SOP {OS} republished this cycle invalidates the documents that
   describe the old way, whether or not they are past their review date.
6. **Report drift to the owner** with three lines: what the document claims,
   what is now true, and what would resolve it.
7. **Mark the stale ones visibly** so a reader who arrives before the sweep
   finishes is warned rather than misled.
8. **Send the unresolvable set to Review & Governance {OS}**: documents with no
   owner, or where two owners disagree about what is true.

## Completion test

- Every document at or past its review date has been classified into true,
  wrong in detail, drifted or dead.
- No verified date advanced without a check against a real source.
- Every drift report names the claim, the current truth and the resolver.
- Stale documents that were not reached are visibly marked as unverified.
- Documents invalidated by this cycle's process changes were swept even if they
  were not yet due.

## Failure paths

| Situation | Response |
|---|---|
| the owner does not respond | leave the document marked stale, and escalate after the second cycle; never silently adopt it |
| reality itself is disputed | record both positions with their sources and route the resolution to Review & Governance {OS} |
| the stale list is too long to sweep | sweep by risk, and publish the unswept remainder as unverified rather than pretending coverage |
| a document is correct but nobody can find it | that is a findability defect, not a freshness one; run the index repair instead |
