# Workflow: the novice test

Produces the only evidence that matters about an SOP: someone who had never done
the task produced an acceptable result with it, unaided.

## Trigger

An SOP draft exists and somebody believes it is finished. That belief is the
trigger, not the evidence.

## Inputs

- The draft.
- A person who has not done this task and did not write the SOP.
- A real instance of the work, or a realistic one.
- The quality bar and the person who judges it.

## Steps

1. **Brief the tester on the rules, not the task.** They follow the document.
   They may not ask the author. If they get stuck, they record the stall and
   continue as best they can.
2. **Have the author present and silent.** Watching a novice fail against your
   own document is uncomfortable and is the entire value of the exercise.
3. **Record every stall.** Where they paused, what they re-read, what they
   asked, what they guessed, and where they did the wrong thing confidently.
   Timestamp each one.
4. **Record the prerequisites they lacked.** Access, permission, a tool, a
   vocabulary word. These are the most common and the most invisible defects.
5. **Let them finish.** A test stopped at the first stall only finds one defect.
6. **Judge the output against the quality bar,** by the person who normally
   judges it, without knowing which run was the test.
7. **Classify each stall:** missing step, ambiguous step, missing decision
   criterion, missing prerequisite, wrong order, or a genuine process defect
   that no writing can fix.
8. **Fix at the cause.** One change per stall. A stall fixed by adding a warning
   sentence is usually a step that should have been split.
9. **Send the process defects to Operations {OS}.** If the SOP cannot be written
   clearly because the work itself is tangled, that is the finding.
10. **Retest if the fixes were substantial,** ideally with a different novice. The
    first tester now knows the task and can no longer test it.

## Completion test

- A person who had never done the task attempted it using only the document.
- The output was judged against the quality bar by the normal judge.
- Every stall is recorded, classified and either fixed or explicitly accepted.
- Prerequisites that were missing have been added.
- Process defects have been routed to Operations {OS} rather than papered over.
- If fixes were substantial, a second test was run with a different person.

## Failure paths

| Situation | Response |
|---|---|
| no novice is available | mark the SOP untested and say so at the top of the document; do not release it as validated |
| the novice asks the author and the author answers | the test is void from that point; record the question as a defect and rerun |
| the novice succeeds but takes four times the estimate | the SOP works and the time estimate is wrong; fix the estimate, since everyone plans with it |
| the output fails the quality bar and the novice followed every step | the defect is in the SOP, not the person; find the step where the result diverged |
