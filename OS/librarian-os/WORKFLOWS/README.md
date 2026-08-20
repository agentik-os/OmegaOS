# Librarian {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [indexed-source.md](indexed-source.md) | a source enters your world | a canonical source record, indexed, with licence limits recorded |
| [reading-extract.md](reading-extract.md) | a source is read, or needed for a specific job | typed extracts, each with a locator back to the page or timestamp |
| [corpus-answer.md](corpus-answer.md) | a question is asked that the corpus may already answer | a cited answer plus an explicit statement of what the corpus does not cover |
| [corpus-audit.md](corpus-audit.md) | the corpus is about to be trusted for something important | a defect list per record, with the quarantined extracts named |
