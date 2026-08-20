---
name: quality-evaluation-os
description: Independent certification of what was built, before it ships. Quality & Evaluation {OS}, unit 25 of the AGENTIK {OS} suite (03 · BUILD). Use when the user asks about quality & evaluation or invokes /quality-evaluation-os.
---

# Quality & Evaluation {OS}

Independent certification of what was built, before it ships.

## When to use this

Use Quality & Evaluation {OS} when:

- Builder {OS} has finalised and someone has to say whether the build actually
  conforms to what was defined;
- a product contains AI behaviour that needs scoring over a dataset rather than
  a demo;
- a release is being argued about and nobody can say what is proven;
- defects are appearing in production and the test coverage is unknown;
- an accessibility or performance contract exists and nobody has tested the
  built product against it.

Do not use it when:

- the question is whether the product can be attacked, whether its dependencies
  are trustworthy, or whether it handles personal data lawfully. That is
  Security {OS}.
- the question is whether to ship, how to roll out, or how to roll back. That
  is Release {OS}.
- the fix itself is what is needed. That is Builder {OS}, through a Stepper
  step.

The near neighbour people confuse it with is Builder {OS}'s own gates. Builder
proves its steps did what their contracts said. Quality proves the product does
what the product was defined to do, and it does that independently, because a
grader with an interest in the result is not a grader.

## Capabilities

- Builds the bidirectional requirement-to-evidence traceability matrix.
- Produces a risk-based test and evaluation plan ordered by consequence and
  uncertainty, not by convenience.
- Designs and runs functional, contract, integration, regression, exploratory,
  performance and data-migration tests.
- Tests the built product against the accessibility contracts Design {OS}
  wrote.
- Designs and runs AI evaluations: task success, groundedness, hallucination
  rate, refusal correctness, and regression when a model or prompt changes,
  each scored over a stored dataset.
- Triages defects with severity, impact, reproduction, workaround and owner.
- Issues a quality verdict that names its residual risk and its uncovered
  surface.

## Procedure

1. **Intake.** Pin the build artifact, the Blueprint requirements, the Design
   contracts and the Stepper plan verdict. An unpinned input is not certifiable.
2. **Trace.** Map every critical requirement to the evidence that would prove
   it. Gaps found here are the cheapest gaps of the whole chain.
3. **Model the risk.** What is high consequence, what is high uncertainty, what
   is regulated. That ordering is the test plan.
4. **Plan.** One planned test or evaluation per mapped requirement, with the
   environment and the data it needs. Where real data is required, stop and ask.
5. **Execute.** Run the plan. Record the command, the environment and the real
   output. A refused or unreachable surface is recorded as blocked, never as
   passing.
6. **Evaluate the AI surfaces.** Score over a dataset, store the dataset, and
   record the model and prompt version the score belongs to.
7. **Triage.** Every finding becomes a defect with severity, impact,
   reproduction, workaround and owner, or an observation explicitly labelled as
   not a defect.
8. **Return defects to Builder {OS}** as Stepper steps, so each fix carries a
   contract and produces evidence.
9. **Rule.** Issue `CONFORMS`, `CONFORMS WITH KNOWN DEFECTS` (each with an
   acceptance authority) or `DOES NOT CONFORM`, always naming what was not
   covered.
10. **Hand to Security {OS}** with the verdict and the evidence.

## Handoffs

| Receives from | What arrives |
|---|---|
| Builder {OS} (24) | the build artifact, the evidence ledger, BG01 to BG20 results, step to requirement traceability |
| Blueprint {OS} (20) | requirements and acceptance criteria, pinned |
| Design {OS} (21) | surface, state and accessibility contracts, and the `EVAL-###` cases |
| Stepper {OS} (23) | the plan-completion verdict, as one input among several |

| Hands to | What it expects |
|---|---|
| Security {OS} (26) | the quality verdict plus its evidence, so the security assessment starts from a known-conformant build |
| Builder {OS} (24) | defects, as Stepper steps, never as informal fixes |
| Blueprint {OS} (20) | decision requests where a requirement has no testable criterion |
| Release {OS} (27), through Security | the verdict, which Release weighs but does not overrule quietly |
