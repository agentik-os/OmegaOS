# Workflow: Repair a failing step

**Mode:** `REPAIR`
**Produces:** a step that passes its own checks, or a bounded escalation to a
human with the accumulated evidence.

## Trigger

`omega-stepper done <STEP-ID>` failed, or `verify` shows a check that will not
pass. Also triggered when `resume` returns an interrupted attempt to FAILED.

## Preconditions

- The step is claimed and its attempt is open.
- `execution.max_fix_attempts` is set. Repair without a ceiling is thrash with
  a nicer name.

## Steps

1. **Read the real output.** The verifier prints the failing check and the
   command output. Read that, not the summary of it.
2. **Name the failure.** Which check, which assertion, which line. A repair
   aimed at "the tests are red" fixes the wrong thing.
3. **Ask whether the contract is wrong.** Three outcomes: the implementation is
   wrong (repair), the check is wrong (that is a contract change and needs
   approval), or the upstream definition is wrong (block and escalate). Deciding
   this before editing is what keeps the ceiling meaningful.
4. **Repair against the evidence, not against a theory.** One change aimed at
   the named failure. Changing three things at once means the next failure
   teaches you nothing.
5. **Re-verify.** `verify <STEP-ID>`. Pass means close with `done`. Fail means
   the attempt count rises.
6. **Watch the ceiling.** Each failed attempt counts. The count is not refunded
   by a partial improvement.
7. **Before the third attempt on the same failure, get runtime evidence.** Not
   another guess: a log, a real response, a reproduction. Repeated blind fixes
   are how a step consumes a whole session.
8. **Escalate at the ceiling.** Stop repairing. Hand the human the step, every
   attempt, the failing output each time, and what you believe the real cause
   is. Do not raise the ceiling on your own.
9. **Record either way.** `done` on a pass, `fail` on a give-up, so the event
   log carries the truth and the planner can re-offer the step.

## Completion test

```bash
omega-stepper verify <STEP-ID>     # every check passes
omega-stepper done <STEP-ID>       # closes the step
```

Or, on escalation: the step is in FAILED or BLOCKED, the attempt count has
reached the ceiling, and an escalation record exists naming the step, each
attempt, the failing evidence and the suspected cause. An escalation with no
evidence attached fails this test.

## Failure paths

| What happens | What the workflow does |
|---|---|
| the check itself is wrong | stop repairing the code, raise a contract change for approval; never weaken a check to make a step pass |
| the failure is in another step's output | block this step on that one, do not fix the other step's code from inside this contract |
| the fix would touch a do not touch path | stop; either the scope is wrong or the plan is; raise it rather than widening the diff |
| the ceiling is reached and nobody is watching | write the escalation record, push it through the alert path, and leave the step FAILED rather than looping |
| the same step fails again after a human unblocks it | treat the contract as suspect, not the implementation |
