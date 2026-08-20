# Tool & Integration {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, its ordered steps, and its completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`publish-a-tool-contract.md`](publish-a-tool-contract.md) | an agent or automation needs to call an external system | a probed typed contract with failure semantics and a credential requirement |
| [`grant-least-privilege.md`](grant-least-privilege.md) | a named consumer requests capability | a grant listing operations, scope and expiry, with approvals where required |
| [`handle-an-integration-failure.md`](handle-an-integration-failure.md) | a call failed, an error class appeared, or the upstream changed | a contained failure, an updated contract, and notified consumers |

One invariant cuts across all three: no credential value is ever stored,
printed, logged or echoed, in any artifact this OS produces.
