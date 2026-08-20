# Quality & Evaluation {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, its ordered steps, and the completion test
that decides whether it is done.

| Workflow | Trigger | Produces |
|---|---|---|
| [certify-a-build.md](certify-a-build.md) | Builder {OS} finalised a build artifact | the traceability matrix, executed evidence and the quality verdict |
| [evaluate-ai-behaviour.md](evaluate-ai-behaviour.md) | the product contains model-driven behaviour, or the model or prompt changed | scored evaluations over a stored dataset |
| [triage-defects.md](triage-defects.md) | findings exist and someone must decide what blocks | a ranked defect ledger with owners and acceptance authorities |

A workflow is finished when its completion test passes, not when its last step
has been performed.
