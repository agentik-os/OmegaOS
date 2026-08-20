# Workflow: Answer with citations

Answer a real question so that every claim can be opened and read at its source,
or abstain honestly.

## Trigger

- Someone asks a question whose answer must be defensible.
- Another OS requests a fact it intends to act on.
- A claim already in circulation is being checked.

## Steps

1. **Restate the question** in the terms the corpus uses. Most retrieval failures
   are vocabulary failures, and this step is cheap.
2. **Resolve the asker's permission scope first.** Passages the asker may not see
   are excluded from the working set entirely, not retrieved and then hidden.
3. **Retrieve candidate passages** and read them before composing anything.
4. **Judge whether the passages actually answer the question.** If they do not,
   stop and say the retrieval failed. Synthesising a plausible answer around
   passages that do not support it is the failure mode this whole OS exists to
   prevent.
5. **Check for disagreement between sources.** Where two sources contradict,
   return both with their authors and dates, and do not choose. Choosing is a
   judgment for the asker or for Context & Memory {OS}.
6. **Check dates.** A passage past its review date, or superseded, is marked in
   the answer itself. Never present stale material as current.
7. **Compose the answer as discrete claims.** Each claim carries a citation to a
   source and a span. A sentence with no citation is either explicitly labelled
   as common knowledge or removed.
8. **State the confidence honestly,** based on the authority of the sources and
   whether they agree, not on the fluency of the answer.
9. **If the corpus is silent, abstain properly:** what was searched, what terms
   were tried, what is absent, and what source would resolve it. Log the question
   for the gap report.
10. **Offer the trace.** Any claim can be followed back to chunk, source, span
    and original document on request.
11. **If the answer is intended to become a durable fact,** stage it to Context &
    Memory {OS} as a sourced record for confirmation. It is not canonical here.

## Completion test

- Every claim in the answer cites a source and a span that can be opened.
- No passage outside the asker's permission scope entered the working set.
- Contradicting sources are presented as a contradiction, with dates, not
  resolved silently.
- Stale and superseded passages are marked in the answer.
- If the corpus did not support an answer, the output is an abstention that names
  what was searched and what would resolve it, and the question is logged as a
  gap.
- Any claim can be traced end to end on request.
- Nothing was promoted to canonical truth inside this workflow.

The measure of this workflow is not how often it answers. It is that the answers
it gives can be checked, and that the questions it cannot answer are visible
rather than smoothed over.
