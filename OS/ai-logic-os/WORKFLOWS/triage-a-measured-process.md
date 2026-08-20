# Workflow: Triage a measured process

Bin every step of a process, announce the deletions first, and leave with
exactly one move specified.

## Trigger

- Operations {OS} has mapped and simplified a process and someone now wants it
  automated.
- A process has grown steps nobody can justify.
- Automation {OS} asks which steps of a candidate are rules and which are
  judgment.

## Steps

1. **Refuse without a baseline.** Volume per period, time per unit, error rate,
   cost. If any is missing, stop here, deliver a measurement device
   specification, and say plainly that triage cannot run yet.
2. **Confirm the process was simplified.** If steps exist only because something
   upstream is broken, hand back to Operations {OS}. Automating a broken process
   makes it permanent.
3. **Number the steps** with an owner and a duration each, exceptions included.
4. **Bin each step** into codify, augment, keep human or delete, using the
   arbitration workflow for any step that is contested.
5. **Announce the delete bin first,** with the time it recovers. This is the
   most profitable bin and it is the one that is skipped when additions lead.
6. **For each augment step, name its falsifier.** A step that cannot be checked
   does not stay in augment.
7. **For each keep human step, confirm the gate** and record why the step
   remains human, so the reason can be revisited later against statistics.
8. **Score the remaining moves**, costliest gap first, showing the score inputs.
9. **Do the arithmetic on the top move**: annual gain against build plus
   maintenance. If it does not clear, say no and show the numbers.
10. **Specify the first move only** and route it to Automation {OS}, with the
    baseline attached so the improvement can be checked later.
11. **Write what you do not recommend**, including every step someone wanted
    automated and should not.

## Completion test

- Every step sits in exactly one bin with one line of justification.
- The delete bin is reported before the others, with the time it recovers.
- Every augment step has a named falsifier.
- The top move has visible arithmetic and a verdict that follows it.
- Exactly one move is specified and handed to Automation {OS} with its baseline.
- The not recommended section is non empty.

The process is not "automated" at the end of this workflow. One move is
specified. That is the intended output, and it is deliberate.
