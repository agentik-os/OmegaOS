# Validation {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [claim-register.md](claim-register.md) | a plan, deck, concept or blueprint exists and its assumptions are implicit | the ranked claim register, every claim falsifiable and owned |
| [signed-test-spec.md](signed-test-spec.md) | one claim is selected for testing | an immutable test spec with a threshold signed before any data exists |
| [verdict-record.md](verdict-record.md) | a signed test has been run | the verdict, the kill note if it died, and the propagation to downstream OS units |
| [validation-audit.md](validation-audit.md) | an inherited artifact uses the word "validated" | a per-claim report of what was measured versus what is asserted |
