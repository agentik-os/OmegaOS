# Write the allocation policy

Produces a versioned allocation policy in which every ceiling carries a number
and a stated consequence for breaching it.

## Trigger

Any of the following: a new pool of capital exists, the first candidate has
arrived and no policy is written, a ceiling has been breached and the allocator
wants to change the rule rather than repeat the breach, or the period review
found a policy line that never survived contact with reality.

## Inputs

- The mandate for the pool: what it is for, over what horizon, and whose money
  it is.
- Funded capital today, and any facility that is drawable and named.
- The personal capital constraint from Wealth {OS}, event
  `wealth.capital_constraint.published`, where the pool is the allocator's own.
- The existing policy and its amendment history, if there is one.
- The realised outcomes of the last period, if there is one.

## Steps

1. State the mandate in one paragraph in the allocator's own words. If the
   allocator cannot say what the pool is for without describing a specific
   candidate, stop and resolve that first: a policy written to fit one deal is
   not a policy.
2. Establish the pool size, split into funded capital and drawable facility.
   Anything that is a pledge, a forecast distribution or an expected exit is
   listed separately and is not part of the pool.
3. Read the Wealth {OS} capital constraint and record it as a hard ceiling on
   the pool. If it is absent or stale, record that fact and proceed against
   explicitly stated funded capital only.
4. Set the cheque band: minimum and maximum for a single initial commitment,
   with the reasoning. A band wide enough to contain every deal you might like
   is not a band.
5. Set the mix: how the pool divides across stage, asset class, sector or
   whatever axis actually drives correlation in this portfolio. Name the axis
   before naming the percentages.
6. Set the concentration ceilings: maximum for a single position including its
   reserve, maximum for a single sector, maximum for a single vintage. For each
   one, write the consequence of a breach.
7. Set the reserve ratio: how much is held back per unit of initial cheque, and
   the rule for when a zero reserve is legitimate.
8. Set the illiquidity ceiling: the maximum fraction of the pool that may be
   locked at once, and the maximum lock period accepted without a separate
   decision.
9. Set the pacing rule: how much of the budget may be deployed in any one
   period, so a strong quarter cannot consume the year.
10. Label every forward-looking assumption in the policy on the E1 to E5 scale.
    Loss rates, hold periods and follow-on rates are assumptions, not facts.
11. **Human approval gate.** Present the complete policy for signature. The
    policy is not in force and no screen may cite it until the allocator has
    signed it. On signature, emit `capital.policy.set`.
12. Where this is an amendment rather than a first policy, route it through
    Review & Governance {OS} first and wait for `change.approved`, then record
    the old line, the new line, the date and the reason in the amendment
    history.

## Completion test

Open the policy and pick any ceiling at random. It has a number, a consequence
for breaching it, and a named axis. Then run `/capital screen` against a
candidate you already declined in the past: the screen returns the same verdict
you reached by judgement, and names the line. If it does not, the policy does
not describe how you actually allocate.

## Failure modes

| Failure mode | What it looks like | Response |
|---|---|---|
| policy written to fit a live deal | the cheque band happens to contain exactly the ask on the desk | flag the coincidence in the amendment history and require the allocator to confirm it in writing |
| ceilings without consequences | "roughly 20 percent per position" | refuse the line, ask for the number and what happens at 21 percent |
| pool includes unfunded capital | a pledge or an expected exit counted as deployable | move it to the separate list, restate the pool, restate the bands |
| every axis at once | mix defined across stage, sector, geography and vintage simultaneously | name the axis that actually drives correlation here, keep the rest as reporting cuts, not ceilings |
| assumptions presented as facts | a loss rate stated flat with no label | apply the E1 to E5 label, usually E3 or E4, and keep it visible in the policy |
| amendment slipped in without governance | a ceiling quietly different from last version | reject the version, route through Review & Governance {OS}, restore the recorded line until `change.approved` arrives |
