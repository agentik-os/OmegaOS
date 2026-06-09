# R-COUNCIL — Convene the council on high-stakes & contested calls

**Kind:** Rule
**Category:** Orchestration
**Added:** 2026-06-09

## Rule

High-stakes, ambiguous, or irreversible decisions go to the COUNCIL (@council, /llm-council, /council) BEFORE acting. AUTO-convene on: irreversible operations (data loss, force-push, prod DB migration/drop), prod-wide or architecture-level changes, cross-project decisions, and contradictory adversarial-verification verdicts that do not cleanly resolve. On demand, any operator or agent may invoke it. The council runs MULTIPLE Claude models — Opus 4.8, Sonnet 4.6, Haiku 4.5, Fable 5 — in parallel on the same question, has them peer-review each other ANONYMOUSLY (blind to model identity), and an Opus president synthesizes a verdict with confidence and recorded dissent. 100% Claude Code-native via the Workflow primitive — no API keys, no external providers. Not for routine work (~4x tokens); reserve it for calls where several independent minds buy real safety.

## Origin

High-stakes / irreversible / contested calls made unilaterally by one model — or accepted on a single verification pass — drift and occasionally go catastrophically wrong; a multi-model Claude council with blind peer-review and an Opus president that records the dissent makes such decisions auditable and far harder to get wrong, at zero external API cost.
