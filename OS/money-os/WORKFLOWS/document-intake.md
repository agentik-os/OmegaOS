# Workflow: Document intake

Turn whatever the operator actually has, a bank export, a card statement, a
photographed receipt, into staged records that carry their source.

**Trigger:** `/money-intake <path>` or `/money-intake --paste`. Also triggered
by the close workflow when an account has no document for the period.

**Owner:** the operator confirms every promotion from staged to verified. The OS
never promotes on its own confidence alone.

## Steps

1. **Identify the document.** Determine the account, the period covered, the
   currency and the document type. If the account cannot be identified, ask; do
   not attach the lines to the most likely account.
2. **Extract.** Parse lines into date, amount, direction, counterparty, and
   where present the running balance. Each field carries an extraction
   confidence.
3. **Hold low confidence.** Any field below the confidence floor stays empty and
   is asked about. An amount is never inferred from surrounding context, and a
   date is never inferred from the file name.
4. **Check the period.** Compare the document's coverage against what is already
   staged or verified for that account. Report gaps (a missing week) and
   overlaps (a re-imported statement) before anything is counted.
5. **Detect duplicates.** Match against verified records by account, date,
   amount and running balance. Present suspected duplicates as a list for the
   operator to rule on.
6. **Suggest classification.** Apply confirmed rules first, then suggest for the
   rest with the reason for the suggestion shown. Never re-categorise a line the
   operator already categorised.
7. **Stage.** Write staged records with their source document path, the page or
   line reference where it exists, and the date seen. Staged records are visible
   everywhere but counted nowhere.
8. **Report.** Return the counts: lines staged, auto-classified, awaiting the
   operator, suspected duplicate, and unreadable.

## Completion test

Intake is complete for a document when:

- every line in the document is staged, or explicitly listed as unreadable with
  what was legible
- the account, period and currency are known and recorded, not assumed
- period gaps and overlaps against existing records are reported
- no staged record has been counted in any total, forecast or runway figure
- nothing was promoted to verified without an explicit operator confirmation
