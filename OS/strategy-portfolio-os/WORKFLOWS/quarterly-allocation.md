# Workflow: Quarterly allocation

**Produces:** the quarterly strategic plan and the execution packet: a small
number of outcomes with owners, an allocation of time, attention, people and
capital that fits real capacity, leading and lagging signals, and the
exclusions, handed off to Execution {OS} and to the relevant product OS units.

## Trigger

A period is starting, or the previous one just closed. Also triggered when
`capital.reallocation.proposed` or `health.capacity.assessed` changes the
constraint set enough that the current allocation no longer fits.

Runs the quarterly strategy protocol.

## Steps

1. **Close the previous period first.** Pull the outcomes actually proven
   (`execution.outcome.proven`), the metrics as they landed, and every review
   trigger that fired. A new quarter planned before the last one is scored
   repeats the last one's mistakes with fresh optimism.
2. **Re-read the kernel and the standing exclusions.** If the diagnosis no
   longer describes the situation, stop and run the strategy kernel workflow.
   Quarterly planning is not the place to quietly change the strategy.
3. **Select a small number of strategic outcomes.** Each one an outcome, not an
   activity, with a horizon and the measures that would show it was reached.
   More outcomes than owners is the first symptom of a plan that will not
   survive week three.
4. **Assign an owner per outcome.** One named person. An outcome owned by
   everyone is owned by nobody, and it is the first to be quietly dropped.
5. **Total the real capacity.** Hours per week that genuinely exist after
   maintenance, people who genuinely exist, capital that can be committed without
   endangering reserves. Consume `health.capacity.assessed` and
   `capital.reallocation.proposed` rather than assuming.
6. **Allocate against that total.** Time, attention, people and capital per
   outcome, in real units, over the stated period. If the sum exceeds capacity,
   report the overcommitment and name what must be cut. Do not publish a plan
   that only fits if nothing goes wrong.
7. **Verify the allocation matches the stated priority.** The outcome called
   most important receives the most of the scarcest resource, or the ranking is
   wrong and is corrected here. This is the step that catches a strategy that
   exists only in the memo.
8. **Define the signals.** Per outcome: one leading signal, one lagging signal
   and one guardrail, each with its type and the decision it is allowed to
   affect. Remove any metric that cannot change a decision.
9. **Specify the exclusions.** What is explicitly not being done this period,
   with the reason and the reopening condition. Carry forward the standing
   not-doing list and mark what changed.
10. **Set the review triggers.** The observable events that bring an item back
    for a kill review before the period ends, not only the end-of-period date.
11. **Route approval and governance.** Capital commitments, people decisions and
    changes to an approved strategic objective go to the human approval
    boundary. Consequential allocation changes go out as
    `strategy.change.requested` and wait for `change.approved` before
    `allocation.changed` is emitted.
12. **Hand off.** Emit `strategy.execution_packet.created` to Execution {OS}
    with the outcomes, owners, allocation and exclusions. Emit
    `strategy.product_bet.approved` to Blueprint {OS} for any selected product
    bet, carrying its thesis, its constraints and what it must prove. Neither
    packet restates work the receiving OS owns.
13. **Record.** Write the objectives, allocations and metrics to canonical state
    through Context & Memory {OS}.

## Completion test

- Every outcome has one named owner, a horizon and its success measures.
- The allocation is stated in real units per resource type and per period, and
  the total does not exceed the capacity reported at step 5.
- The outcome named most important holds the largest share of the scarcest
  resource.
- Every outcome has a leading signal, a lagging signal and a guardrail, each
  naming the decision it may affect.
- The exclusions are written, each with a reason and a reopening condition.
- At least one review trigger exists that can fire before the period ends.
- The execution packet has been emitted and Execution {OS} needs no further
  restatement to begin.
- No `allocation.changed` was emitted for a consequential change before its
  `change.approved` returned.
