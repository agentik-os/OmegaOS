# Workflow: Recover canonical truth

**Mode:** `RECOVER`
**Produces:** one canonical pack rebuilt from prior sources, with every
recovered record labelled, traced and reconciled.

## Trigger

The project exists and is possibly already in production, but its definition
does not exist in one place. Symptoms: three specs disagreeing, decisions
living only in a chat thread, a codebase that is the only real source of truth,
a handover from someone who left.

## Preconditions

- Read access to whatever prior sources exist: repository, docs, tickets,
  exports, prior packs.
- The user can name which sources are authoritative when two disagree, or can
  say that they do not know, which is itself recorded.

## Steps

1. **Inventory the sources.** List every artifact that might carry truth, with
   its date and its author. Assign each a source ID. An undated source is
   recorded as undated, not guessed.
2. **Rank authority.** The user declares the order, or the workflow proposes
   one (running code, then approved specs, then tickets, then chat) and the
   user confirms it. This ranking is itself a DECISION record.
3. **Extract, do not summarise.** Pull discrete statements out of each source
   as records with IDs. A paragraph becomes several records or none.
4. **Label honestly.** A statement supported by running code is a FACT with the
   code as its source. A statement found in a stale spec with no confirmation
   is an ASSUMPTION. Never promote by tidiness.
5. **Reconcile.** Where sources disagree, record a CONFLICT holding both
   sides and the authority ranking's verdict. Material conflicts go to the user
   as a decision; the rest resolve by the ranking and say so.
6. **Find the holes.** Run the section coverage: what a complete pack requires
   and no source supplies. Each hole becomes an UNKNOWN with an owner.
7. **Checkpoint.** Recovery is long. `omega-blueprint checkpoint` after each
   source is fully extracted.
8. **Trace.** Link recovered requirements to outcomes and acceptance criteria.
   Recovery routinely finds requirements with no outcome and outcomes with no
   requirement; both are reported, neither is invented away.
9. **Gate.** `omega-blueprint validate`. Recovery packs usually open with red
   gates; that is the finding, not a failure of the workflow.
10. **Report before freezing.** Present the recovered pack, the conflicts, the
    unknowns and the red gates. Freezing waits on the user closing the material
    unknowns.

## Completion test

```bash
omega-blueprint validate blueprint/state.json
omega-blueprint status   blueprint/state.json
```

Passes when: every inventoried source has been extracted or explicitly
excluded with a reason, every recovered record carries a source ID and a
label, every conflict is either resolved by the recorded authority ranking or
owned by a named person, and the remaining unknowns are listed rather than
filled in.

A recovery that produces a pack with no unknowns from sources that were known
to be incomplete has fabricated something. That outcome fails this test.

## Failure paths

| What happens | What the workflow does |
|---|---|
| a source is unreadable or lost | record it as excluded with the reason, do not silently drop it |
| the code contradicts every written spec | code wins as FACT, the specs become SUPERSEDED with history, the divergence is reported |
| the user cannot rank authority | record that, treat every disagreement as a CONFLICT, and do not resolve on the workflow's own preference |
| recovery reveals the product is not what the user believed | stop, report, and let the user decide between `REVISE` and a new definition |
