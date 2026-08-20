# Workflow: Answer the riskiest question

**Modes:** `TRIAGE`, then `FLOW`, `FAKE` or `BENCH`, closing with `TEARDOWN`
**Produces:** one falsifiable question, the cheapest artifact that answers it,
the evidence, and a verdict that names the upstream records it settles.

## Trigger

A decision is blocked and both options look defensible on paper. Typically:
Blueprint {OS} holds an ASSUMPTION with a high cost of being wrong, or Design
{OS} produced two shells or two flows nobody can choose between.

## Preconditions

- The decision owner is identified and available to agree a threshold.
- A budget ceiling exists: hours, money, and an expiry date.
- Where participants are needed, they can be recruited within that ceiling.

## Steps

1. **List the open questions.** From Blueprint ASSUMPTION and UNKNOWN records,
   Design decisions with reversal triggers, and anything the team keeps
   relitigating.
2. **Cost both sides.** For each question: what it costs to be wrong, and what
   it costs to find out. Rank by the ratio, not by curiosity.
3. **Select one and say what waits.** Testing two questions at once means you
   cannot attribute the result.
4. **Make it falsifiable.** Rewrite until a result could disprove it. "Is the
   navigation good" is not testable. "Can a first-time user reach the second
   panel without help in under 30 seconds" is.
5. **Agree the threshold in writing.** With the decision owner, before the
   artifact exists. Record it in the verdict file up front.
6. **Choose the cheapest sufficient method.** Paper, clickable, wizard of oz,
   concierge, smoke test or bench. Justify every escalation past paper.
7. **Set the ceiling and the expiry.** Both written on the artifact itself.
8. **Build only the risky part.** Fake, stub or hand-run everything else.
9. **Run the same protocol every time.** Fixed script, fixed tasks, no
   coaching. Record raw observations, including the ones that are inconvenient.
10. **Rule against the threshold.** `CONFIRMED`, `REFUTED` or `INCONCLUSIVE`.
    Do not reinterpret the threshold after seeing the data.
11. **Route the verdict.** Stepper {OS} when the plan is unblocked, Blueprint
    {OS} as a decision request when product truth is contradicted, Design {OS}
    as a flow challenge when an interaction is refuted.
12. **Tear down.** Run the teardown workflow. Always.

## Completion test

By inspection of `prototypes/<id>/verdict.json`:

- the question is stated in a form a result could disprove;
- the threshold is recorded with a timestamp earlier than the first observation;
- the raw evidence is attached, not summarised away;
- the verdict is one of `CONFIRMED`, `REFUTED`, `INCONCLUSIVE`;
- at least one upstream record ID is named as settled or reopened;
- the teardown record exists and the artifact path no longer exists.

A verdict whose threshold timestamp is later than its evidence fails this test.

## Failure paths

| What happens | What the workflow does |
|---|---|
| the decision owner will not commit to a threshold | stop before building, escalate, record that the decision is being made on preference |
| the evidence lands between two outcomes | report `INCONCLUSIVE` with what would have resolved it, propose the next cheapest step |
| the artifact turns out to cost more than the decision | abandon it, record the assumption explicitly in Blueprint {OS}, and move on |
| someone asks to ship the artifact | refuse, name what a shippable version needs, hand the question to Stepper {OS} |
| participants cannot be recruited in the ceiling | switch to the next cheapest method or report the question as unanswerable at this budget |
