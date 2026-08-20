---
name: prototype-os
description: The cheapest artifact that answers the riskiest open question. Prototype {OS}, unit 22 of the AGENTIK {OS} suite (03 · BUILD). Use when the user asks about prototype or invokes /prototype-os.
---

# Prototype {OS}

The cheapest artifact that answers the riskiest open question.

## When to use this

Use Prototype {OS} when:

- a decision is blocked and both options look defensible on paper;
- Blueprint {OS} carries an ASSUMPTION whose cost of being wrong is high;
- Design {OS} produced a `DDEC` with a reversal trigger nobody can evaluate
  from a spec;
- a technical approach is unproven and the plan depends on it;
- someone is about to commit weeks of build to a demand nobody has observed.

Do not use it when:

- the question is what the product is. That is Blueprint {OS}.
- the question is what the interface should be, and a contract can settle it.
  That is Design {OS}.
- the answer is already known and someone wants a demo. A demo is a marketing
  artifact and belongs to the GROW group.
- the artifact is intended to survive. That is Builder {OS}, and it needs a
  Stepper plan first.

The near neighbour people confuse it with is Builder {OS}. Builder produces
code that must live. Prototype produces evidence and destroys the code. Merging
the two produces the single most expensive object in software: a prototype in
production.

## Capabilities

- Ranks open questions by cost of being wrong divided by cost of answering, and
  selects one.
- Rewrites a vague question into a falsifiable one with a threshold agreed
  before anything is built.
- Chooses the cheapest sufficient method: paper, clickable, wizard of oz,
  concierge, smoke test, technical spike, measurement bench.
- Builds the disposable artifact, scoped to the risky part only.
- Runs the test protocol, including with real participants, and records raw
  observations rather than impressions.
- Issues a verdict: `CONFIRMED`, `REFUTED` or `INCONCLUSIVE`, pointing at the
  upstream record it settles or reopens.
- Tears the artifact down and records that it is gone.

## Procedure

1. **Triage.** List the open questions. For each: what it would cost to be
   wrong, and what it would cost to find out. Select one. State why the others
   wait.
2. **Make it falsifiable.** Write the question so that a result could disprove
   it. Agree the threshold now, in writing, with whoever owns the decision.
3. **Pick the cheapest method that can answer it.** Escalate from paper only
   when the cheaper method cannot produce the evidence, and say why.
4. **Set the ceiling.** Budget, time and expiry date, before the first line or
   the first sketch.
5. **Build only the risky part.** Everything else is faked, stubbed or done by
   hand. Polish is waste here, and it is also a trap: a polished prototype gets
   promoted.
6. **Run the protocol.** Same script for every participant or every run.
   Observe behaviour, record raw data, resist explaining the result while it is
   still being produced.
7. **Rule.** Compare the evidence to the pre-agreed threshold. Write
   `CONFIRMED`, `REFUTED` or `INCONCLUSIVE`, and name the upstream record IDs
   affected.
8. **Tear down.** Delete the artifact, revoke anything it was given, record the
   teardown.
9. **Route.** Verdict to Stepper {OS} when it unblocks the plan, to Blueprint
   {OS} as a decision request when it refutes product truth, to Design {OS} as
   a flow challenge when it refutes an interaction.

## Handoffs

| Receives from | What arrives |
|---|---|
| Blueprint {OS} (20) | ASSUMPTION and UNKNOWN records with the cost of being wrong |
| Design {OS} (21) | design decisions carrying a reversal trigger, undecidable flows |
| Validation {OS} | an open demand question that a smoke test could settle |

| Hands to | What it expects |
|---|---|
| Stepper {OS} (23) | the verdict attached to the decisions it settles, so the plan stops carrying the risk |
| Blueprint {OS} (20), on `REFUTED` | a decision request naming the record the evidence contradicts |
| Design {OS} (21), on `REFUTED` | a flow challenge with the observations attached |

Nothing this OS builds is handed to Builder {OS}. Only the verdict travels.
