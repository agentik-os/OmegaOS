# Workflow: the process diagnosis

Produces a current-state map the people who run the process recognise, with a
number or an explicit unknown against every step.

## Trigger

A repeating process is expensive, slow or error-prone, or somebody is about to
buy or build a tool to speed it up. The second trigger is the more urgent: a
tool bought before the diagnosis usually preserves the waste in software.

## Inputs

- The people who touch the process, including the ones who do it differently
  from the documented way.
- Access to observe a real run, with their informed consent.
- Volumes over a representative period.
- Existing numbers from the systems involved, and from KPI & Analytics {OS}.

## Steps

1. **Agree the boundary.** First trigger, last output, roles in scope and out.
   Write it down and have it confirmed. Most process arguments are boundary
   arguments in disguise.
2. **Interview each role separately.** Ask what they do, what goes wrong, what
   they wait for, and what they do when the normal path does not work. Record
   the workarounds; they are the real process.
3. **Ask each person what they would remove.** They know. They are rarely asked.
4. **Observe a real run end to end**, with consent, and time each step and each
   wait. Do not help, do not correct, and do not accept a demonstration run
   staged for the observer.
5. **Keep interview notes and observation notes separate.** The difference
   between them is a finding, not noise to be averaged away.
6. **Draw the current-state map:** steps, who performs each, handoffs between
   people or systems, waits, decision points and rework loops.
7. **Attach numbers:** frequency, touch time, wait time, error rate, rework
   rate, and cost per run. Anything unmeasured is marked unknown, never
   estimated into the sheet.
8. **Enumerate the exceptions** and their rate. If nobody knows the rate, that is
   the first thing to measure.
9. **Show the map back.** If the people who run the process do not recognise it,
   the map is wrong. Correct it until they do.
10. **Publish the map and the measurement sheet** to Documentation {OS}, and send
    any control gap to Review & Governance {OS}.

## Completion test

- The boundary is written and confirmed by the participants.
- Every role that touches the process has been interviewed.
- At least one full run has been observed and timed.
- The map includes waits and rework loops, not only touch steps.
- Every step carries a number or is explicitly marked unknown.
- The exception list exists, with a rate or a stated plan to measure it.
- The people who run the process recognise the map as accurate.

## Failure paths

| Situation | Response |
|---|---|
| observation is refused | proceed on interviews only, label the map self-reported, and lower the confidence of every downstream decision |
| everyone describes a different process | that is the finding; there is no single process yet, and standardisation is premature |
| the process only runs monthly | observe the artifacts of past runs and interview against them, and say the observation is indirect |
| people fear the diagnosis is about their job | say plainly what it is for, never publish individual timings without consent, and route any staffing consequence to a human decision |
