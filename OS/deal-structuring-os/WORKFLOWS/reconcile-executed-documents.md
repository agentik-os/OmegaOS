# Reconcile executed documents

Produces a line by line comparison of the documents that came back from the
lawyers against the terms that were actually agreed, with every difference
raised before completion.

## Trigger

Draft or final transaction documents arrive from either side's lawyers. The
workflow runs on every version, not only the last one, because a difference
introduced in version three is cheapest to catch in version three.

## Inputs

- The agreed term sheet, at its final agreed version, from Context & Memory
  {OS}.
- The term register, with the cash value of each priced term.
- The executed or near final documents from the lawyers.
- The protection register and the incentive model, so a weakened protection is
  recognised as a weakened protection and not read as tidier drafting.
- The conditions from Due Diligence {OS} that were supposed to appear in the
  documents.

## Steps

1. Build the checklist from the agreed term sheet: one row per agreed term, plus
   one row per diligence condition that was to be reflected in the documents.
2. Locate each term in the documents. A term that cannot be located is a
   difference, not an oversight to be assumed away.
3. Compare substance, not wording. A clause that reads differently but produces
   the same cash is not a difference. A clause that reads similarly and changes
   the cash is.
4. For every difference, re-run the model and state its cash effect at the three
   exit values.
5. Check the defined terms and the definitions, since a change in a definition
   moves value silently across many clauses at once.
6. Check what is absent: an agreed protection that simply does not appear, an
   information right that was dropped, a condition that was quietly satisfied
   by a representation instead.
7. Classify each difference: administrative, value moving, or unacceptable.
8. Report the value moving and unacceptable differences with their cash effect,
   ranked by size, before completion.
9. **Human approval gate:** accepting any difference is a human decision. The
   OS never accepts a difference on its own authority, however small it looks.
10. Send unresolved differences back through Acquisition {OS} into the
    negotiation, and the drafting questions to the lawyer.
11. Record the final reconciled position, and mark the term sheet version it
    corresponds to.
12. Emit `structure.terms.agreed` only once the reconciliation is clean or every
    difference has been explicitly accepted by a human.

## Completion test

Every agreed term and every diligence condition has a row in the reconciliation
report, each row is marked matched, differing or absent, every differing and
absent row carries a cash effect and an explicit human decision, and the report
is dated before completion rather than after it.

## Failure modes

| Failure | What happens |
|---|---|
| a term cannot be located in the documents | it is recorded as absent, never assumed to be implied elsewhere |
| a definition changed | the workflow re-checks every clause that uses it, since one definition can move value across many |
| a difference arrives at the last hour | it is still priced and still raised, because time pressure is the condition under which bad terms are accepted |
| the lawyers say a change is standard | standard is not a reason, the change is priced and decided on its cash effect |
| a protection was replaced by a representation | that is reported as a value moving difference, since a promise is not a protection |
| the reconciliation is skipped to save a day | the record shows it was skipped, and any post completion dispute starts from that fact |
