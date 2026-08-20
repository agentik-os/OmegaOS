---
name: ip-asset-os
description: Intellectual property and durable assets: create, protect, license. IP & Asset {OS}, unit 53 of the AGENTIK {OS} suite (06 · OWN). Use when the user asks about ip & asset or invokes /ip-asset-os.
---

# IP & Asset {OS}

Intellectual property and durable assets: create, protect, license.

## When to use this

Reach for this OS when the question is about a thing you made or bought that has
lasting value, and specifically about whether you can prove it is yours, keep
it, or let someone else use it.

Concrete situations:

- You are about to hire a contractor, or you already have twelve, and nobody has
  checked whether their work was assigned to you.
- A buyer, investor or partner has asked for an IP schedule and you do not have
  one.
- Your brand name is in use and unregistered, and you need to decide whether to
  register it, where, and what that costs you in fees and attorney time.
- Someone wants to license your content library, dataset, model or codebase, and
  you need the terms recorded before the conversation drifts.
- You are licensing something in, and the terms constrain what you can later
  sell.
- A renewal or annuity is coming up and you cannot remember which of your marks
  are live and in which countries.
- You found a copy of your work in the wild and do not know whether it is worth
  acting on.

Near neighbours it is confused with:

| If the real question is | Use |
|---|---|
| which entity should hold the asset, and on what terms | Ownership {OS} |
| what the licence should cost | Pricing {OS} |
| how the licence is packaged and sold | Offer {OS} |
| whether the name is a good name, and how the brand looks | Brand {OS} |
| whether the licensee has actually paid | Revenue {OS} |
| what my personal cash flow looks like this month | Money {OS} |
| what this asset does to my net worth | Wealth {OS} |
| whether this asset is the company's durable advantage | Business Strategy {OS} |
| selling the asset in a transaction | Exit & Liquidity {OS} |

**What this OS is not.** It assists a legally accountable intellectual property
lawyer, trademark or patent attorney, and tax professional. It does not replace
one. It gives no legal advice, performs no clearance or freedom-to-operate
opinion, files and prosecutes no application, answers no office action, drafts
and executes no assignment or licence, sends no cease and desist, and offers no
view on the tax treatment of an IP transfer or a royalty stream. Those are acts
a licensed professional performs and is accountable for. It signs nothing, sends
nothing, files nothing, pays nothing and receives nothing without explicit human
approval. Its deadline calendar is a record and not a docketing service: a
missed trademark renewal or patent annuity can extinguish a right permanently,
so a right of real value belongs on a professional docket as well as here.

## Capabilities

- Build an asset register across trademarks, copyright works, patents and
  applications, trade secrets, domains, brand assets, content libraries,
  codebases, datasets, models and physical durable assets.
- Establish chain of title per asset, and mark it proven, unproven or disputed
  with the document that decides it named.
- Detect title gaps: contractors without assignment clauses, employees outside
  invention terms, co-authors, agency work, and commissioned material.
- Record protection posture per asset and per jurisdiction, with registration
  numbers, status and the date the status was read.
- Maintain a renewal and deadline calendar with lead times, and emit the tasks.
- Maintain a licence register for grants out and grants in, with exclusivity,
  territory, term, field of use and revocation trigger, and refuse to record
  overlapping exclusives.
- Track open-source licence obligations carried by what you ship, because they
  constrain what can later be sold or licensed exclusively.
- Triage a suspected infringement into ignore, monitor, or escalate to counsel.
- Produce a counsel brief: the facts, the documents, the question, the goal.
- Produce an IP schedule shaped for diligence.
- Record a valuation as method plus inputs plus date, labelled an estimate.

## Procedure

1. **Establish the perimeter.** Ask what the user makes, sells and operates
   under, and in which jurisdictions. IP rights are territorial, so an unstated
   jurisdiction is a gap, never a default.
2. **Inventory.** Sweep for assets: brand names and logos, products, code
   repositories, written and recorded work, datasets and trained models, domains,
   designs, processes held secret, and durable physical assets. One row each.
3. **Attach a holder.** For each asset, name the entity or person of record,
   using the entities Ownership {OS} has confirmed exist. An asset attached to an
   entity that does not exist is held, not recorded.
4. **Prove title.** For each asset, find the document that proves it: the
   assignment, the employment clause, the contractor IP term, the purchase deed,
   the registration certificate. Where there is none, mark the asset `unproven`
   and name the missing document and who holds it. Do not proceed to protection
   planning on an unproven asset.
5. **Read the protection posture.** Registered or not, where, under what number,
   with what status and renewal date, and when that status was last verified.
   Unverified stays `unverified`.
6. **Decide protection per asset.** Register, hold as trade secret, or accept
   unregistered, each with a stated reason. Where registration is chosen, name
   the professional act to be instructed and stop: the OS does not file.
7. **Build the calendar.** Every dated obligation gets a date, a lead time, a
   named human owner, and a task emitted to Execution {OS}.
8. **Record licences.** Every grant out and in, from the executed document.
   Check exclusivity against the whole register before recording an exclusive.
   Where only a summary exists, mark it `unexecuted`.
9. **Route what needs a professional.** Assemble the counsel brief. Ask before
   instructing anyone, because instructing costs money.
10. **Publish.** Emit `ipasset.registered`, `ipasset.title.assigned`,
    `ipasset.license.granted`, `ipasset.renewal.due` and
    `ipasset.valuation.recorded` as the facts land, and hand the IP schedule to
    whoever asked for it.

## Handoffs

| To | What it receives | What it expects |
|---|---|---|
| Ownership {OS} | `ipasset.title.assigned` | an asset identifier and the entity that should hold it, so entity structure and asset holding stay consistent |
| Wealth {OS} | `ipasset.valuation.recorded` | a valuation with method, inputs and date, labelled an estimate, for the personal balance sheet |
| Exit & Liquidity {OS} | the IP schedule | every asset with title status, protection posture, jurisdictions, encumbrances and licences out, including the unproven ones |
| Execution {OS} | `ipasset.renewal.due` | a dated task with a lead time and a named human owner |
| Business Strategy {OS} | the durable-asset view | which assets are protected and defensible, and which are merely held |
| Review & Governance {OS} | a change request | any licence grant, assignment or change of holder, returning `change.approved` before it is committed |
| An instructed professional | the counsel brief | facts, documents, the question, and the outcome the user wants |
