# Workflow: Build the entity map

Produces the first honest picture of what exists and who holds what, including
the parts nobody can currently evidence.

## Trigger

Any of:

- first run of Ownership {OS}
- an entity is incorporated, acquired, dissolved or discovered
- a professional (lawyer, corporate secretary, accountant) asks for a structure
  chart and the user cannot produce one

## Steps

1. **Enumerate entities, not positions.** Ask the user to list every legal
   entity they are connected to: operating companies, holding companies,
   partnerships, trusts, and personal holdings. Include dormant entities and
   entities held indirectly through a spouse, a trust or a nominee. Touches:
   nothing yet, this is collection.
2. **Record identity per entity.** Legal name, entity type, jurisdiction of
   registration, registration number, incorporation date, registered office or
   agent. Any field the user cannot produce is written as `unknown`, never
   inferred from the entity name or the country the user lives in. Writes:
   entity records, staged.
3. **Attach positions.** For each entity, capture holder, instrument, quantity
   and class. Do not compute a percentage at this step. Writes: position
   records, staged, all with verification state `unknown`.
4. **Establish denominators.** For each entity, determine the issued,
   outstanding and fully diluted share or unit counts, and whether an
   unallocated option pool sits inside the fully diluted figure. Ask the
   corporate secretary or read the statutory register rather than deriving it.
   Writes: denominator per entity.
5. **Compute percentages, labelled.** Now compute each position's percentage,
   and emit it with its denominator name and its verification state on the same
   line. Reads: steps 3 and 4.
6. **Request the source documents.** Produce the document request list: articles,
   shareholder or operating agreement, subscription agreements, grant notices,
   share-transfer forms, statutory register extracts, trust deeds. One line per
   missing document per entity. Touches: the request list, handed to the user.
7. **Draw the map.** Render entities as nodes and holdings as edges labelled by
   instrument and percentage, with unverified edges visibly marked. Reads:
   steps 2 to 5.
8. **Approval gate.** Present the map, the unknowns and the document request
   list. Nothing becomes canonical until the user approves it. Writes on
   approval: entity and position records to Context & Memory {OS}, and
   `ownership.entity.registered` per entity.
9. **Open the calendar.** For each registered entity, hand its jurisdiction and
   entity type to `cap-table-reconcile.md` and to the obligations calendar so the
   filing schedule can be built with the corporate secretary. Emits:
   `ownership.obligation.due` once dates are confirmed by that human.

## Completion test

- Every entity the user named has a record with type, jurisdiction and
  registration number, or an explicit `unknown` in that field.
- Every position carries a class, a quantity, a denominator name and a
  verification state.
- No percentage appears anywhere in the output without its denominator.
- The document request list exists and names, per entity, exactly which
  documents would move facts from `user-asserted` to `verified`.
- The user approved the map before it was written to Context & Memory {OS}.
