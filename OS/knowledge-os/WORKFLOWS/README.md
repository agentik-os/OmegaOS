# Knowledge {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, its ordered steps, and its completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`ingest-a-source-corpus.md`](ingest-a-source-corpus.md) | material should become retrievable | registered, screened, chunked sources and a rebuildable index |
| [`answer-with-citations.md`](answer-with-citations.md) | a question is asked whose answer must be defensible | claims each citing a span, or an abstention naming what is absent |
| [`close-the-gaps-and-the-stale.md`](close-the-gaps-and-the-stale.md) | a review cadence, or repeated unanswerable questions | a gap report routed to the material's owner and a staleness pass |

One invariant cuts across all three: a claim without a traceable span is not a
claim, it is a guess with a confident tone.
