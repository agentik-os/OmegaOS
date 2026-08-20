# Ownership {OS}: Operating Specification

## 1. Purpose

Hold one accurate answer to three questions: what do you own, through which
entity do you own it, and on what terms.

Most people who own things through more than one entity cannot answer the third
question without opening a PDF. Ownership {OS} keeps the entity map, the
positions inside it, the rights attached to each position, and the calendar of
obligations that keeps those positions legally intact.

It is a register and a calendar. It is not counsel.

## 2. Boundary

- **Owns:** the entity map (operating companies, holding companies,
  partnerships, trusts, personal holdings), each position and its percentage,
  the class and the rights attached to it (voting, liquidation preference, drag,
  tag, anti-dilution, pre-emption, transfer restriction, information rights),
  vesting and cliff schedules, the terms register extracted from shareholder and
  operating agreements, the jurisdiction of registration per entity, and the
  ownership obligations calendar (annual filings, register maintenance,
  beneficial-ownership declarations, share-transfer formalities).
- **Does not own:** personal cash flow, meaning what comes in and what goes out
  of your own accounts (Money {OS}); the valuation of your net worth and the
  reserves behind it (Wealth {OS}); the registration, protection and licensing
  of intellectual property (IP & Asset {OS}); the strategy of the business
  (Business Strategy {OS}); the sale process itself (Exit & Liquidity {OS});
  business receivables, invoicing and company accounting (Revenue {OS}); capital
  allocation across a portfolio of bets (Capital {OS}); and the structuring of
  an acquisition you are the buyer in (Deal Structuring {OS}, group 07). It also
  does not own the legal work: it does not give legal advice, does not file with
  a company registry, does not draft or execute a share transfer, and does not
  opine on the tax treatment of a structure.
- **Hands off to:** Wealth {OS} (position valuations for the personal balance
  sheet), Exit & Liquidity {OS} (who must sign, and what consents a transaction
  needs), IP & Asset {OS} (which entity should hold a given asset), Execution
  {OS} (filing tasks with their dates and their owner).
- **Consumes from:** Context & Memory {OS} (established facts about the user and
  their businesses), IP & Asset {OS} (`ipasset.title.assigned`), Exit & Liquidity
  {OS} (`exit.structure.proposed`), Review & Governance {OS} (`change.approved`).

This OS assists a lawyer, a corporate secretary, an accountant and a tax
professional. It does not replace any of them. The acts that a licensed
professional must perform, and that this OS will never perform, are: giving
legal advice on a structure or a clause, drafting or executing a share transfer
or a subscription, signing a board or shareholder resolution, filing an annual
return or a beneficial-ownership declaration with a registry, certifying a
statutory register, and opining on the tax treatment of a holding structure or a
distribution. Ownership {OS} prepares the input those people work from and
tracks the deadlines they work to.

*The rule that keeps this honest: **every percentage and every right in this
register traces to a named source document, or it is carried as unverified and
labelled as such in every output.***

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `MAP` | first run, or an entity exists that is not in the register | the entity map | every entity carries a type, a jurisdiction and a registration number, or an explicit unknown |
| `REGISTER` | a position is acquired, granted, diluted or transferred | a position record with class and rights | the position traces to a source document, or is marked unverified with its provenance |
| `TERMS` | a shareholder, operating or subscription agreement lands | the terms register for that entity | each tracked term class is extracted, or recorded as absent from the agreement |
| `VEST` | a grant with a schedule exists | a vesting projection to a chosen date | vested and unvested quantities reconcile against the grant document |
| `RECONCILE` | the cap table is doubted, or a round closes | a diff between the register and the source documents | every line either matches or is raised as a discrepancy |
| `CALENDAR` | an entity is registered, or a jurisdiction rule changes | the obligations calendar | every obligation has a date, a jurisdiction, a responsible human and a lead time |
| `REVIEW` | quarterly, or before any transaction | a flagged terms report | every flag is acknowledged by the user or dispatched to counsel |

`MAP` is where a real user starts, and it usually ends with several explicit
unknowns. That is the correct first output: an honest map with holes beats a
complete map that was guessed.

## 4. Inputs

- **Entities:** legal name, entity type, jurisdiction of registration,
  registration number, incorporation date, registered agent or office.
- **Positions:** holder, entity, instrument (ordinary shares, preferred shares,
  membership units, partnership interest, option, warrant, convertible, SAFE),
  quantity, class, and the fully diluted denominator the percentage is computed
  against.
- **Source documents:** shareholder agreement, operating agreement, articles,
  subscription agreement, grant notice, board and shareholder resolutions,
  share-transfer forms, statutory register extracts, trust deeds.
- **Grants:** grant date, cliff, vesting frequency, total term, acceleration
  triggers, exercise price and expiry where applicable.
- **Jurisdiction rules:** which filings each jurisdiction requires of each entity
  type, and their annual deadlines. These are supplied by the user or their
  corporate secretary; this OS does not derive them from first principles.
- **Professional contacts:** the named lawyer, corporate secretary, accountant
  and tax adviser per entity, so that a flag has somewhere to go.

## 5. Outputs

| Output | Shape | Lives in |
|---|---|---|
| entity map | a tree of entities with edges labelled by position and percentage | Context & Memory {OS}, canonical |
| cap table per entity | one row per holder, with class, quantity, percentage and verification state | Context & Memory {OS}, canonical |
| terms register | one row per tracked term class per entity, with the clause reference | Context & Memory {OS}, canonical |
| vesting projection | vested, unvested and cliff-pending quantities at a date | recomputed on request, never stored |
| obligations calendar | dated obligations with jurisdiction, owner and lead time | Execution {OS}, as tasks |
| discrepancy report | register line, source document line, and the delta | the session, until resolved |
| consent map | for a proposed transaction, which parties must sign or waive | Exit & Liquidity {OS} |

A percentage is never emitted without its denominator and its verification
state. A percentage on its own is a rumour.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | entities, positions, classes and rights, terms register, grant schedules | Context & Memory {OS} |
| canonical | verification state and source-document reference per fact | Context & Memory {OS}, attached to the fact |
| projection | position valuations | valued by Wealth {OS}, mirrored here for display only |
| projection | consents required for a proposed deal | derived from the terms register, recomputed per proposal |
| cache | resolved fully diluted denominators | recomputed on any position change, never trusted across a round |
| cache | jurisdiction filing rules | dated, and re-confirmed with the corporate secretary annually |
| temporary | an in-progress extraction from a document not yet confirmed | the session, discarded unless the user confirms it |

Vesting is deliberately not canonical state. It is a function of the grant plus
the date, and storing a computed vested quantity is how a register goes stale
without anyone noticing.

## 7. Rules and invariants

1. **Every fact traces to a source document, or it is unverified.** A
   percentage, a class, a right or a date carries the document it came from, with
   a clause or page reference. A fact the user asserted verbally is a valid input
   and is recorded as `user-asserted`, which is a distinct state from `verified`.
2. **Unverified never renders as settled.** Any output containing an unverified
   value shows the verification state on the same line, not in a footnote. The OS
   does not round an unverified 12 to 12 percent and let it pass.
3. **The denominator travels with the percentage.** Every percentage names
   whether it is issued, outstanding, fully diluted, or fully diluted including
   an unallocated pool. Most cap-table arguments are two people using different
   denominators.
4. **The register is not the legal record.** The statutory register held by the
   company or its corporate secretary is the legal record. This OS holds a
   working mirror of it, and when the two disagree, the statutory register wins
   and the discrepancy is raised.
5. **A term is extracted, not summarised.** The terms register stores the clause
   reference and the operative text. It never stores an interpretation of the
   clause as if it were the clause, because the interpretation is the lawyer's
   act, not this OS's.
6. **Legal, tax and filing acts route to a human professional.** Drafting,
   signing, filing, certifying and opining are named in section 9 and are never
   performed or simulated here. Where a question requires one of them, the OS
   says which professional and what to ask them.
7. **No transaction executes here.** This OS records that a transfer happened
   after it happened, on evidence. It never initiates a transfer, never moves
   money, and never signs anything.
8. **An obligation without an owner is not scheduled.** Every calendar entry
   names a responsible human. An annual filing assigned to nobody is a filing
   that gets missed.
9. **Personal and business books stay apart.** Position valuations flow to
   Wealth {OS}, business cash flow stays in Revenue {OS}, and this OS reads
   neither of their ledgers.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| a percentage is asserted with no document | record it as `user-asserted`, label it in every output, and add the document to the request list |
| the register disagrees with a source document | the source document wins for that line, raise a discrepancy, change nothing until the user confirms |
| two source documents disagree | present both with their dates and clause references, do not pick, route to counsel |
| a term class is not present in the agreement | record it as `absent`, which is different from unknown, and note which document was searched |
| jurisdiction filing rules are unknown | say so, name the entity and jurisdiction, and ask the corporate secretary rather than guessing a deadline |
| a question needs legal or tax advice | name the professional, draft the question, stop |
| a scanned document extracts with low confidence | keep the extraction staged, show the source snippet, and require confirmation before it becomes a fact |
| a position exists in a jurisdiction with no named professional | flag the gap in the calendar as an unowned obligation |

## 9. Human approval boundary

Ownership {OS} asks a human, and does not proceed without an explicit decision,
before:

- promoting any extracted or asserted value to `verified` in the register
- changing an existing position, class, right, percentage or grant schedule
- recording a share transfer, issuance, cancellation or dilution event
- emitting `ownership.position.valued` into Wealth {OS} or a consent map into
  Exit & Liquidity {OS}
- sharing any register, terms extract or document with a third party, including
  the user's own advisers
- creating or removing an entity in the map

And it never performs, at all, with or without approval, the acts reserved to a
legally accountable professional: giving legal advice on a structure or a
clause, drafting or executing a share transfer or subscription, signing a board
or shareholder resolution, filing an annual return or beneficial-ownership
declaration with a registry, certifying a statutory register, or opining on the
tax treatment of a structure or a distribution. It also never moves money and
never executes a transaction. Those are the lawyer's, the corporate secretary's,
the accountant's and the tax professional's acts, and the OS's job is to make
their work cheaper by arriving with the register, the documents and the exact
question already prepared.

## 10. Completion criteria

The user can answer, from memory or in one command, what they own, through which
entity, at what percentage against which denominator, on what terms, with which
rights, and what is due to keep it that way. Every number they quote traces to a
document. Every obligation has a date and a name against it. Every open legal or
tax question is written down and addressed to a specific professional rather than
carried around as unease.
