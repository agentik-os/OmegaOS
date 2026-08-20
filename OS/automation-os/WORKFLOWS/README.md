# Automation {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, its ordered steps, and its completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`score-an-automation-candidate.md`](score-an-automation-candidate.md) | Operations {OS} approved a simplified map and someone wants it automated | scored candidates with visible arithmetic, or a refusal naming what is missing |
| [`deploy-a-governed-automation.md`](deploy-a-governed-automation.md) | a candidate cleared its score and its arithmetic | a live automation with controls, a runbook, observability and a named owner |
| [`contain-and-recover-an-incident.md`](contain-and-recover-an-incident.md) | a run failed, or a green run produced the wrong effect | containment, a safe recovery, and the name of the control that was missing |

One invariant cuts across all three: no approved simplified process map, no
automation work of any kind.
