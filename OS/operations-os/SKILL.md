---
name: operations-os
description: Process diagnosis and work simplification before automation. Operations {OS}, unit 45 of the AGENTIK {OS} suite (05 · OPERATE). Use when the user asks about operations or invokes /operations-os.
---

# Operations {OS}

Find out how the work is really done, measure it, remove what should not exist,
simplify what remains, and only then decide whether it should be standardised,
delegated or automated.

## When to use this

Use it for work that repeats and hurts: something takes too long, the same
error keeps happening, three people touch a thing that one person could, or
somebody is about to buy a tool to speed up a process nobody has examined.

Typical openings: this takes us four hours every week, we keep making the same
mistake, can we automate our onboarding, why does this always get stuck, we
need a tool for this.

The last two are the important ones. The correct first answer to "can we
automate this" is a diagnosis, not a build.

Near neighbours it is confused with:

| If the real need is | The right OS is |
|---|---|
| building, deploying or monitoring the automation | Automation {OS} |
| writing the procedure so anyone can run it | Process & SOP {OS} |
| handing the work to a specific person | Team & Delegation {OS} |
| one-off work with a start and an end | Project {OS} |
| the numbers that drive business decisions | KPI & Analytics {OS} |
| approving the removal of a control | Review & Governance {OS} |

## Capabilities

- Set a process boundary that the people who run it agree on.
- Interview every role that touches the work, including for the workarounds
  nobody documents.
- Observe a real run end to end and time it, rather than accepting the described
  version.
- Build a current-state map with steps, handoffs, waits, decisions and rework
  loops, and have it recognised as accurate by the people in it.
- Measure frequency, touch time, wait time, error rate, rework rate and cost per
  run, marking unknowns as unknown.
- Name waste and control gaps with evidence, and separate the two.
- Run the ladder in order: eliminate, simplify, standardise, delegate, automate.
- Design the target operating model with its controls and exception paths.
- Issue an automation readiness verdict, and assemble the handoff packet for
  Automation {OS}.

## Procedure

1. Scope the process: first trigger, last output, roles, and what is out of
   scope. Get agreement on the boundary before anything else.
2. Interview each role. Ask what they actually do, not what the document says.
3. Observe at least one real run, with timings and with consent.
4. Map the current state, including waits and rework loops. Show it back to the
   people in it and correct it until they recognise it.
5. Measure each step. Mark unknowns rather than estimating.
6. Enumerate the exceptions and their rate. If that rate is unknown, measure it
   before continuing.
7. Run the ladder. For every step ask, in order: can it be removed, can it be
   made simpler, does it need to be written down, should someone else do it, and
   only then, should a machine do it.
8. Separate waste from controls. Controls route to Review & Governance {OS}.
9. Design the target operating model and check it is reachable from today.
10. Issue the readiness verdict and hand the packet to the right next OS. Not
    ready is a legitimate verdict.

## Handoffs

| Send to | What | What they expect |
|---|---|---|
| Process & SOP {OS} | the simplified process | the steps, decision points, tools and the quality bar |
| Team & Delegation {OS} | work that should move to a person | outcome, definition of done, authority level |
| Automation {OS} | only what passed readiness | the map, the measures, the exception list, the controls, the volumes and the failure modes |
| KPI & Analytics {OS} | measures worth tracking beyond the diagnosis | definition, source, owner, cadence |
| Documentation {OS} | current-state and target maps | the artifact, its owner and its review date |
| Review & Governance {OS} | control gaps and control removals | evidence and the decision requested |
