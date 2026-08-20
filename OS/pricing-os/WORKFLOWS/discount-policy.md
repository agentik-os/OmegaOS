# Discount policy

Produces the rules a seller may move inside, the ladder of approvals above
that range, the floor beneath it, and the named owner of that floor.

## Trigger

A price book version has gone live and no policy governs movement against it,
or a `/pricing-review` variance report showed sellers routinely operating
outside the policy that exists.

## Steps

1. **Pricing {OS}** reads the live price book version and the unit economics
   from Business Model {OS}. Produces the cost to serve per line, which is the
   arithmetic the floor is derived from.
2. **Revenue {OS}** supplies the realised discount history: what was actually
   granted, on which deals, by whom, and what those deals did afterwards.
   Produces the behavioural baseline.
3. **Pricing {OS}** sets the permitted range a seller may move inside without
   asking anybody. Produces a range per price line, expressed as a percentage
   and as an absolute number, because sellers quote absolutes.
4. **Pricing {OS}** sets the approval ladder above the range: who approves
   what, at which depth of discount. Produces the ladder with named roles, not
   named committees.
5. **Pricing {OS}** sets the floor and names its owner. Produces a floor per
   line and one owner who is structurally not the person negotiating.
6. **Pricing {OS}** states what is recorded on every discount granted: deal,
   amount, approver, reason, and whether the reason was volume, term,
   strategic value or capitulation. Produces the discount record schema.
7. **A human** approves the range, the ladder, the floor and the owner.
   **Pricing {OS}** emits the policy to Sales {OS} and Revenue {OS}.

## Completion test

The policy is complete when:

- every live price book line has a permitted range and a floor
- each floor is at or above cost to serve, or is explicitly a named subsidy
- the floor owner is named and is not any person who carries a sales quota on
  the deals the floor governs
- the approval ladder resolves any request to exactly one approver, with no
  request falling into a gap between two rungs
- the discount record schema names the reason field, and the reason field has
  a closed set of values including one that means capitulation
- Sales {OS} can answer, for a specific requested number, granted, escalated
  or refused, without contacting Pricing {OS} for anything inside the range
- a human approval of the exact policy text is recorded

## Failure and abort

- **No unit economics.** Abort at step 1. A floor without cost to serve is a
  number chosen for how it sounds, and it will be argued away in the first
  hard negotiation.
- **The floor owner also carries the quota.** Refuse to publish. Escalate for
  a different owner. A floor that its own beneficiary can move is not a floor,
  it is a suggestion, and the policy would be decorative from day one.
- **Discount history contradicts the proposed policy.** Publish the policy and
  the variance side by side. Do not loosen the policy to match observed
  behaviour: that is not calibration, it is ratification.
- **A requested price below the floor arrives while the policy is being
  written.** Refuse it at the Sales boundary and escalate to the interim
  owner. Record it whether or not it is granted.
- **Human approval withheld.** No policy is emitted. Until one exists, every
  request outside the price book escalates, with no exception granted for
  urgency, because urgency is the condition under which floors are lost.
