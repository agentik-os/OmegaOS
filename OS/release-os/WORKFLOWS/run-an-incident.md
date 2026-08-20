# Workflow: Run an incident

**Mode:** `INCIDENT`
**Produces:** contained impact, an incident record written as it happened, and
a postmortem routed to Review & Governance {OS}.

## Trigger

Production is degraded: an alert fired, verification failed, a customer
reported it, or a rollback did not restore service. The cause does not matter
at the start; the impact does.

## Preconditions

- Someone is the incident commander. One person, named, at all times.
- The observability contract exists, so the signals being read mean something.
- The rollback plan is at hand.

## Steps

1. **Declare it.** An unnamed incident is handled by everyone and therefore by
   nobody. Declaring costs nothing and undeclaring is easy.
2. **Name the commander.** One person owns decisions and the timeline. Everyone
   else reports to that person, including anyone more senior.
3. **State the impact before the cause.** Who is affected, doing what, since
   when. The cause is interesting; the impact is what decides the next action.
4. **Contain first.** Roll back, disable the feature flag, shed load, block the
   path, fail over. Diagnosis comes after the bleeding stops, and it is far
   easier once it has.
5. **Record as you go.** Every observation, decision, action and time, written
   in the moment. A timeline reconstructed afterwards is shorter and wrong in
   the places that matter.
6. **Communicate on a cadence.** A fixed interval to stakeholders, even when
   the update is that nothing changed. Silence gets filled with worse
   information.
7. **Diagnose once contained.** Change one thing at a time. Under pressure this
   is the discipline that goes first, and it is what turns a one hour incident
   into a six hour one.
8. **Decide fix forward or stay rolled back.** Explicitly, with the commander
   owning it and a time box on any forward attempt.
9. **Verify recovery.** Exercise the golden path, read the signals, and confirm
   with a real user path rather than a health endpoint.
10. **Close with the residue named.** What is still inconsistent, what was lost,
    what customers saw. Closing is an approval boundary.
11. **Route the postmortem.** To Review & Governance {OS}, blameless, with the
    contributing factors and the actions, each with an owner. Defects go to
    Builder {OS} as Stepper steps.

## Completion test

By inspection of the incident record:

- a single commander is named for every phase;
- the impact statement precedes the cause analysis and states who, what and
  since when;
- the timeline carries times, and was written during rather than after;
- containment is recorded before diagnosis;
- recovery is verified on a real user path, not on a health endpoint alone;
- the residue is stated, including what was lost;
- every action item has an owner, and the postmortem is routed.

An incident record whose entire timeline was written after resolution fails
this test, and the record says so rather than pretending otherwise.

## Failure paths

| What happens | What the workflow does |
|---|---|
| the cause is unknown and pressure is rising | keep containing; an unexplained but contained incident is an acceptable state, an explained but ongoing one is not |
| two people are directing | stop, name one commander, say it out loud, continue |
| the rollback does not restore service | escalate scope, consider upstream and infrastructure causes, and communicate the change in expectation |
| a fix is applied directly in production | record it as an emergency change, and open the Stepper step that brings the repository back in line |
| the incident is caused by a third party | contain what is in your control, communicate honestly about what is not, and record the dependency as a risk |
| someone starts assigning blame | redirect to contributing factors; a blaming postmortem buys silence during the next incident |
