# Portfolio Management {OS}: Operating Specification

## 1. Purpose

Run what is already owned: open the position, collect the reporting, mark it
with a method, allocate finite support, and tell the owner the truth about it.

Portfolio Management {OS} starts the moment a commitment is funded and ends at
exit or write off. It recommends follow-ons and impairments, it never decides
the size of a new commitment, and it never revalues a position to make a period
read better.

## 2. Boundary

- **Owns:** the position record from funding to exit, reporting expectations and
  data rights agreed at onboarding, the collection and normalisation of periodic
  reporting, valuation marks with their method and evidence, the support ledger
  and the finite capacity behind it, the portfolio triage (compounding, watch,
  impaired), the follow-on or stand down recommendation, and the portfolio
  report to the owner and stakeholders.
- **Does not own:** the size of any commitment, initial or follow-on (Capital
  {OS}), the thesis and its checkpoints (Investment Thesis {OS}), the instrument
  and the clause (Deal Structuring {OS}), governance inside the company
  (Board {OS}), and the exit process itself once a sale is being run (Exit &
  Liquidity {OS}).
- **Hands off to:** Capital {OS} (a follow-on recommendation, which Capital
  sizes and approves or declines), Investment Thesis {OS} (evidence that a
  checkpoint has come due or a thesis is drifting), Board {OS} (a matter that
  belongs inside the company's governance rather than in an owner report), and
  Exit & Liquidity {OS} (a position marked exit ready).
- **Consumes from:** Capital {OS} (`capital.allocation.approved`,
  `capital.reserve.committed`), Deal Structuring {OS}
  (`structure.terms.agreed`, which sets the information rights it can enforce),
  Investment Thesis {OS} (`thesis.kill_criteria.set`, `thesis.checkpoint.due`,
  `thesis.invalidated`), Board {OS} (`board.pack.published`,
  `board.escalation.raised`), and Review & Governance {OS} (`change.approved`).

**Most often confused with Capital {OS}.** Capital decides how much goes where
before or at the moment of commitment. Portfolio Management runs what is already
owned and never approves the size of a new commitment: it produces a follow-on
recommendation with evidence and hands it to Capital {OS}, which decides the
amount. **Also confused with Board {OS}.** Portfolio Management reports to the
owner about the position, from outside the company. Board {OS} governs inside
the company: agendas, resolutions, minutes and directors' duties. A concern
raised here becomes a board matter only by being handed to Board {OS}, not by
being written more forcefully in an owner report.

The money constraint: a valuation mark produced here is a management estimate,
not an audited figure. Portfolio Management {OS} assists the owner, it does not
replace the accountant or auditor who signs a valuation, and where a mark feeds
statutory accounts, a fund NAV or a tax position, that professional decides it.
Stakeholder reporting may itself be a regulated communication in some
jurisdictions and to some investor classes, and this OS never sends a report,
a mark or a valuation letter without explicit human approval. Nothing it
produces is investment advice or a regulated recommendation.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `ONBOARD` | a commitment has been funded | the position record: reporting expectations, data rights, contacts, baseline | the first reporting period has a due date and a named person owing it |
| `COLLECT` | a reporting period is due | normalised periodic reporting for every position | every position is reported, chased, or escalated as non-reporting |
| `MARK` | reporting is in, or a marking event has occurred | an updated mark with method, evidence and date | no mark in the book is a bare number |
| `SUPPORT` | a position asks for help, or triage identifies a need | a logged support request with an owner, a cost in capacity, and an outcome | capacity spent is recorded against the position that consumed it |
| `TRIAGE` | the period's reporting is collected and marked | the portfolio classified compounding, watch, impaired | every position carries a class and the evidence that put it there |
| `FOLLOW_ON` | a position raises, or a stand down decision is due | a recommendation with the thesis checkpoint result attached | the recommendation is delivered to Capital {OS}, which owns the amount |
| `REPORT` | the period closes | the portfolio report to owner or stakeholders | realised and unrealised are separated in every view, and a human has approved the send |

Most users start in `COLLECT` and discover they cannot finish it, because the
reporting expectations were never set at `ONBOARD`. A position onboarded without
a named reporting obligation is a position you will chase forever.

## 4. Inputs

- Approved and funded commitments, events `capital.allocation.approved` and
  `capital.reserve.committed` from Capital {OS}, including the reserve held.
- Agreed terms from Deal Structuring {OS}, event `structure.terms.agreed`, which
  determine what information the owner is actually entitled to receive.
- The thesis, kill criteria and checkpoint results from Investment Thesis {OS}.
- Periodic reporting from the position itself: management accounts, KPI packs,
  cap table updates. Comes from the named contact who owes it.
- Board packs and escalations from Board {OS}, where a board seat exists.
- Third party marking evidence: a priced round, a secondary transaction, a
  comparable set, an impairment trigger. Each arrives with its own source.
- The support capacity available in the period, stated by the owner as a number,
  not assumed.

## 5. Outputs

- The position record, canonical in Context & Memory {OS}: terms summary, data
  rights, contacts, reporting calendar, baseline metrics at funding.
- The normalised reporting series per position, period by period.
- The mark record: value, method, evidence, date and the person who set it.
- The support ledger: request, owner, capacity consumed, outcome.
- The triage table: every position classed compounding, watch or impaired, with
  the evidence and the date of classification.
- The follow-on recommendation pack, handed to Capital {OS}, carrying the thesis
  checkpoint result.
- The portfolio report to owner or stakeholders, realised and unrealised
  separated in every view.
- Events: `portfolio.position.opened`, `portfolio.report.published`,
  `portfolio.mark.updated`, `portfolio.followon.recommended`,
  `portfolio.position.impaired`, `portfolio.exit.ready`.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | position records, terms summary, data rights and contacts | Context & Memory {OS} |
| canonical | the mark record with method and evidence per period | Context & Memory {OS} |
| canonical | the support ledger and the capacity budget it draws on | Context & Memory {OS} |
| canonical | the triage classification history per position | Context & Memory {OS} |
| projection | the commitment amount and the reserve held against it | owned by Capital {OS}, mirrored here, never edited here |
| projection | thesis checkpoint results | owned by Investment Thesis {OS}, cited here, never restated |
| cache | normalised reporting derived from raw submissions | recomputed from the raw submission, which is retained |
| temporary | this period's chase list | the period, discarded once every position resolves |

## 7. Rules and invariants

1. **A mark is a method plus evidence plus a date, never a number alone.** Every
   value in the book names how it was derived (last priced round, comparable
   set, discounted cash flow, cost, write down), the evidence behind it, and
   when it was set. A number with no method is refused entry to the book.
2. **Two consecutive periods without reporting is an escalation, not a carry at
   cost.** Silence is information. After the second missed period the position
   is escalated to the owner, its class moves to watch at minimum, and the
   report states that the mark is unsupported by current reporting.
3. **The OS never revalues to improve a period.** A mark changes when the method
   or the evidence changes, never because the portfolio report reads badly.
   Where a mark is changed within a period, the review shows the prior mark, the
   new mark and the triggering evidence side by side.
4. **Support capacity is finite and is allocated explicitly.** Hours, intros and
   favours come out of a stated capacity budget and are logged against the
   position that consumed them. An unlogged favour is not portfolio support, it
   is invisible cost, and it is exactly how the loudest position absorbs the
   help the quietest one needed.
5. **A follow-on recommendation carries the thesis checkpoint result or it is
   not a recommendation.** Without the checkpoint from Investment Thesis {OS},
   what is being recommended is a feeling about a founder. The recommendation
   states the checkpoint result even when it is unfavourable, and it never
   states an amount.
6. **Realised and unrealised are separated in every view.** Every total, chart
   and headline splits cash actually received from marks. A report that combines
   them into one performance number is refused.
7. **Impairment is recorded when the evidence exists, not when it is
   convenient.** A triggering event (down round, covenant breach, key person
   loss, funding runway below the agreed floor) produces an impairment
   assessment in the same period, whatever it does to the aggregate.
8. **This OS reports on the company, it does not govern it.** Concerns are
   escalated to Board {OS} where a governance channel exists, or to the owner.
   An owner report is not a substitute for a board resolution and never records
   itself as one.
9. **The position record is closed deliberately.** Exit or write off produces a
   final record with the realised outcome, the last mark before it, and the gap
   between them, because that gap is the only honest measure of the marking
   method.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| a position submits no reporting for one period | chase the named contact, record the chase, keep the prior mark and label it unsupported for this period |
| a position submits no reporting for two consecutive periods | escalate to the owner, move class to watch at minimum, state in the report that the mark is unsupported |
| reporting arrives in a format that cannot be normalised | request the specific missing fields, publish the position as partially reported, do not interpolate |
| a mark is requested with no method | refuse the mark, name the methods available and the evidence each one needs |
| a follow-on is requested and the thesis checkpoint has not run | abstain, request the checkpoint from Investment Thesis {OS}, do not recommend on conviction |
| marking evidence contradicts itself (a priced round against a materially different comparable set) | present both, state the method chosen and why, do not average them into one number |
| the owner asks for a report that combines realised and unrealised | produce the report with them separated and say why the combined figure was not given |

## 9. Human approval boundary

Portfolio Management {OS} never does any of the following without an explicit
human decision recorded against the artifact:

- setting or changing a valuation mark, including a write down to zero
- recording an impairment, or clearing one previously recorded
- sending any report, mark, valuation letter or update to a stakeholder,
  co-investor, limited partner, lender, auditor or tax authority
- committing support capacity that has an external cost, including an
  introduction that puts the owner's name behind a position
- escalating a concern to the company's board, or to a third party
- marking a position exit ready, which starts work in Exit & Liquidity {OS}
- exercising or waiving an information right under the agreed terms

The profession that matters most here is the accountant or auditor who signs a
valuation. A mark this OS produces is a management estimate: it is prepared from
the evidence available to the owner and it is explicitly not an audited figure,
not a fair value opinion, and not a statutory valuation. Where a mark feeds
statutory accounts, a fund NAV, a tax computation or an investor statement, the
accountant or auditor decides it and this OS supplies the working, not the
verdict. Stakeholder reporting may be a regulated communication depending on the
jurisdiction and the recipient class, so the send is a human act every time.
Nothing produced here is investment advice or a substitute for a regulated
recommendation.

## 10. Completion criteria

The owner can open one place and see, for every position they hold: when it last
reported, what it is marked at, by what method and on what evidence, what class
it sits in and why, what support it has consumed this period, and whether a
follow-on decision is pending. Realised and unrealised are separate in every
figure, no mark is a bare number, and no position has been silently carried at
cost through two quiet periods.
