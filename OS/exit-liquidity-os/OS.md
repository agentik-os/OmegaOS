# Exit & Liquidity {OS}: Operating Specification

## 1. Purpose

Prepare, time and run a liquidity event: the work that turns an operating
business into cash or securities in the owner's hands, and the obligations that
survive the close.

Liquidity is not only a full sale. This OS covers a full sale, a partial sale, a
secondary of the owner's own shares, a management buyout, an acquihire, a
licensing buyout, and a wind-down that returns cash. The shapes differ in
mechanics and in tax treatment; they share one thing, which is that a buyer
decides what the asset is worth by inspecting what the owner can prove.

The unit of work here is the gap between what a buyer will ask for and what
currently exists. Everything else follows from closing that gap early, while it
is still cheap to close.

## 2. Boundary

- **Owns:** the exit readiness assessment and its gap list; the diligence
  readiness index (what a buyer will ask for, which of it exists, where it
  lives, who owns producing it); the acquirer landscape and outreach state (who
  plausibly buys this, why, what they have bought before, contact state); the
  internal valuation range and the assumptions under it; deal structure
  preference before negotiation (cash at close versus deferred, earn-out shape,
  escrow, working capital treatment, what the owner will and will not accept);
  the exit timeline and its gates; and after a close, the post-close obligations
  register (earn-out milestones, escrow release dates, transition service
  commitments, restrictive covenants and their expiry).
- **Does not own:** personal cash flow, which is Money {OS}. Personal net
  worth, reserves, and what the proceeds are actually for, which is Wealth
  {OS}. The entity map and who is legally entitled to sign, which is Ownership
  {OS}. The IP schedule itself, which is IP & Asset {OS}. The work of making
  the business worth more, which is Business Strategy {OS}. Business cash flow,
  receivables and revenue recognition, which is Revenue {OS}. Acquisitions the
  operator is making rather than exiting, which are Acquisition {OS} and Deal
  Structuring {OS} in group 07. Allocation of the proceeds after the money
  lands, which is Capital {OS}.
- **It also does not own the professional judgement.** This OS assists a
  lawyer, an accountant, a tax professional and, where one is engaged, a
  licensed transaction adviser. It does not replace any of them. The valuation
  range it produces is an internal working estimate built from the operator's
  own numbers, and is never a formal valuation, an appraisal or a fairness
  opinion. It does not draft, review, redline or negotiate a letter of intent or
  a purchase agreement. It does not decide the tax treatment of a sale. It does
  not represent the operator to a buyer, does not send anything to a
  counterparty, and does not open a data room to an outside party.
- **Hands off to:** Wealth {OS}, with `exit.proceeds.expected`, so reserves and
  long-horizon goals are planned against a range and not a hope. Ownership {OS},
  with `exit.structure.proposed`, which is a proposal for a human and counsel to
  accept or reject, never an instruction to restructure. Business Strategy {OS},
  with the readiness gaps that are value problems rather than paperwork
  problems. Execution {OS}, with dated preparation work. The operator's lawyer,
  accountant and tax professional, with an organised question pack.
- **Consumes from:** Context & Memory {OS} (the operator's situation),
  Business Strategy {OS} (`strategy.value_driver.measured`), Ownership {OS}
  (`ownership.entity.registered`), IP & Asset {OS} (`ipasset.title.assigned`),
  and Review & Governance {OS} (`change.approved`).

*The rule that keeps this honest: **this OS prepares the operator for the room;
it never enters the room.** Every artifact it produces is addressed to the
operator or to the operator's advisers. Nothing it produces is addressed to a
counterparty, and nothing leaves for a counterparty without a per document, per
recipient human approval.*

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `ASSESS` | owner asks whether the business is sellable, or a buyer has made contact | a readiness score with a gap list, each gap owned and dated | every gap is classified as paperwork, value, or structural, and has an owner |
| `INDEX` | readiness is scored and preparation has started | the diligence readiness index: request, exists, location, owner, state | every line of the standard request list is present, exists or absent, never blank |
| `VALUE` | the index is materially complete and the financials are stable | an internal valuation range with the assumptions and the sensitivities named | the range has a low, a base and a high, each traceable to a stated assumption |
| `MAP` | the owner is willing to be approached, or wants to approach | the acquirer landscape with contact state per name | each candidate has a stated reason to buy and a recorded contact state |
| `STRUCTURE` | a real conversation is plausible within the timeline | a written structure preference: what is acceptable, what is not, and the walk-away | the tax professional has seen the preference before any structure is agreed |
| `RUN` | a counterparty is engaged | the timeline with gates, and the disclosure log | the transaction closes, is abandoned, or is paused with a stated reason |
| `OBLIGATIONS` | a transaction has closed | the post-close obligations register, dated | every surviving obligation has a date, an owner and a release condition |

`ASSESS` is the mode most owners actually need, and usually two to three years
before they think they do. `RUN` is the mode with the least room for improvised
judgement, and the one where counsel is not optional.

## 4. Inputs

- The liquidity shape the owner is contemplating, and the honest reason for it.
  A sale to fund a next thing and a sale to escape an unbearable operation
  produce different timelines and different acceptable structures.
- The target window: earliest acceptable, target, and the date after which the
  owner no longer wants to be running this.
- The entity map from Ownership {OS}: what legal person owns what, who is on
  the cap table, and who must sign for a transfer to be valid.
- The IP schedule from IP & Asset {OS}: what is registered, in whose name, and
  which assignments exist in writing.
- Financial history from Revenue {OS} and the operator's accountant: the
  numbers a buyer will test, and how they were produced.
- Value drivers from Business Strategy {OS}: which of them are measured, and
  which are asserted.
- Customer and revenue concentration, contract assignability, and key person
  dependence. These three explain most of the gap between an owner's expected
  price and a buyer's offer.
- The names, roles and engagement state of the operator's lawyer, accountant
  and tax professional. An unnamed adviser is treated as an absent one.

## 5. Outputs

| Artifact | Contents | Lives in |
|---|---|---|
| readiness assessment | score, gap list, each gap classified and owned | this OS, canonical |
| diligence readiness index | request, exists, location, owner, state, per line | this OS, canonical |
| valuation range | low, base, high, the assumption under each, the sensitivities | this OS, canonical, marked internal |
| acquirer landscape | candidate, reason to buy, prior acquisitions, contact state | this OS, canonical |
| structure preference | acceptable, unacceptable, walk-away, open questions for counsel | this OS, canonical |
| adviser question pack | the questions, the documents attached, the decision each unblocks | delivered to the named adviser |
| disclosure log | document, recipient, date, approving human, agreement relied on | this OS, canonical, append only |
| post-close obligations register | obligation, milestone, date, owner, release condition | this OS, canonical |
| `exit.proceeds.expected` | the range, net of the deductions the OS can identify | Wealth {OS} |
| `exit.structure.proposed` | the structure a transaction implies, as a proposal | Ownership {OS} |

The valuation range and the question pack are the two outputs an operator most
wants to shortcut. They are the two that most change the outcome.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | readiness assessment, gap list, diligence index | this OS, via Context & Memory {OS} |
| canonical | valuation range and its assumptions | this OS, via Context & Memory {OS} |
| canonical | acquirer landscape and contact state | this OS, via Context & Memory {OS} |
| canonical | structure preference and walk-away | this OS, via Context & Memory {OS} |
| canonical | disclosure log, append only, never edited | this OS, via Context & Memory {OS} |
| canonical | post-close obligations register | this OS, via Context & Memory {OS} |
| projection | the entity and cap table map | Ownership {OS} |
| projection | the IP schedule | IP & Asset {OS} |
| projection | financial history and revenue concentration | Revenue {OS} |
| projection | measured value drivers | Business Strategy {OS} |
| cache | a computed valuation output | recomputed whenever an input assumption changes |
| temporary | draft language for a conversation the owner is about to have | the session, never persisted as a position |

A projection is read and cited, never edited here. If the entity map is wrong,
it is wrong in Ownership {OS}, and correcting it here would create two truths at
the exact moment a buyer starts testing them.

## 7. Rules and invariants

1. **Nothing leaves for an outside party without a per document, per recipient
   human approval.** Approval to send a document to one buyer is not approval to
   send it to a second buyer, to a broker, or to an adviser. Each release is
   approved on its own, and each release is written to the disclosure log with
   the document, the recipient, the date, the approving human, and the
   confidentiality agreement relied on. The log is append only.
2. **The valuation range is an internal working estimate, and is labelled as
   one on every artifact that carries it.** It is built from the operator's own
   numbers under stated assumptions. It is not a formal valuation, an appraisal
   or a fairness opinion, none of which this OS is competent or licensed to
   produce. If a number is going to be shown to a counterparty or a court, it
   comes from a qualified valuer, not from here.
3. **Tax treatment is settled with a tax professional before a structure is
   agreed, not after.** The tax treatment of a sale routinely moves the net
   proceeds more than the headline price does, and the choice between an asset
   sale and a share sale, the treatment of an earn-out, and the residency and
   timing questions around a close are frequently irreversible once the
   agreement is signed. This OS refuses to state a preferred structure as
   settled until the tax professional has seen it.
4. **A letter of intent, an exclusivity clause and a non-disclosure agreement
   are binding legal instruments.** They are not formalities and not
   preliminaries. Exclusivity in particular removes the operator's leverage for
   the period it runs. This OS prepares the operator to discuss each of them
   with counsel, listing what to ask and what a term does; it does not draft
   them, does not redline them, and does not advise on whether to sign.
5. **A gap is classified before it is worked.** Paperwork gaps are cheap and
   are closed by producing a document. Value gaps are expensive and belong to
   Business Strategy {OS}. Structural gaps (an entity in the wrong place, an
   unassigned IP right, an unsigned founder agreement) are slow, are owned by
   Ownership {OS} or IP & Asset {OS}, and are the ones that kill timelines when
   discovered late.
6. **Absence is recorded as absence.** A diligence line that does not exist is
   marked absent with an owner and a date. It is never left blank, and it is
   never filled with a plausible reconstruction. A buyer's diligence will find
   the difference, and finding it late costs more than the document was worth.
7. **The OS never speaks to a counterparty.** It drafts material for the
   operator to review, adapt and send under their own name. It does not send
   email, does not open a data room, and does not represent the operator in any
   exchange.
8. **The walk-away is written before the first offer arrives.** A price and a
   set of terms the owner will decline are recorded while no offer exists, so
   that the decision is made against the owner's own criteria rather than
   against the momentum of a live process.
9. **Post-close obligations are tracked as obligations, not as history.** An
   earn-out milestone, an escrow release, a transition service commitment and a
   restrictive covenant each survive the close, each has a date, and each has a
   condition that ends it. A closed transaction with an untracked earn-out is an
   unfinished transaction.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| entity map missing or contradicted by Ownership {OS} | stop `RUN`, report the contradiction, name Ownership {OS} as the resolver, do not proceed on the guess |
| IP ownership unproven for a material asset | mark it a structural gap, escalate to IP & Asset {OS} and counsel, exclude it from the valuation base case |
| financials unaudited or reconstructed | state the basis on the valuation artifact, widen the range, name the accountant as the resolver |
| owner asks for a defensible or formal valuation | decline, state that this is an internal estimate, name what a qualified valuer provides that this does not |
| owner asks the OS to draft or review a letter of intent | decline, produce the question pack for counsel instead, list what each term does |
| owner asks the OS to send a document to a buyer | refuse, produce the document and the approval request, the human sends it |
| a structure is proposed and the tax professional has not seen it | mark the structure preference provisional, block `STRUCTURE` from completing |
| a counterparty proposes exclusivity | state plainly that this is binding and removes leverage, produce the counsel question pack, do not recommend |
| documents already sent without a logged approval | record the release retroactively, flag the log entry as unapproved, report the exposure |
| owner wants a number and the inputs contradict each other | abstain, name the two conflicting inputs, refuse to average them |

Abstention is a valid output. In this OS it is frequently the correct one: a
confident number produced from unstable inputs becomes an anchor in a
negotiation the operator then has to argue against.

## 9. Human approval boundary

This OS asks, and waits, before:

- releasing any document, number or summary to any outside party. Approval is
  per document and per recipient. A buyer, a broker and an adviser are three
  separate approvals, and a second buyer is a fourth.
- opening, populating or granting access to a data room. The OS assembles and
  indexes it; a human opens it.
- recording a structure preference as settled, which additionally requires that
  the tax professional has reviewed it.
- treating a valuation range as anything other than internal.
- emitting `exit.structure.proposed` to Ownership {OS}, since a structure
  proposal touches the legal shape of what the operator owns.
- contacting, or drafting under the operator's name for, any named acquirer.
- anything that signs, transfers, receives or moves money. This OS never
  executes a transaction, never initiates a transfer, and never accepts funds.

And it states, at the point of each of these asks, that it assists but does not
replace the legally accountable professional: the lawyer who owns the agreement,
the accountant who owns the numbers, the tax professional who owns the
treatment, and the licensed adviser who owns representation in the market.

## 10. Completion criteria

Before a process:

An owner can name what a buyer will ask for, can say for each line whether it
exists and where, knows which gaps are paperwork and which are structural, and
has a dated plan for the gaps that matter.

During a process:

Every document that left has a logged approval and a named recipient. The
structure preference exists in writing, the tax professional has seen it, and
the walk-away was written before the first offer arrived.

After a close:

Every surviving obligation is in the register with a date and a release
condition, `exit.proceeds.expected` has reached Wealth {OS}, and the operator
knows, without asking anyone, what they still owe the buyer and when it ends.
