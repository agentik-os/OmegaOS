---
name: investment-thesis-os
description: Write the thesis before the cheque, and test it after. Investment Thesis {OS}, unit 57 of the AGENTIK {OS} suite (07 · CAPITAL). Use when the user asks about investment thesis or invokes /investment-thesis-os.
---

# Investment Thesis {OS}

Write the thesis before the cheque, and test it after.

## When to use this

Reach for this OS when:

- an opportunity has been qualified and money is about to move, and the reason
  for the bet exists only in the user's head or in a chat thread.
- the user can describe an opportunity fluently but cannot say what would make
  them walk away from it.
- a position is held, the story around it has changed, and nobody has compared
  the current story with what was originally written.
- a checkpoint date or milestone has arrived and the claims need testing
  against evidence rather than against feelings.
- a bet has closed, and the user wants a verdict that separates a faulty
  argument from an unlucky outcome.
- the user asks how their judgement is actually performing across many bets.

Near neighbours, and the line between them:

- **Due Diligence {OS}.** Diligence verifies what is claimed to be true today
  and returns findings with sources. This OS states what must become true in
  the future and returns claims with kill criteria. If the question can be
  answered by an existing document, it is diligence.
- **Capital {OS}.** Capital decides how much goes where and approves the
  amount. This OS never sizes and never approves; it produces the reference
  that an allocation request has to cite.
- **Deal Flow {OS}.** Deal Flow decides which opportunities are worth looking
  at. This OS only opens once one is worth writing about.
- **Portfolio Management {OS}.** Portfolio Management reports what the position
  is doing now. This OS decides whether what it is doing still matches the
  reason it was taken.

## Capabilities

- Draft a thesis covering what must become true, why now, why us, and what the
  user is being paid for taking which risk.
- Convert loose reasoning into numbered claims, each with an explicit
  disproof condition and a date by which it should be observable.
- Strike unfalsifiable statements and record why they were struck.
- Set kill criteria while exit is still cheap, tied to the real cost of exiting.
- Run a structured pre-mortem and map each ranked loss cause to a claim or to
  an unmonitored gap.
- Maintain a checkpoint calendar and produce a claim by claim verdict against
  dated evidence at each checkpoint.
- Detect thesis drift by comparing the current stated justification against the
  stored original text, quoting both.
- Version a thesis on new evidence with the change and its reason recorded, and
  keep the superseded text readable.
- Retire a thesis as validated, invalidated or superseded, and separate a wrong
  argument from an unlucky outcome.
- Maintain a hit rate across closed theses that excludes retrospective ones.
- Build a pattern library from retirements, written so it changes the next
  draft rather than decorating a report.
- Refuse to size, approve, transmit or recommend anything, and route those
  requests to the human decision that owns them.

## Procedure

1. Establish whether a commitment has already been made. If it has, the thesis
   is labelled `retrospective` from the first line and excluded from the hit
   rate. Say this to the user before writing anything else.
2. Collect the user's own reasoning verbatim, without cleaning it up. The
   unedited version is where the real assumptions are visible.
3. Structure it into the four questions: what must become true, why now, why
   us, what are we paid for taking which risk.
4. Pull verified present-day facts from Due Diligence {OS} by reference, with
   source and date. Anything unverified stays a claim, not a fact.
5. Rewrite each claim as a falsifiable statement: the observation that would
   disprove it, and by when. Strike whatever cannot be disproved and record the
   strike reason.
6. Read the intended commitment size and its exit cost from Capital {OS}, then
   set kill criteria against them while exit is still cheap.
7. Run the pre-mortem: two years on, the money is gone, what most likely
   happened. Rank the causes and map each one to a claim or mark it as an
   unmonitored gap, then decide whether the gap becomes a claim.
8. Store and timestamp the thesis, emit `thesis.drafted` and
   `thesis.kill_criteria.set`, and set the checkpoint calendar.
9. At each checkpoint, gather dated evidence, mark every claim holding,
   weakening, broken or untestable, test the kill criteria, and record the
   result before writing any narrative around it.
10. On contradiction, open a revision: new version, stated change, stated
    reason, superseded text kept, next checkpoint set.
11. On close, record the retirement verdict and the wrong versus unlucky call
    with its basis, then update the hit rate and write the pattern entry.

## Handoffs

- **Due Diligence {OS}** receives the claims that depend on present-day facts,
  and expects each one framed as a question whose answer would change the
  decision, with a stated relevance so it can be scoped or dropped.
- **Capital {OS}** receives the thesis reference and its kill criteria, and
  expects a stored, timestamped identifier it can cite in an allocation
  request. It never receives an amount or a recommendation from this OS.
- **Portfolio Management {OS}** receives the claim register and the checkpoint
  calendar, and expects the claims expressed as things it can observe from the
  reporting it already collects.
- **Deal Structuring {OS}** receives the risks the thesis says the user is
  being paid for, and expects them stated plainly enough to be argued about in
  terms, without any instrument being proposed here.
- **Context & Memory {OS}** receives every thesis version, checkpoint record
  and retirement verdict as canonical state, and expects immutable versions
  rather than edits.
- **Review & Governance {OS}** receives missed checkpoints and unresolved drift
  as items for the user's own operating cadence.
