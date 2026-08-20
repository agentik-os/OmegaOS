# Workflow: Corpus audit

**Produces:** the defect list of the corpus, per record, with the unusable
extracts quarantined rather than silently cited.

## Trigger

Any of:

- Monthly, or on whatever cadence you actually keep.
- Before the corpus is trusted for something that matters (a published piece, a
  decision, a client deliverable).
- After a bulk import, which is where duplicates and lost locators arrive.

## Steps

1. **Extracts without a locator.** List them. Quarantine each: it stays in the
   corpus, it is excluded from `ASK`, and it is flagged for repair by returning
   to the source.
2. **Sources without extracts.** List them with their ingest date. A source
   ingested six months ago with no extracts is either unread or was not worth
   keeping; say which.
3. **Duplicate source records.** Match on author plus title similarity plus
   edition. Propose merges. Do not merge without approval, because a merge
   silently rewrites which record every extract belongs to.
4. **Superseded editions still cited.** Find extracts from an edition that a
   later record supersedes, and every synthesis or answer that used them.
5. **Reconstructed extracts.** Extracts whose locator was added from memory
   rather than from the page. Report them separately from clean ones.
6. **Unindexed present sources.** Sources the corpus holds but cannot search, so
   that `ASK` results can honestly say what they did not look at.
7. **Licence drift.** Extracts whose source is restricted but whose quote length
   exceeds what the licence permits.
8. **Stale index.** Sources ingested after the last index build.
9. **Rank the defects by consequence,** not by count: a wrong citation in a
   published piece outranks fifty unread books.
10. **Emit `librarian.corpus.audited`.**

## Completion test

- Every defect names the record it lives in.
- Every locator-less extract is quarantined and excluded from retrieval.
- Proposed merges are listed and none was executed without approval.
- Every synthesis affected by a superseded edition is named.
- The report ends with the ranked repair list, and the top item is the one whose
  being wrong costs the most.
