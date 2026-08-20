# Workflow: the reading cycle

Produces the period's values with movement separated from noise, breaches routed
to their decision owners, and gaps reported as gaps.

## Trigger

The measurement cadence fires. Also runs on demand when a decision owner needs a
current value before a decision.

## Inputs

- The metric definitions in force, with their versions.
- The source systems.
- The history and the normal variation for each metric.
- Any definition changes since the last cycle.

## Steps

1. **Compute from the definitions,** never from a hand-maintained sheet. A number
   somebody typed cannot be reproduced and will eventually be wrong.
2. **Record the definition version** in force for each value. A reading without
   its definition version cannot be compared to anything later.
3. **Report gaps as gaps.** A metric that could not be computed is reported as
   missing, with the reason. It is never interpolated and never drawn as a
   continuous line.
4. **Compare each movement to normal variation** before writing a single word of
   interpretation. Most weekly movements are inside the noise band.
5. **Say so when it is noise.** Naming noise as noise is the single most valuable
   output of this workflow, and the one most often skipped.
6. **For real movements, state what is known and what is not.** An unexplained
   movement is reported as unexplained. Inventing a cause is worse than
   admitting the gap, because the invented cause gets acted on.
7. **List the threshold breaches** with the pre-agreed decision and the owner of
   that decision.
8. **Route the breaches** the same day: to Operations {OS}, Project {OS}, Client
   {OS} or Revenue {OS} as appropriate, and to Review & Governance {OS} for the
   review pack.
9. **Check for definition drift.** If a source system changed, the number may
   have changed meaning without moving. Flag it.
10. **Publish the pack:** values, movements, noise calls, breaches, gaps. Short
    enough to be read in full.

## Completion test

- Every metric has a value, or an explicit gap with a reason.
- Every value carries the definition version that produced it.
- Every movement is labelled as inside or outside normal variation.
- Unexplained movements are labelled unexplained rather than narrated.
- Every breach names its pre-agreed decision and its owner, and was routed the
  same day.
- No number in the pack was entered by hand.

## Failure paths

| Situation | Response |
|---|---|
| a source was unavailable | report the gap; do not substitute last period's value |
| a breach's pre-agreed decision is now unwelcome | present it as agreed, and record any renegotiation of the threshold as an event |
| several metrics moved together | check for a shared cause, usually a definition or source change, before writing several separate explanations |
| someone requests an explanation you do not have | say what is not known and what would resolve it; an invented cause will be acted on |
