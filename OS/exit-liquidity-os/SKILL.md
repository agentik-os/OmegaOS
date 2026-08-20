---
name: exit-liquidity-os
description: Prepare, time and run a liquidity event. Exit & Liquidity {OS}, unit 55 of the AGENTIK {OS} suite (06 · OWN). Use when the user asks about exit & liquidity or invokes /exit-liquidity-os.
---

# Exit & Liquidity {OS}

Prepare, time and run a liquidity event.

## When to use this

Use it when the operator is contemplating, preparing for, or living through any
event that converts an operating business into cash or securities: a full sale,
a partial sale, a secondary of their own shares, a management buyout, an
acquihire, a licensing buyout, or a wind-down that returns cash.

The most valuable moment to open it is the least urgent one. An unsolicited
approach from a buyer is a common trigger and a poor starting point, because by
then the structural gaps are expensive to close and the operator is negotiating
on someone else's clock.

Also open it after a close, and keep it open. An earn-out milestone, an escrow
release, a transition service commitment and a restrictive covenant all survive
the transaction, and each has a date the operator will otherwise miss.

Near neighbours it is confused with:

| If the question is | The OS is |
|---|---|
| what will I live on, month to month | Money {OS} |
| what is my net worth, and what are the proceeds for | Wealth {OS} |
| which entity holds this, and who must sign | Ownership {OS} |
| is this IP registered, and in whose name | IP & Asset {OS} |
| how do I make the business worth more | Business Strategy {OS} |
| what is the business collecting, and from whom | Revenue {OS} |
| I am buying a company, not selling one | Acquisition {OS}, Deal Structuring {OS} |
| the money has landed, where does it go | Capital {OS} |

**What this OS is not.** It assists a lawyer, an accountant, a tax professional
and a licensed transaction adviser. It replaces none of them. Specifically:

- The valuation range it produces is an internal working estimate built from
  the operator's own numbers under stated assumptions. It is never a formal
  valuation, an appraisal or a fairness opinion.
- It does not draft, review, redline or negotiate a letter of intent or a
  purchase agreement. A letter of intent, an exclusivity clause and a
  non-disclosure agreement are binding legal instruments, not formalities. The
  OS prepares the operator to discuss them with counsel; counsel decides them.
- It does not decide the tax treatment of a sale. Treatment routinely moves the
  net proceeds more than the headline price does, and it must be settled with a
  tax professional before a structure is agreed, not after it is signed.
- It does not represent the operator to a buyer, does not send anything to a
  counterparty, and does not open a data room to an outside party.
- It never signs, transfers, receives or moves money. Every one of those needs
  an explicit human decision, and the human is the one who acts.

## Capabilities

- Score exit readiness against what a buyer will actually test, and return a
  gap list where each gap is classified paperwork, value or structural, and has
  an owner and a date.
- Build the diligence readiness index: the standard request list, and for each
  line whether it exists, where it lives, who produces it, and its state.
  Absence is recorded as absence, never left blank.
- Produce an internal valuation range with a low, a base and a high, each
  traceable to a named assumption, plus the sensitivities that move it most.
- Map the plausible acquirer landscape: who buys this shape of asset, why, what
  they have bought before, and the current contact state per name.
- Write the structure preference before negotiation: cash at close versus
  deferred, earn-out shape, escrow, working capital treatment, and the explicit
  walk-away.
- Assemble and index a data room, and maintain the append only disclosure log
  of what was released, to whom, on what date, approved by which human, under
  which confidentiality agreement.
- Assemble question packs for the lawyer, the accountant and the tax
  professional, each stating the decision it unblocks and carrying the documents
  the adviser needs.
- Track post-close obligations: earn-out milestones, escrow release dates,
  transition service commitments, restrictive covenants and their expiry.
- Maintain the exit timeline and its gates, and report which gate is blocking.

## Procedure

1. **Establish the shape and the window.** What kind of liquidity, why, and
   between which dates. A sale to fund a next venture and a sale to escape an
   unbearable operation accept different structures.
2. **Pull the projections.** Entity map from Ownership {OS}, IP schedule from
   IP & Asset {OS}, financial history from Revenue {OS} and the accountant,
   measured value drivers from Business Strategy {OS}. Read them, cite them,
   never edit them here.
3. **Score readiness and classify every gap.** Paperwork gaps stay here. Value
   gaps go to Business Strategy {OS}. Structural gaps go to Ownership {OS} or
   IP & Asset {OS} and to counsel, and go first, because they are slow.
4. **Build the diligence index** line by line against the standard request
   list, marking each present with a location and an owner, or absent with an
   owner and a date. Emit `exit.dataroom.indexed`.
5. **Produce the internal valuation range**, labelled internal, with the
   assumptions and sensitivities exposed. Where an input is unstable, widen the
   range rather than picking a point. Emit `exit.readiness.scored`.
6. **Map acquirers** and record contact state per candidate. No candidate is
   contacted by this OS.
7. **Write the structure preference and the walk-away**, before any offer
   exists. Assemble the tax question pack and route it to the tax professional.
   The preference stays provisional until that professional has reviewed it.
8. **Hand the range to Wealth {OS}** as `exit.proceeds.expected`, so reserves
   and long-horizon goals are planned against a range rather than a hope, and
   hand the implied structure to Ownership {OS} as `exit.structure.proposed`,
   as a proposal for a human and counsel to accept or reject.
9. **Run the process against the timeline.** Every release to an outside party
   is approved per document and per recipient, and written to the disclosure
   log. Counsel owns the letter of intent, the exclusivity and the agreement.
10. **On close, open the obligations register.** Every surviving obligation
    gets a date, an owner and a release condition. Emit
    `exit.obligation.tracked`, and keep tracking until each one expires.

## Handoffs

| Receives | What it gets | What it does with it |
|---|---|---|
| Wealth {OS} | `exit.proceeds.expected`, a range net of identifiable deductions | plans reserves and long-horizon goals against a range, not a single number |
| Ownership {OS} | `exit.structure.proposed`, a proposal only | evaluates the entity and cap table consequences with counsel |
| Business Strategy {OS} | the readiness gaps classified as value problems | works them as value drivers, and measures them |
| Execution {OS} | the dated preparation work | schedules and tracks it as ordinary work |
| Review & Governance {OS} | consequential change requests | returns `change.approved`, which this OS consumes |
| the lawyer, accountant, tax professional | an organised question pack per adviser | answers within their own professional accountability |

What this OS expects back: `strategy.value_driver.measured` from Business
Strategy {OS}, `ownership.entity.registered` from Ownership {OS},
`ipasset.title.assigned` from IP & Asset {OS}, and `change.approved` from
Review & Governance {OS}. Each of those closes a gap that would otherwise be
found by a buyer, at a worse moment and at a worse price.
