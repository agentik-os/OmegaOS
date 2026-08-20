# Workflow: Revise and propagate

**Mode:** `REVISE`, closing with `DELTA`
**Produces:** a new frozen version, a semantic delta, and a named impact set
for every downstream OS that already read the previous version.

## Trigger

A decision changed after a handoff was frozen. The change can originate with
the user, or arrive from downstream: Design {OS} found a flow the definition
cannot support, Prototype {OS} refuted an assumption, Stepper {OS} found a
requirement it cannot decompose, Builder {OS} hit a contradiction, Quality &
Evaluation {OS} found a requirement with no testable criterion, Security {OS}
found a requirement that cannot be met safely.

## Preconditions

- A frozen version exists and is identified by version and checksum.
- The change request names the record IDs it affects, or enough detail to find
  them.

## Steps

1. **Locate the record.** Find the DECISION, requirement or contract that is
   changing, by ID. A change request that cannot be attached to an ID is not
   yet a revision; it goes back for specifics.
2. **Record the reason.** What evidence forces the change, and where it came
   from. A revision with no reason is indistinguishable from drift.
3. **Supersede, never overwrite.** The old record stays, marked SUPERSEDED,
   pointing at its replacement. History is the only defence against relitigating
   the same decision next quarter.
4. **Walk the dependents.** Every record that traces to the superseded one is
   visited. Each is updated, explicitly confirmed as unaffected, or flagged as
   a new decision. Nothing is left unvisited.
5. **Check the gates the change touches.** A change to data governance re-opens
   G10, a change to an interface contract re-opens G07 and G11, and so on. Run
   `omega-blueprint validate` and read the result, do not assume.
6. **Checkpoint and cut a version.** `omega-blueprint checkpoint`, then freeze a
   new version with its own checksum. The previous frozen artifact is not
   touched.
7. **Produce the delta.** Semantic diff between the two versions, each changed
   record classified by blast radius: definition only, design impacting,
   plan impacting, code impacting.
8. **Notify downstream by name.** State which OS must re-read what: Design {OS}
   for surface and flow impact, Stepper {OS} for plan impact, Builder {OS} for
   in-flight steps, Quality & Evaluation {OS} for acceptance criteria that
   moved, Security {OS} for changed trust boundaries, Release {OS} for a changed
   release definition.

## Completion test

```bash
omega-blueprint validate blueprint/state.json
omega-blueprint delta <previous-version> <new-version>
```

Passes when: the superseded record is retained with history, every dependent
record has been visited and carries a verdict, the gates the change touches have
been re-evaluated and are green, a new frozen version exists alongside the old
one, and the delta names an affected artifact for every downstream OS that read
the previous version.

## Failure paths

| What happens | What the workflow does |
|---|---|
| the change makes a shipped requirement impossible | record a CONFLICT, escalate to the user, do not quietly drop the requirement |
| a dependent record's owner is unavailable | flag it as an open decision in the delta rather than deciding for them |
| the change arrives while Builder {OS} is mid step | emit the delta with the in-flight step named, and let Builder finish or abort that step; never edit its contract underneath it |
| the requester wants the frozen artifact edited in place | refuse, explain that downstream plans are bound to the checksum, and cut a version instead |
