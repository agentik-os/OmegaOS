# Story object

Take raw lived material and produce a durable story object in the bank: mined,
truth classed, deepened, shaped, consent cleared and versioned.

## Trigger

The operator has raw material (a moment, a transcript, a business event, a note,
an audio transcription) and no story object for it, or an existing object needs
to be rebuilt after new evidence arrived.

## Steps

1. **Storyteller {OS}** runs `/story` and produces the routing decision: the
   intended outcome, the agency contract, the story class and the truth class.
   The contract is stated in one line, and COACH holds unless the operator
   explicitly asked for a draft.
2. **Storyteller {OS}** runs INTENT and produces the audience, the job, the
   channel, the privacy level and the stakes. One precise question is asked
   only where the missing answer changes the work materially.
3. **Storyteller {OS}** runs CAPTURE with `/moment` or `/interview` and produces
   the raw record in the operator's own words. Questions are open and
   observable, hypotheses are offered as hypotheses, and exact words are marked
   as quotes only when supplied or confirmed.
4. **Storyteller {OS}** runs MINE and produces the candidate change under
   pressure: what changed, what it cost, what was chosen, what is contradicted.
5. **Storyteller {OS}** runs VERIFY with `/truthcheck` and produces the truth
   class of every element: documented, corroborated, remembered, interpreted,
   composite, hypothetical or fictional, with the load bearing elements flagged.
6. **Network {OS}** supplies the consent status of every named third party.
   Storyteller {OS} produces the consent gap list: who is named and not
   cleared.
7. **Operator** resolves each gap by consent, abstraction, anonymisation,
   omission or delay. The lifecycle does not pass VERIFY until every load
   bearing fact and every consent gap is resolved.
8. **Storyteller {OS}** runs DEEPEN and produces desire, obstacle, stakes,
   belief, choice, consequence and the unresolved edge, named rather than
   smoothed.
9. **Storyteller {OS}** runs SHAPE and produces the architecture, the scene
   order, the hook and the ending, selected after the material is known.
10. **Storyteller {OS}** runs VOICE against the operator's own samples and the
    Brand {OS} voice rules, and produces the drift report. A conflict with a
    Brand rule is reported, never silently resolved inside the story.
11. **Storyteller {OS}** persists with `omega-story capture`, then `add-claim`
    per claim and `add-consent` per named person, and produces the bank record
    id. Nothing is reported saved unless the CLI wrote it.
12. **Storyteller {OS}** runs `omega-story validate` and `/score`, and produces
    the structural completeness report, explicitly labelled as not a truth
    signal.

## Completion test

`omega-story show <id>` returns a record whose every load bearing element
carries a truth class, whose every named third party carries a consent record
with a surface scope and an expiry, whose unresolved edge is a populated field,
and whose structure was recorded after the material rather than before.
`omega-story validate <id>` passes. Any element with an empty truth class, any
named person with no consent record, or a shape recorded before MINE completed
means the workflow did not complete.

## Failure and abort

- A load bearing fact cannot be verified at step 5: hold at VERIFY, keep the
  object in the bank as unreleased, and return NEEDS TRUTH CHECK naming the
  exact fact. Do not soften it into an impression to get past the gate.
- Consent is refused or unreachable at step 7: offer abstraction,
  anonymisation, omission, delay or private only storage, and never pressure
  disclosure. If none is acceptable, the object stays private and DO NOT
  PUBLISH is the verdict.
- Two accounts of the same event contradict at step 4 or 5: record both with
  their truth classes side by side, mark the contradiction on the object, and
  let the operator decide which is told. Do not merge them into a smoother
  account.
- The operator asks for a draft while the contract is COACH: state the
  contract, ask for explicit authorisation, and produce no prose in the
  meantime.
- The CLI write fails at step 11: report the failure, keep the material in the
  session, and never report the object as banked. A claimed save that did not
  happen is the one failure this bank exists to prevent.
