# Workflow: Compile a definition pack

**Mode:** `NEW`
**Produces:** a validated `blueprint/state.json` and a frozen, checksummed
handoff.

## Trigger

An idea is about to become a build, and no definition pack exists for it. Also
triggered when the DISCOVER group closes with a validated opportunity and the
user asks what to build.

## Preconditions

- The user can state the outcome the product is supposed to produce.
- Any existing constraint the user is not free to change is available.
- The project directory is writable, and `omega-blueprint` is installed.

## Steps

1. **Initialise state.** `omega-blueprint init blueprint/state.json` with the
   project id, name, namespace and the request. No compiling before there is a
   state file to write into.
2. **Ingest evidence.** Read what the DISCOVER group produced, plus any
   existing repository docs. Record each item with a source ID. Anything
   without a source is recorded as ASSUMPTION, whatever it sounded like.
3. **Fix scope and identity.** Product thesis, outcomes, non-goals, actors,
   permissions, trust boundaries. This is G01 and G05 territory and everything
   later depends on it.
4. **Raise questions once.** At most three, each only where a wrong answer
   changes product promise, economics, trust, legal exposure, data ownership,
   irreversible architecture or major scope. Register every other open point as
   an ASSUMPTION with a reversal trigger.
5. **Compile the product definition.** Capabilities, requirements, actions and
   flows, interface contracts, acceptance criteria. One record per statement,
   each with an ID.
6. **Compile the technical definition.** Domain objects and invariants, data
   governance, architecture coherence, API and event contracts, AI behaviour
   and evaluability, security and privacy requirements, NFRs and operations.
7. **Compile the release definition.** What the first release contains, what it
   deliberately excludes, and what metric decides it worked.
8. **Trace.** Every critical requirement linked up to an outcome and down to an
   acceptance criterion. Orphans in either direction are fixed here, not later.
9. **Checkpoint.** `omega-blueprint checkpoint` with a current and next
   pointer, before the gate pass and before any risk of context loss.
10. **Gate.** `omega-blueprint validate`. Repair every critical and high issue.
    Re-run until it exits zero.
11. **Freeze.** Ask the user to approve the freeze, then emit the handoff with
    version, revision and checksum.
12. **Stop.** Print `BLUEPRINT COMPLETE, STEPPER READY` and name the next OS:
    Design {OS} when the product has a UX surface, otherwise Stepper {OS}.

## Completion test

```bash
omega-blueprint validate blueprint/state.json   # must exit 0
omega-blueprint status   blueprint/state.json   # every gate G01 to G20 green
```

And, by inspection: a frozen handoff file exists carrying a version and a
checksum, no critical UNKNOWN or CONFLICT is ownerless, and critical records
report 100% trace coverage in both directions.

## Failure paths

| What happens | What the workflow does |
|---|---|
| a gate stays red on missing information | register the UNKNOWN with an owner, keep compiling everything else, do not freeze |
| two sources contradict on a material point | record a CONFLICT with both sources and escalate as a decision, never average |
| the user asks for implementation steps | decline, name Stepper {OS}, offer the handoff |
| output limits force a split | mark the pack INCOMPLETE, list finished and remaining sections, resume at the exact next section with IDs preserved |
