# Positioning {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [Claim ledger](claim-ledger.md) | a claim is proposed, a rival changes theirs, or a review expires one | a ledger entry with claim, evidence, exclusion, expiry, tester and date |
| [Category decision](category-decision.md) | no category on record, repeated distinctiveness failure, or a proposed category change | the category decision with its demand verdict and review condition |
| [Claim retirement](claim-retirement.md) | a claim expires, is contested, or its expiry condition fires | a retirement record with the killing evidence and every surface corrected |
