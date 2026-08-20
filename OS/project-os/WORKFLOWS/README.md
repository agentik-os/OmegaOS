# Project {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`scope-to-plan.md`](scope-to-plan.md) | a project is proposed and someone is about to start work | a scope statement with a done test, an out-of-scope list, and a milestone plan with a named critical path |
| [`status-and-slip.md`](status-and-slip.md) | the reporting cadence, or the day a slip becomes known | a status report stated as position against plan, with the slip and the next decision due |
| [`change-request.md`](change-request.md) | anything is asked for that is not in the agreed scope | a priced change record and a recorded decision to accept, reject or defer it |
| [`landing.md`](landing.md) | the done test appears to be met, or the project must stop | acceptance, a closeout record, and the estimate correction for the next project |
