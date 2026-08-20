# Workflow: Structure preference and tax gate

Write down what the operator will and will not accept, before an offer exists,
and route the tax question to a tax professional before any structure is agreed.

**Mode:** `STRUCTURE`
**Produces:** the written structure preference, the walk-away, the tax question pack, and `exit.structure.proposed`
**Typical duration:** one session to draft, then the adviser's turnaround

## Trigger

Any of:

- a real conversation with a counterparty is plausible within the timeline
- a counterparty has signalled interest, with or without a number
- the readiness assessment is scored and the target window is inside twelve months
- an existing preference is older than the last material change to the entity
  map, the cap table or the tax position

Run it while no offer is live. A walk-away written during a live process is
written against the momentum of that process rather than against the operator's
own criteria.

## Steps

1. **State the acceptable consideration mix.** Cash at close versus deferred,
   and the minimum cash at close below which the operator declines. Deferred
   consideration is a bet on the buyer's future conduct, and the operator is
   usually no longer in control of that conduct.

2. **Define the earn-out position, if any.** What is measured, who measures it,
   over what period, and under whose operating control. An earn-out measured on
   a metric the buyer controls is not consideration, it is hope. Record the
   measurement disputes the operator can foresee.

3. **State the escrow position.** Percentage, period, and what may be claimed
   against it. Record what the operator considers an unacceptable holdback.

4. **State the working capital treatment.** The target, how it is calculated,
   and who prepares the closing statement. Working capital adjustments routinely
   move real money after the headline price is agreed.

5. **Record the non-negotiables.** Restrictive covenants and their scope and
   duration, transition service commitments and the hours they imply, any
   personal guarantee or indemnity cap, and any employment or earn-out lock-in
   period. Each one is a constraint on the operator's life after the close.

6. **Write the walk-away.** A price and a set of terms below which the operator
   declines, dated, and recorded now. It is the single artifact of this workflow
   that only has value if it exists before the first offer.

7. **Assemble the tax question pack and send it to the tax professional.** At
   minimum: asset sale versus share sale and their respective treatments, the
   treatment of deferred consideration and of an earn-out, the timing questions
   around the close date, any residency or entity-location question the entity
   map raises, and what the operator would owe under each shape of the
   preference. **Tax treatment routinely moves the net proceeds more than the
   headline price does, and most of it is irreversible once the agreement is
   signed.**

8. **Hold the preference at provisional until the tax professional responds.**
   This OS refuses to record a structure preference as settled before that
   review. `STRUCTURE` does not complete on an unanswered pack.

9. **Assemble the counsel question pack in parallel.** A letter of intent, an
   exclusivity clause and a non-disclosure agreement are binding legal
   instruments, not preliminaries, and exclusivity removes the operator's
   leverage for the period it runs. The pack lists what each term does and what
   to ask; it does not draft, redline or recommend. Counsel decides.

10. **On the tax professional's response, settle the preference**, record what
    changed and why, and with human approval emit `exit.structure.proposed` to
    Ownership {OS}. It is a proposal for a human and counsel to accept or
    reject, never an instruction to restructure.

11. **Hand the resulting proceeds view to Wealth {OS}** via `/exit-proceeds`, so
    reserves and long-horizon goals are planned against a range that reflects
    the structure rather than the headline.

## Completion test

The preference is settled when all of the following hold:

- consideration mix, earn-out, escrow, working capital treatment and
  non-negotiables are each written down with a stated position
- the walk-away exists, is dated, and predates any live offer
- the tax question pack has been sent to a named tax professional and answered
- the preference records what the tax response changed
- the counsel question pack exists for the letter of intent, exclusivity and
  non-disclosure terms, and no term has been decided here
- `exit.structure.proposed` has been emitted, with human approval, as a proposal

It is not done while the tax pack is unanswered, while the walk-away is unwritten,
or if this OS has stated a preferred structure as settled on its own judgement.
This OS assists a lawyer, an accountant and a tax professional; it does not
replace any of them, and it never signs, transfers, receives or moves money.
