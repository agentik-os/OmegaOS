# Workflow: the weekly reset

Produces the week's honest truth, one win for next week, and one experiment on
the system itself.

## Trigger

The week ends. Also runs at the first boot of a new week if the previous week
was never reset.

## Inputs

- Every day record of the week: boot, halt, classification, friction.
- Every commitment closed, deferred, cancelled or delegated during the week.
- The promise ledger, including promises due next week.
- Last week's system experiment and its stated test.

## Steps

1. **Count, do not narrate.** Days booted, days halted, commitments closed with
   evidence, commitments closed without evidence, deferrals, cancellations.
2. **Classify the week** using the same five classifications as a day:
   `SHIPPED`, `VERIFIED`, `PROGRESSED`, `TOUCHED`, `ABANDONED`. State which
   evidence justifies the classification.
3. **Read last week's experiment.** It had a test. Run the test. Say kept,
   changed or dropped, with the reason.
4. **Name the friction.** The repeated one, not every one. If the same friction
   appears in three or more halt cards, it is the friction of the week.
5. **Look at deferrals.** Anything deferred three or more times gets a decision
   now: shrink it, delegate it, schedule it with a protected block, or cancel
   it. Silent carry-over is not an option.
6. **Check the promise ledger.** Anything due next week whose next action is
   not startable triggers the late-promise workflow now, not on the deadline.
7. **Choose next week's single win.** One outcome. Write its acceptance test.
8. **Choose one system experiment.** A change to how the system runs, not to
   what is on the list. Write the test that will tell you next week whether it
   worked.
9. **Send the reset upward.** The truth, the win and the experiment go to Review
   & Governance {OS} as evidence, not as a request for approval.

## Completion test

- The week has a classification with the evidence that justifies it.
- Last week's experiment has a verdict: kept, changed or dropped.
- Every commitment deferred three or more times has a decision recorded.
- Next week's single win exists and has an acceptance test.
- Exactly one system experiment is recorded, with the test that will judge it.

## Failure paths

| Situation | Response |
|---|---|
| the week has fewer than three halt cards | say the data is thin, classify on what exists, and make the experiment about closing days |
| the user wants to add three experiments | keep one, park the others, and say why more than one makes the test unreadable |
| the truth is unflattering | record it plainly, without softening and without a verdict about the person |
