# Workflow: Reconcile the cap table

Produces a line-level diff between the working register, the source documents
and the statutory register, and leaves every difference either resolved or
raised to a named human.

## Trigger

Any of:

- a funding round, transfer, issuance or cancellation closes
- a buyer, an investor or an adviser opens due diligence
- the quarterly review cadence
- any output would otherwise be published with a percentage the user doubts

## Steps

1. **Freeze the working register.** Snapshot the current positions and
   denominators for the entity under review, so the diff has a fixed left-hand
   side. Reads: Context & Memory {OS}. Writes: a session snapshot, temporary.
2. **Collect the right-hand side.** Gather the source documents for every line:
   subscription agreements, share-transfer forms, grant notices, board and
   shareholder resolutions, and the latest statutory register extract. Where a
   document is missing, mark that line `unevidenced` and continue rather than
   stopping. Touches: documents, read only.
3. **Diff line by line.** For each holder and instrument, compare quantity,
   class and rights. Classify each line as `matched`, `differing` or
   `unevidenced`. Never merge a differing line into a single value. Writes: the
   discrepancy report, temporary.
4. **Recompute the denominators from the documents.** Rebuild issued,
   outstanding and fully diluted counts from the evidence rather than from the
   stored figure, then compare. A denominator drift silently shifts every
   percentage in the table, so it is checked before the percentages are.
5. **Apply the precedence rule.** Where the working register and the statutory
   register disagree, the statutory register is treated as correct for that line
   and the difference is raised as a discrepancy. The working register is not
   edited at this step. Reads: step 3.
6. **Route the conflicts.** Where two source documents disagree with each other,
   present both with their dates and clause references and produce a
   `/counsel-pack` for the lawyer or corporate secretary. Do not choose between
   them. Emits: `ownership.term.flagged` where the conflict is a term rather
   than a quantity.
7. **Approval gate.** Present the full report: matched lines, differing lines
   with both values and both sources, and unevidenced lines. The user decides,
   line by line, which corrections to apply. Nothing mutates before that.
8. **Apply and mark.** Write the approved corrections, set the verification
   state per line, and record the source document reference on each corrected
   fact. Writes: Context & Memory {OS}. Emits: `ownership.position.valued` to
   Wealth {OS} for lines that are now `verified`, and only those.
9. **Close the loop.** Add each remaining `unevidenced` line to the document
   request list, and each outstanding conflict to the obligations calendar with a
   responsible human and a date. Emits: `ownership.obligation.due` to Execution
   {OS}.

## Completion test

- Every line in the entity's cap table is classified `matched`, `differing` or
  `unevidenced`, with none left unclassified.
- Every `differing` line shows both values and both sources in the report.
- The denominators were rebuilt from documents in this run, not carried over.
- No line was silently corrected: every applied change traces to a user approval
  in step 7 and carries its source document reference.
- Only `verified` lines were emitted to Wealth {OS}.
- Every unresolved item has a named human and a date against it.
