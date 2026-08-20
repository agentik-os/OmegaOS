# Documentation {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`doc-inventory.md`](doc-inventory.md) | the document set is unknown, inherited, or no longer trusted | an inventory with an owner, a location and a verified date per document, plus the duplicate and orphan lists |
| [`write-once.md`](write-once.md) | someone is about to write a document, or has answered the same question twice | one document in the canonical location, answering one question, with its four required fields |
| [`freshness-sweep.md`](freshness-sweep.md) | the review cadence fires, or reality changed under a document | verified dates advanced, corrections applied by their owners, and drift reports for what nobody can confirm |
