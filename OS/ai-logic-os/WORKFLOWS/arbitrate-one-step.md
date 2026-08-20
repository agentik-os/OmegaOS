# Workflow: Arbitrate one step

Put a single named step in exactly one bin, and say what it would cost to be
wrong.

## Trigger

Any of:

- Someone proposes to add a model call to a process.
- An existing model call is slow, expensive or inconsistent and you suspect a
  rule would do the job.
- Agent {OS} or Automation {OS} requests an arbitration before writing a brief
  or a blueprint.
- A consequential output is being trusted and nobody can name how it is checked.

## Steps

1. **Name the step precisely.** One input, one output, one actor. If the
   description covers more than one decision, split it and run this workflow per
   decision. A step two readers would scope differently is not yet a step.
2. **Describe the input.** Is it structured (fields, enums, numbers) or not
   (free prose, an image, an intent)? Structured input is the strongest single
   signal that a rule is sufficient.
3. **Describe the decision.** Can it be written as conditions over the input?
   Write the conditions out. If you can write them, the model call is already
   redundant.
4. **State the consequence of being wrong.** Who is affected, how quickly it is
   noticed, and what it costs to undo. Record whether the action is reversible.
5. **Classify into exactly one bin.**
   - *Codify* when the input is structurable and the decision is expressible as
     rules.
   - *Augment* when the input is genuinely unstructured or the decision needs
     judgment, and the output can be checked quickly.
   - *Keep human* when the action is irreversible, high consequence, or requires
     accountability that a system cannot carry.
   - *Delete* when the step compensates for a defect elsewhere, or nobody
     consumes its output.
6. **If the bin is augment, name the falsifier.** A deterministic assertion, a
   schema the output must satisfy, a source the output must cite, or a human who
   can reject it in under ten seconds. If no falsifier exists, the step is
   re-binned to keep human or to codify. There is no third option.
7. **If the action is irreversible, confirm the human gate.** A gate may be
   removed only against execution statistics that are shown, not asserted.
8. **Cost the two versions.** Deterministic: build hours plus maintenance.
   Model: per call cost times volume, plus latency, plus the variance cost of
   being wrong at the observed error rate. Show both.
9. **Write the verdict.** One step, one bin, one line of reason, the falsifier,
   and the arithmetic. Stage it to Context & Memory {OS}.
10. **Write what you do not recommend.** For this step, the alternative that was
    rejected and the number that would make you reconsider.

## Completion test

The workflow is complete when all of the following hold:

- The step sits in exactly one bin, and the bin has a written justification.
- If the bin is augment, a named falsifier exists and is concrete enough to
  implement.
- If the action is irreversible, either a human gate is confirmed or execution
  statistics are shown.
- Both cost sides are written down with their inputs visible.
- The verdict is staged to Context & Memory {OS} with its baseline attached.
- The section naming what is not recommended is non empty.

If any of these is missing, the arbitration is not finished and no downstream OS
may treat the step as decided.
