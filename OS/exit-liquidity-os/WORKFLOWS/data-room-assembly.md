# Workflow: Data room assembly

Turn the buyer's request list into an index where every line has a state, and
assemble the room without opening it to anyone.

**Mode:** `INDEX`
**Produces:** the diligence readiness index, the assembled room, and `exit.dataroom.indexed`
**Typical duration:** weeks, driven by how many documents do not yet exist

## Trigger

Any of:

- the readiness assessment has been scored and preparation has started
- a counterparty sends a diligence request list
- a letter of intent is under discussion with counsel
- the index is more than six months old and the business has changed

## Steps

1. **Start from the standard request list**, then merge in any list a
   counterparty has actually sent. A buyer's own list is authoritative for that
   buyer, and the difference between the two lists is itself informative about
   what they care about.

2. **Assign every line one of four states:** `present`, `partial`, `absent`,
   `not applicable`. `not applicable` requires a written reason. There is no
   fifth state, and there is no blank.

3. **For every `present` line, record where it lives and who owns producing
   it.** A document that exists but that nobody can locate on the day it is
   asked for behaves exactly like a document that does not exist.

4. **For every `absent` and `partial` line, create a gap** with an owner and a
   date, and classify it as paperwork, value or structural on the same scale the
   readiness assessment uses. Do not reconstruct a missing document from memory
   and file it as present. Diligence will test it, and a reconstruction found
   late costs more than the document was worth.

5. **Reconcile against the projections.** Every entity named in the room must
   match Ownership {OS}. Every IP asset must match the schedule in IP & Asset
   {OS}. Every financial statement must match what Revenue {OS} and the
   accountant hold. A contradiction stops assembly: report it, name the OS that
   resolves it, and do not choose a version here.

6. **Structure the room to the index**, one section per request category, with
   the index itself as the first document. A buyer who can navigate the room
   without asking questions asks fewer questions.

7. **Redact before release, not at release.** Personal data, employee
   identifiers, customer identifiers where the contract restricts disclosure,
   and anything under a third party confidentiality obligation. Where the
   redaction turns on a legal question, that question goes to counsel, not to
   this OS.

8. **Assemble, and stop.** The room is built, indexed and closed. This OS does
   not open a data room to an outside party, does not grant access, and does not
   send a link. Opening is a human action with an explicit approval.

9. **Every release goes through `/exit-release`, per document and per
   recipient.** Approval to send a document to one buyer is not approval to send
   it to a second buyer, to a broker, or to an adviser. Each release writes an
   append only disclosure log entry with the document, the recipient, the date,
   the approving human, and the confidentiality agreement relied on. A
   non-disclosure agreement is a binding legal instrument; whether the one in
   hand covers a given release is a question for counsel, and the log records
   which agreement was relied on precisely so that question can be answered
   later.

10. **Emit `exit.dataroom.indexed`** with the line count by state, the absent
    lines that block the earliest gate, and the date of the index.

11. **Re-run steps 2 to 5 whenever a projection changes.** A new entity, a new
    registration, a restated financial period, or a material new contract each
    invalidate part of the index.

## Completion test

The room is ready when all of the following hold:

- every line of the merged request list carries one of the four states, and no
  line is blank
- every `not applicable` carries a written reason
- every `present` line has a location and a named owner
- every `absent` and `partial` line has a gap with an owner, a date and a
  classification
- entities, IP assets and financials in the room reconcile with Ownership {OS},
  IP & Asset {OS} and Revenue {OS}, with zero unresolved contradictions
- redactions are applied, and the ones turning on a legal question are with
  counsel
- the room is closed, and no access has been granted to any outside party
- the disclosure log contains an approved entry for every document that has left
- `exit.dataroom.indexed` has been emitted

It is not done if a document was released without a logged approval, if a
contradiction with a projection is still open, or if any line reads as present
on the basis of a reconstruction rather than the document itself.
