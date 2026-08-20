# Workflow: Map the consents a transaction needs

Produces the list of people who must sign or waive before a proposed
transaction can complete, and the clause that creates each requirement.

## Trigger

Any of:

- Exit & Liquidity {OS} emits `exit.structure.proposed`
- a new issuance, a share transfer or a partner admission is contemplated
- a buyer's counsel asks who can block the deal

## Steps

1. **State the transaction precisely.** Entity, instrument, quantity,
   counterparty, and whether it is an issuance, a transfer, a redemption or a
   full sale. A consent map computed against a vague transaction is worthless,
   so this step refuses to proceed on "we might sell".
2. **Load the terms register for the entity.** Reads: the terms register built by
   `/terms`. If pre-emption, transfer restriction, drag, tag or consent clauses
   are recorded as `unknown` rather than `absent`, stop and run the extraction
   first. An unknown clause is not the same as no clause.
3. **Walk the clauses against the transaction.** For each tracked term class,
   determine whether this specific transaction triggers it, and record the
   clause reference and the operative text alongside the determination. Writes:
   the draft consent map, temporary.
4. **Resolve the holders.** For every triggered clause, name the actual people or
   entities whose signature or waiver is required, from the cap table. Flag
   separately any holder who is unknown, unreachable, deceased, dissolved or in
   dispute, because those are the ones that delay a closing.
5. **Check the chain.** Where a holder is itself an entity, walk up the entity
   map to the human or board that can actually sign for it, and note the
   authorisation each level requires (board resolution, shareholder resolution,
   trustee decision). Reads: the entity map.
6. **Mark the legal boundary.** Every determination in the map is a reading of a
   clause against a fact pattern, which is a legal act. Label the whole map as
   requiring confirmation by the entity's lawyer, and produce a
   `/counsel-pack` containing the transaction description, the clauses and the
   determinations. This OS never certifies that a consent is or is not required.
7. **Approval gate.** Present the map to the user. On approval, hand it to Exit &
   Liquidity {OS}. Without approval it stays in the session. Emits on approval:
   the consent map to Exit & Liquidity {OS}, and `ownership.term.flagged` for
   any clause that is unusual, expiring or in conflict with another.
8. **Schedule the chase.** Each required signature or waiver becomes a dated
   obligation with a responsible human and a lead time. Emits:
   `ownership.obligation.due` to Execution {OS}.

## Completion test

- The transaction is described with entity, instrument, quantity, counterparty
  and type, and none of those is blank.
- Every tracked term class is marked triggered or not triggered, with a clause
  reference on both outcomes.
- No clause in the map is in state `unknown`.
- Every triggered clause names a real signatory, or explicitly flags that the
  holder is unknown or unreachable.
- The map carries the label that a lawyer must confirm it, and a `/counsel-pack`
  exists for that confirmation.
- Exit & Liquidity {OS} received the map only after user approval.
