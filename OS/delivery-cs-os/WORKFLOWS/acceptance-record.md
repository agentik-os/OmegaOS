# Acceptance record

Close a deliverable on evidence the customer produced, not on a status field the
team set.

## Trigger

A deliverable or milestone in the client plan is complete internally and is
being put forward for acceptance.

## Steps

1. **Delivery & Customer Success {OS}** retrieves the acceptance criteria agreed
   in the client plan for this milestone. Produces: the criteria.
2. **Quality & Evaluation {OS}** supplies its acceptance evidence for the work
   itself: tests, review, release verification. Produces: quality evidence.
3. **Delivery & Customer Success {OS}** checks the deliverable against the
   criteria and against the promise register rows it was meant to satisfy. A
   promise not satisfied is named, not omitted. Produces: a criteria check.
4. **Delivery & Customer Success {OS}** drafts the acceptance request to the
   customer, stating exactly what is being accepted and against which criteria.
   Produces: an unsent request.
5. **Human** approves the exact text of the customer-facing request. Produces:
   approval.
6. **Customer** performs an acceptance act: a written confirmation, a sign-off,
   or the agreed equivalent. Produces: customer evidence with a timestamp.
7. **Delivery & Customer Success {OS}** writes the acceptance record with the
   customer's evidence attached, the criteria it satisfies, and the promise
   register rows it closes. Produces: the acceptance record.
8. **Delivery & Customer Success {OS}** emits the acceptance record to Revenue
   {OS}, which may make the milestone billable, and to Growth {OS} as retention
   evidence.
9. **Delivery & Customer Success {OS}** opens the adoption question: the
   deliverable is accepted, and usage is a separate measurement that starts now.

## Completion test

The acceptance record contains customer-produced evidence with a timestamp, and
every promise register row it claims to close is satisfied by that deliverable.
An acceptance record with no customer evidence, or one that closes a promise the
deliverable does not satisfy, fails this workflow. Internal completion never
substitutes for the customer act.

## Failure and abort

- The customer says it is not accepted: it is not accepted. Record the gap in
  their words, convert it to the next work item, and leave the milestone open.
- Quality evidence is absent: do not request acceptance. Meeting standard is an
  input to asking, and asking without it spends the customer's trust to find a
  defect they should never have seen.
- The customer accepts without engaging: record the acceptance, and flag the
  low-engagement acceptance as an adoption risk. A signature is not usage.
- A promise register row is unsatisfied at acceptance: it stays open and is
  named explicitly in the acceptance request. It is not closed by proximity to
  work that did land.
- Approval on the request text is withheld: nothing is sent, and the milestone
  stays internally complete and externally unaccepted, which is exactly what it
  is.
