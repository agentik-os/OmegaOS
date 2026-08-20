# Workflow: the automation readiness verdict

Produces a stated verdict on whether a simplified process should be automated,
and the packet Automation {OS} needs if it should.

This workflow is the boundary between Operations {OS} and Automation {OS}. It
ends with a handoff. Nothing is built here.

## Trigger

A target operating model exists and somebody wants the process automated. Also
runs when an automation is being proposed for a process that has never been
diagnosed, in which case it sends the request back to the diagnosis workflow.

## Inputs

- The target operating model with its controls and exception paths.
- The measurement sheet: frequency, touch time, error rate, cost per run.
- The exception list and its measured rate.
- The failure consequences: what happens when a run goes wrong, and who notices.
- Compliance and contractual constraints.

## Steps

1. **Confirm the ladder was climbed.** Eliminated and simplified first. If not,
   send it back; automating an unexamined process is the failure this OS exists
   to prevent.
2. **Test the value.** Frequency multiplied by time saved multiplied by the
   error reduction. A process that runs twice a year is rarely worth automating
   however annoying it is.
3. **Test the stability.** A process whose steps changed twice in the last
   quarter will change again, and the automation will be maintained more often
   than it is used.
4. **Test the exception rate.** A high or unknown exception rate means the
   automation will hand back to a human constantly, which is a worse process
   than the manual one.
5. **Test the inputs.** Automation needs inputs that are structured and
   predictable. Free-text inputs from humans are the most common cause of an
   automation that works in the demonstration and not in the month.
6. **Test the consequence of a silent failure.** If a wrong run is invisible,
   automation multiplies the error rather than the throughput. Name the
   detection path or refuse.
7. **Test the controls.** Every control in the target model must survive
   automation, including the human approvals. Anything that would remove a
   control goes to Review & Governance {OS} first.
8. **State the verdict:** ready, not ready, or ready for part. Ready for part is
   the most common honest answer: the predictable path is automatable and the
   exception path stays human.
9. **If ready, assemble the handoff packet:** the map, the measures, the
   exception list, the controls and approvals, the volumes, the failure modes
   and their detection, and the value estimate with its assumptions.
10. **Hand off to Automation {OS}** and stop. Design, build, deploy, monitor and
    incident recovery all belong there.

## Completion test

- The verdict is recorded as ready, not ready, or ready for part, with reasons.
- Value, stability, exception rate, input quality, silent-failure detection and
  controls were each tested and answered.
- No control was dropped without a Review & Governance {OS} decision.
- If ready, the handoff packet is complete and Automation {OS} has accepted it.
- If not ready, the reason is written and the alternative rung of the ladder is
  named: standardise, delegate, or leave it alone.

## Failure paths

| Situation | Response |
|---|---|
| the process was never diagnosed | refuse the verdict and route to the diagnosis workflow |
| someone has already built the automation | diagnose anyway, and give Automation {OS} the exception list and controls it is missing |
| the value case depends on unmeasured numbers | say the case is unproven, name the two numbers that would settle it, and measure them |
| a control would be lost | not ready, and route the control question to Review & Governance {OS} before any build begins |
