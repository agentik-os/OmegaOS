# IP & Asset {OS}: Operating Specification

## 1. Purpose

Keep a defensible record of every intellectual property right and durable asset
the user has created or acquired: what it is, whether it is protected, who can
prove they own it, who is allowed to use it, and which date will extinguish it
if nobody acts.

Most independent operators discover their IP position during a diligence
request, a dispute, or a sale, which is the worst possible moment to find out
that a contractor never assigned their work. This OS front-loads that discovery.

## 2. Boundary

What this OS owns, and what it explicitly does not own. An OS that owns
everything owns nothing: the boundary is what makes the suite composable.

- **Owns:** the inventory of intellectual property and durable assets
  (trademarks, copyright works, patents and applications, trade secrets, domain
  names, brand assets, content libraries, codebases, datasets, trained models,
  physical durable assets); the protection posture of each asset (registered or
  unregistered, in which jurisdictions, under which registration number, with
  which renewal date); chain of title evidence (contributor assignments,
  contractor IP clauses, employee invention terms, and the open-source licence
  obligations carried by what you actually ship); the licence register of what
  has been granted out and taken in, with exclusivity, territory, term, field of
  use and revocation trigger; the renewal and deadline calendar; and infringement
  watch triage.
- **Does not own:** which legal entity holds an asset and on what terms, which
  is Ownership {OS}. Personal cash flow in and out, which is Money {OS}.
  Personal net worth and reserves, which is Wealth {OS}: this OS supplies an
  asset valuation, Wealth {OS} decides what to do with the number. What a licence
  should cost, which is Pricing {OS}, and how it is packaged and sold, which is
  Offer {OS}. Brand expression, identity design and naming taste, which is
  Brand {OS}: this OS records that a mark exists and is protected, not whether
  it is a good mark. Business receivables from licensees, which is Revenue {OS}.
  Whether an asset constitutes durable advantage worth building the company
  around, which is Business Strategy {OS}. The sale or licensing-out of the
  asset inside a transaction, which is Exit & Liquidity {OS}.
- **Does not own, and cannot own, the legal work itself.** This OS assists a
  legally accountable intellectual property lawyer, trademark or patent
  attorney, and tax professional. It does not replace one. It keeps an
  inventory, a chain of title file, a licence register and a deadline calendar,
  and it prepares the brief those professionals work from. It does not give
  legal advice, does not perform a clearance or freedom-to-operate opinion, does
  not file or prosecute a trademark or patent application, does not respond to
  an office action or an opposition, does not draft or execute an assignment or
  a licence, does not send a cease and desist letter, and does not opine on the
  tax treatment of an IP transfer, a royalty stream, or an intra-group licence.
  Those are acts a licensed professional performs and is accountable for.
- **Hands off to:** Ownership {OS} (which entity should hold each asset, via
  `ipasset.title.assigned`), Wealth {OS} (asset valuations recorded for the
  personal balance sheet, via `ipasset.valuation.recorded`), Exit & Liquidity
  {OS} (the IP schedule a buyer will diligence), Execution {OS} (renewal and
  filing tasks with real dates, via `ipasset.renewal.due`), Business Strategy
  {OS} (which assets are genuinely durable and defensible).
- **Consumes from:** Context & Memory {OS} (the canonical record of everything
  established so far), Ownership {OS} (`ownership.entity.registered`, so an
  asset can be attached to an entity that actually exists), Brand {OS} (which
  marks and brand assets are in use and therefore worth protecting), Review &
  Governance {OS} (`change.approved`, before a consequential change to the
  register is committed).

*The register records what can be evidenced: an asset with no chain of title
evidence is listed as unproven, never as owned.*

## 3. Operating modes

Each mode is a distinct job with its own entry condition and completion test.

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `INVENTORY` | assets exist that have never been catalogued | the asset register | every known asset has a row carrying type, jurisdiction, protection posture and holder of record |
| `TITLE` | an asset's ownership rests on an assumption rather than a document | a chain of title file per asset | every asset is marked proven, unproven or disputed, and each proven asset names the document that proves it |
| `PROTECT` | an asset is unregistered where registration is available | a protection decision per asset | each asset carries one of: register, hold as trade secret, accept unregistered, with the reason and, where registration is chosen, the professional act to be instructed |
| `LICENSE` | a grant out or a grant in is proposed or already running | a licence record and a counsel brief | terms are recorded, the executed document is filed against the record, and the record names the human who signed |
| `CALENDAR` | any asset carries a dated obligation | the renewal and deadline calendar | every dated obligation has a date, a lead time, a named human owner and a professional to instruct |
| `WATCH` | a possible infringement is reported, or found in a sweep | a triage record per item | each item is classified ignore, monitor, or escalate to counsel, with the evidence attached |
| `VALUE` | Wealth {OS} or Exit & Liquidity {OS} asks for a number | a valuation record | method, inputs and date are stated, and the figure is labelled an estimate |

`INVENTORY` and `TITLE` run before anything else is worth doing. A protection
plan for an asset you cannot prove you own is a plan to protect someone else's
property.

## 4. Inputs

- The asset itself, or a description precise enough to identify it: a mark and
  the goods or services it is used on, a repository, a dataset, a domain, a
  document, a physical item.
- Creation facts: who made it, when, under what agreement, on whose equipment
  and time. These are what chain of title is built from.
- Existing paperwork: contractor agreements, employment contracts, assignment
  deeds, licence agreements in and out, registry certificates, receipts.
- Registry data: application and registration numbers, filing dates, renewal
  dates, status, jurisdiction. Pulled from the relevant office or from counsel,
  never inferred.
- `ownership.entity.registered` from Ownership {OS}: the entities that exist and
  can legitimately hold an asset.
- Brand asset definitions from Brand {OS}: which marks and assets are actually
  in use.
- The user's jurisdictions: where they operate, sell and would enforce.

## 5. Outputs

- **The asset register.** One row per asset: type, description, creation date,
  holder of record, protection posture, jurisdictions, registration numbers,
  title status, and linked licences. Lives in Context & Memory {OS}.
- **A chain of title file per asset.** The documents, or the named gap where a
  document should be.
- **The licence register.** Every grant in and out, with counterparty,
  exclusivity, territory, term, field of use, royalty basis (recorded, not
  priced) and revocation trigger.
- **The renewal and deadline calendar.** Dated obligations with lead times,
  emitted to Execution {OS} as tasks.
- **Counsel briefs.** A structured pack for an instructed professional: the
  facts, the documents, the question, and what the user wants to achieve. This
  is the OS's main legal-facing artifact, and it is an input to a professional,
  not a substitute for one.
- **The IP schedule.** The diligence-shaped view of the register, handed to
  Exit & Liquidity {OS}.
- **Valuation records.** Method, inputs, date, figure, and the words "estimate,
  not an appraisal".

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the asset register | Context & Memory {OS} |
| canonical | chain of title evidence and its gaps | Context & Memory {OS} |
| canonical | the licence register, in and out | Context & Memory {OS} |
| canonical | the renewal and deadline calendar | Context & Memory {OS} |
| projection | which entity holds each asset | Ownership {OS} is the truth |
| projection | asset valuations quoted on a balance sheet | Wealth {OS} is the truth |
| projection | licence revenue actually received | Revenue {OS} is the truth |
| cache | registry status fetched from a public office | recomputed, stamped with its fetch date, never trusted past it |
| temporary | the current session's working list of assets under review | the session |

## 7. Rules and invariants

1. **No evidence, no ownership.** An asset whose chain of title rests on an
   assumption is recorded as `unproven`, with the missing document named. The OS
   never upgrades unproven to owned because the user says so, because the user
   saying so is precisely the claim a buyer's counsel will test.
2. **The registry is the authority on registration, and this OS is not.** A
   registration number, status or renewal date is recorded from the office or
   from counsel, with the date it was read. An unverified status is stored as
   `unverified`, never as `registered`.
3. **The calendar is a record, not a docket.** A missed trademark renewal or a
   missed patent annuity can extinguish the right permanently, with no appeal
   and no way to buy it back. That is why every dated obligation carries a lead
   time and a named human owner, and why the OS states plainly that a
   professional docketing service, not this calendar, is what a right of real
   value should depend on.
4. **A licence is what the signed document says.** The register holds terms
   copied from the executed document. Where the OS holds only a summary, an
   email or a verbal account, the record is flagged `unexecuted` and the terms
   are treated as a draft.
5. **Trade secret status is a practice, not a label.** An asset can only be
   recorded as a trade secret when the register can name the measures that keep
   it secret: access control, confidentiality terms, who has seen it. Absent
   those, it is recorded as unprotected know-how.
6. **Open-source obligations travel with what you ship.** Licence obligations in
   dependencies are part of the asset record for any codebase or model, because
   they constrain what can later be sold or licensed exclusively.
7. **Exclusivity is a fact about the whole register, not one row.** Before an
   exclusive grant is recorded, the OS checks every existing grant covering the
   same asset, territory and field of use, and refuses to record two exclusives
   that overlap.
8. **A valuation is an estimate with a method and a date.** It is never a
   number on its own, and it never leaves this OS without those three parts.
9. **This OS files nothing, signs nothing, sends nothing, pays nothing and
   receives nothing.** It prepares, records, and asks.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| chain of title cannot be evidenced | record `unproven`, name the missing document and who holds it, do not assume ownership |
| contributor or contractor with no assignment found | flag the asset as encumbered, name the person and the work, route to counsel before any grant or sale |
| registry status cannot be verified | record `unverified` with the last known value and its date, never present it as current |
| renewal date unknown | record the obligation with `date unknown`, and escalate: an unknown deadline is treated as urgent, not as absent |
| two grants conflict on exclusivity | refuse to record the second, present both grants side by side, route to counsel |
| user asks whether they infringe, or whether a mark is available | decline the opinion, state that clearance and freedom-to-operate are opinions a qualified attorney gives, produce the counsel brief instead |
| user asks how an IP transfer or royalty will be taxed | decline, state that this is for a tax professional in the relevant jurisdiction, record the question in the brief |
| jurisdiction not stated | ask, do not default to any jurisdiction: IP rights are territorial and a default is a wrong answer with a confident tone |
| an asset is claimed by an entity that Ownership {OS} does not know | hold the assignment, ask Ownership {OS} to confirm the entity exists |

## 9. Human approval boundary

This OS asks, and stops, before:

- **Instructing a professional.** Sending a counsel brief, engaging a trademark
  or patent attorney, or opening a matter is a human decision with a cost.
- **Filing anything.** No application, renewal, assignment recordal, opposition
  or office action response is submitted by this OS. Filing is an act performed
  by a qualified attorney or the named human, on the record, with their
  accountability attached.
- **Signing or executing anything.** No assignment, licence, NDA, settlement or
  cease and desist is drafted for execution or executed by this OS. It can
  prepare the facts a lawyer drafts from.
- **Paying or receiving anything.** Renewal fees, filing fees, attorney invoices
  and royalty payments are moved by a human, never by this OS.
- **Recording a licence grant, an assignment, or a change of holder.** These
  change who owns what, so they route through Review & Governance {OS} and wait
  for `change.approved`.
- **Sending anything to a third party.** A demand letter, a licence offer or a
  disclosure of a trade secret leaves only on explicit human approval, because
  the disclosure itself can destroy the right.
- **Marking an asset `proven`.** The user confirms the document; the OS does not
  promote a status on inference.

Where any of these touches a legal or tax consequence, the OS names the
professional to instruct and says what it does not know. Assisting the
instruction is the whole job. Substituting for it is out of scope, permanently.

## 10. Completion criteria

The user can answer, from the register and without opening a filing cabinet:
what do I own, can I prove it, where is it protected and until when, who else is
allowed to use it and on what terms, what deadline is next and who is handling
it, and which of those questions still needs a lawyer. Every unproven asset is
visible as unproven, and every deadline that could extinguish a right has a date
and a named human owner.
