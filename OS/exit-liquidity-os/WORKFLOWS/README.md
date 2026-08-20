# Exit & Liquidity {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [Exit readiness assessment](exit-readiness-assessment.md) | the owner asks whether the business is sellable, a buyer makes contact, or the last assessment is over twelve months old | a readiness score and a gap list classified paperwork, value or structural, each with an owner and a date. Emits `exit.readiness.scored` |
| [Data room assembly](data-room-assembly.md) | readiness is scored and preparation has started, or a counterparty sends a request list | the diligence readiness index (state, location and owner per line) and the assembled, unopened room. Emits `exit.dataroom.indexed` |
| [Structure preference and tax gate](structure-preference-and-tax-gate.md) | a real conversation is plausible within the timeline, or a counterparty signals interest | the written structure preference, the walk-away, and the tax and counsel question packs. Emits `exit.structure.proposed` |
| [Post-close obligations register](post-close-obligations-register.md) | a transaction has closed, or an obligation falls due | the register of everything that survives the close, each line with a date, an owner and a release condition. Emits `exit.obligation.tracked` |

They run in that order, and three of the four run best long before they feel
urgent. The readiness assessment is worth most two to three years ahead of the
window, and the walk-away inside the structure workflow only has value if it was
written before the first offer arrived.

None of these workflows sends anything to a counterparty, opens a data room to
an outside party, drafts or reviews a letter of intent or a purchase agreement,
or decides a tax treatment. Those are human actions taken with a legally
accountable lawyer, accountant or tax professional, and every release to an
outside party is approved per document and per recipient and written to the
disclosure log.
