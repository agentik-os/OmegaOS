# Workflow: Run a wave

**Modes:** `PLAN`, then `EXECUTE`
**Produces:** one safe wave of steps, each closed through the deterministic
verifier, with the event log to prove it.

## Trigger

A validated graph exists with a non-empty ready set, and there is capacity to
work. This is the everyday loop.

## Preconditions

- `omega-stepper resume` has run, so no interrupted attempt is being counted as
  live.
- The work-in-progress limit and `max_fix_attempts` are set in `stepper.yaml`.
- Where steps will be dispatched to workers, each worker can be given exactly
  one agent brief.

## Steps

1. **Recover state, never memory.** `resume`, then `status`, then `plan`. What
   the session remembers is not evidence.
2. **Take the wave the planner offers.** It already respects scope locks and
   the work-in-progress limit. Adding a step to it by hand is how two writers
   end up on one file.
3. **Claim each step.** `start <STEP-ID>`. A refusal names the blocking
   dependency, and the correct response is to work a different step, not to
   argue with the graph.
4. **Dispatch with the brief.** `agent-brief <STEP-ID>` for anything handed to
   a worker. The brief is self-contained on purpose: a worker reading the whole
   plan will optimise something outside its contract.
5. **Implement exactly the contract.** The do not touch block is as binding as
   the objective. Work discovered outside the contract becomes a new step, not
   a wider diff.
6. **Check before closing.** `verify <STEP-ID>` shows where you are without
   changing state.
7. **Close through the verifier.** `done <STEP-ID>`. PASS closes it. FAIL opens
   the repair workflow against the printed evidence.
8. **Record real reviews only.** `review <step> <role> PASS --by <name>` for a
   review that actually happened, by the person who did it.
9. **Re-plan after every close.** The ready set changes; the next wave is
   computed, not guessed.
10. **Report at the end of the session.** `report` and `events`, so the next
    session starts from evidence.

## Completion test

```bash
omega-stepper status          # every step of the wave shows DONE
omega-stepper events          # each close preceded by a passing verification
omega-stepper plan            # the next wave is computed and non-empty, or the plan is complete
```

A step that shows DONE with no passing verification event in the log fails this
test, and the step is reopened.

## Failure paths

| What happens | What the workflow does |
|---|---|
| two ready steps write the same files | the planner already serialised them; never hand-merge them into one wave |
| a worker reports success but `done` fails | the verifier wins, the step stays open, the repair loop starts |
| a step reveals work outside its contract | create a new step, keep the current diff inside the contract |
| an upstream decision is needed mid step | `block` the step, raise the decision request, keep the rest of the wave running |
| the session dies mid wave | next session opens with `resume`; interrupted attempts return to FAILED and are re-offered |
| everything is DONE at the target priority | run `release-check`, and hand the build to Builder {OS} completion and then to Quality & Evaluation {OS} |
