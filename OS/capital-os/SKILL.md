---
name: capital-os
description: Allocate capital deliberately across a portfolio of bets. Capital {OS}, unit 56 of the AGENTIK {OS} suite (07 · CAPITAL). Use when the user asks about capital or invokes /capital-os.
---

# Capital {OS}

Decide how much capital goes where, before or at the moment of commitment.

## When to use this

Reach for Capital {OS} when the question is an amount:

- You have a pool of capital and no written rules for spending it.
- A candidate has arrived and you want to know whether it is even in band
  before you spend two weeks on diligence.
- You know you want to do a deal and you do not know how big the cheque should
  be, or what to hold back for the next round.
- You are about to make a commitment that would make one position larger than
  everything else you own.
- The period is closing and you want to know whether you deployed at the pace
  you planned, or whether you front-loaded and are now out of ammunition.
- Portfolio Management {OS} has recommended a follow-on and somebody has to
  decide the number.

Near neighbours, and the one line that separates them:

- **Portfolio Management {OS}:** it runs what you already own and recommends;
  Capital decides the amount. Before or at commitment is Capital, after
  commitment is Portfolio Management.
- **Investment Thesis {OS}:** it says what must become true for the bet to
  work; Capital says how much you are willing to lose finding out.
- **Due Diligence {OS}:** it verifies what is claimed to be true today; Capital
  sizes the position given what diligence found.
- **Deal Structuring {OS}:** it chooses the instrument and writes the clause;
  Capital sets the amount that instrument carries.
- **Wealth {OS}:** it owns your household cash flow, monthly close, emergency
  fund, personal debt and personal net worth; Capital consumes one published
  constraint from it and never sees a personal transaction.

## Capabilities

- Draft or revise an allocation policy: cheque bands, stage or asset mix,
  concentration ceilings, reserve ratio and pacing.
- Convert funded capital and a period into a deployable budget with a pacing
  plan that reconciles to cash that exists.
- Screen a candidate against policy in minutes and return in band, out of band,
  or ceiling breach, naming the specific line.
- Size a commitment against the thesis, the diligence outcome and the agreed
  terms, and produce an approval or a decline.
- Compute the follow-on reserve at the same moment as the initial cheque and
  write it into the same decision record.
- Track concentration by position, sector, stage and vintage against every
  ceiling, and report drift with the number, not an impression.
- Commit and release reserves against named positions, keeping the reserve
  ledger reconciled.
- Run the written amendment path when the allocator wants to breach a ceiling,
  including the Review & Governance {OS} handshake.
- Report pacing against plan at period close, with realised outcomes measured
  against the assumptions the policy was built on.
- Label every forward-looking number E1 to E5 and refuse single-point return
  promises.
- Produce a signed decision record that a human executes, and never touch the
  wire.

## Procedure

1. Establish the pool: mandate, horizon, and what capital is genuinely funded.
   Read the Wealth {OS} capital constraint if one has been published.
2. Write or load the allocation policy. If none exists and a live candidate is
   waiting, say so plainly and offer to write the policy first.
3. Set the deployable budget for the period and the pacing that spends it.
   Reconcile it against cash that exists or a named drawable facility.
4. For each candidate, run `SCREEN` before any deep work: test it against the
   cheque band, the mix and every ceiling, and return a verdict that names the
   line it passed or failed.
5. For a candidate that survives, gather the thesis and its kill criteria from
   Investment Thesis {OS} and the outcome from Due Diligence {OS}. Abstain if
   either is missing.
6. Size the commitment. State the initial amount, the follow-on reserve, the
   expected lock period and the resulting illiquid fraction of the pool.
7. Test the sized commitment against the ceilings again, with the reserve
   included. A breach goes to decline or to a written amendment, never to an
   undocumented exception.
8. Produce the decision record: amount, reserve, policy lines tested, epistemic
   labels on every forward-looking claim, and the human signature block.
9. Obtain the human signature. Emit `capital.allocation.approved` or
   `capital.allocation.declined`, and `capital.reserve.committed` where a
   reserve was set.
10. Hand the approved amount onward and stop. Execution of the payment is a
    human act outside this OS.
11. At period close, run `REVIEW`: pacing against plan, concentration against
    ceilings, realised outcomes against assumptions. Mark every policy line
    held, breached or amended, and emit `capital.pacing.reported`.

## Handoffs

- **Deal Structuring {OS}** receives an approved amount and expects the size,
  the reserve and any conditions the allocation was made subject to, so the
  instrument can carry them.
- **Portfolio Management {OS}** receives an approved commitment and expects the
  amount, the reserve held against it, the lock expectation and the thesis
  reference, so it can open the position with a baseline.
- **Review & Governance {OS}** receives a policy change or a ceiling breach and
  expects the current policy line, the proposed line, the reason, and what is
  lost if the breach is allowed. It returns `change.approved`.
- **Exit & Liquidity {OS}** receives realised proceeds expectations from the
  pacing plan and expects the reserve position, so a release can be planned.
- **The human allocator** receives the signed decision record and performs the
  payment. Nothing downstream of the signature belongs to this OS.
