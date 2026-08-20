# Workflow: Close the gaps and the stale

Turn the questions the corpus cannot answer, and the passages the world moved
past, into work for whoever owns the material.

## Trigger

- The review cadence.
- The same question has been asked repeatedly and abstained on.
- An event invalidated a class of material: a policy change, a product change, a
  reorganisation, a version bump.
- Retrieval quality dropped on the fixed question set.

## Steps

1. **Collect every abstention since the last review,** with the question as it
   was actually asked. Paraphrasing questions into tidier ones destroys the
   signal.
2. **Cluster them.** Ten phrasings of one missing answer are one gap, not ten,
   and ranking by raw count without clustering hides the real one.
3. **Rank gaps by frequency and by consequence.** A rare question whose wrong
   answer is expensive outranks a common one that is merely inconvenient.
4. **Classify each gap:** missing document (route to Documentation {OS}), missing
   reading or source (route to Librarian {OS}), material that exists but is not
   ingested (route to ingestion), material that exists but is not retrievable
   (route to chunking and indexing), or genuinely out of scope (accept it, with a
   written reason, so it is not rediscovered next quarter).
5. **List every source past its review date,** with its owner.
6. **Identify supersessions:** which source replaced which, and on what date.
   Mark the old one superseded; do not delete it, because an answer given last
   month cited it and must remain traceable.
7. **Check for orphaned material:** sources whose owner has left, or whose
   subject no longer exists. These are marked unowned and reported, because an
   unowned source ages without anyone noticing.
8. **Re-run the fixed question set** after any reindexing done in this pass, and
   compare the score to the previous run.
9. **Send each gap and each stale source to its owner** with the specific
   question that exposed it. A gap report addressed to nobody changes nothing.
10. **Stage the report** to Context & Memory {OS} so the next review can see
    which gaps were closed and which have now been open for three cycles.

## Completion test

- Every abstention since the last review has been collected in its original
  wording.
- Gaps are clustered, then ranked by frequency and consequence.
- Every gap has exactly one classification and one named destination, including
  the ones accepted as out of scope with a written reason.
- Every source past its review date is listed with its owner.
- Supersessions are marked and old sources are retained, not deleted.
- Unowned sources are reported individually.
- The fixed question set has been re-run and the score recorded next to the
  previous one.
- The report is staged to Context & Memory {OS} with the status of the previous
  cycle's gaps.

A gap report that returns nothing is a valid result and is worth one line. A gap
report that returns the same top item for three cycles is a different finding
entirely, and it is about ownership rather than about the corpus.
