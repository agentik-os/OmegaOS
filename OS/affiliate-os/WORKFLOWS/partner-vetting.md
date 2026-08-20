# Partner vetting

Decide whether a product deserves access to an audience the operator spent
years building, before the commission rate is allowed to influence anything.

## Trigger

A partner programme is proposed, an inbound sponsorship arrives, or the
operator considers monetising a tool they already use.

Also triggered by `/affiliate-review` when a live partner's product changes.

## Steps

1. **The operator supplies evidence of use.** What was purchased, on what date,
   what was done with it, what worked and what failed. The OS records it as the
   use evidence. If there is none, the workflow stops here and reports that it
   stopped: the partner's own material is not a substitute.
2. **The OS scores the trust cost** (`/trust-cost`): who in the audience would
   be harmed if this product underdelivers, how visibly, and what recovering
   that trust would take. This is written before any commercial term is read.
3. **The OS pulls the positioning claim** from Positioning {OS} and checks
   coherence. A product that fits the audience but contradicts the claim
   produces a contradiction finding and a decline. The contradiction is handed
   to Positioning {OS}; it is not resolved here.
4. **The OS checks the support boundary.** Who does the customer contact when
   the product fails, and is that answer publishable. If the operator would end
   up absorbing support, the verdict is reject.
5. **The OS now reads the commercial terms** (`/partner-terms`): commission,
   attribution window, payout schedule, claim restrictions, unilateral change
   rights, exit clause. Every term the partner can change alone is marked.
6. **The OS issues a verdict**: accept, reject, or accept with named
   restrictions on what may be claimed.
7. **The operator approves or overrides the verdict.** An override is recorded
   with the operator's stated reason, in the vetting record.

## Completion test

The vetting record exists and contains, each non-empty: use evidence with a
date, a named harm case, a coherence verdict against the current claim, a named
support contact for the customer, a terms record with the unilateral change
rights marked, and a verdict of accept, reject or accept-with-restrictions.

A record whose trust cost was written after the commission rate was read fails
this test, and the ordering is checked from the record's own timestamps.

## Failure and abort

- **No use evidence:** abort. Report that the candidate cannot be vetted and
  what would be needed. Do not produce a provisional verdict.
- **Jurisdictional or contractual ambiguity in the terms:** do not issue an
  accept. Record the ambiguity and escalate to the operator.
- **Coherence contradiction with the claim:** decline, emit the contradiction
  to Positioning {OS}, and do not proceed to terms.
- **Operator overrides a reject:** permitted, but the override and its reason
  are written into the vetting record, and the stop condition in the resulting
  promotion is tightened rather than relaxed.
