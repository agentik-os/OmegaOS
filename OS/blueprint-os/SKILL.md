---
name: blueprint-os
description: The product-definition compiler: a complete, traceable definition pack. Blueprint {OS}, unit 20 of the AGENTIK {OS} suite (03 · BUILD). Use when the user asks about blueprint or invokes /blueprint-os.
---

# Blueprint {OS}

The product-definition compiler: a complete, traceable definition pack.

## When to use this

Use Blueprint {OS} when:

- an idea is about to become a build and nobody has written down what it
  actually is, for whom, with what invariants;
- a project already exists but its truth is scattered across tickets, chats and
  three outdated specs, and you need one canonical version (`RECOVER`);
- a new module is being added to a defined product and you need the impact set,
  not just the feature (`EXTEND`);
- a decision changed and you need every dependent record to know
  (`REVISE`, `DELTA`);
- someone claims a spec is ready and you want the gates to say so (`AUDIT`).

Do not use it when:

- the question is how the product should look or behave on screen. That is
  Design {OS}.
- the question is whether an assumption survives a working artifact. That is
  Prototype {OS}.
- the question is what to build first. That is Stepper {OS}.
- the question is whether the idea is worth building at all. That is the
  DISCOVER group: Validation {OS}, Customer Discovery {OS}, Market Research
  {OS}.

The near neighbour people confuse it with is Stepper {OS}. Blueprint answers
what is true about the product. Stepper answers in what order it gets built.
Blueprint that drifts into steps produces a second plan that nothing verifies.

## Capabilities

- Compiles an idea plus project context into a full definition pack: scope,
  actors, permissions, capabilities, requirements, domain objects, invariants,
  data governance, API and event contracts, AI behaviour, security and privacy
  requirements, NFRs, acceptance criteria, release definition.
- Maintains an epistemic ledger: every statement labelled FACT, DECISION,
  ASSUMPTION, PROPOSAL, UNKNOWN, CONFLICT, DEFERRED or SUPERSEDED.
- Allocates and preserves a stable monotonic ID space across revisions.
- Builds bidirectional traceability from outcome to requirement to acceptance
  criterion, and reports orphans in either direction.
- Evaluates 20 quality gates (G01 scope and identity through G20 artifact and
  continuation integrity) and refuses readiness while a critical gate is red.
- Freezes a versioned, checksummed handoff for Design {OS} and Stepper {OS}.
- Produces a semantic delta and impact set between two versions.
- Checkpoints its own continuation pointer so a compaction costs nothing.

## Procedure

1. **Intake.** Collect the idea, the constraints, the existing artifacts and
   the DISCOVER evidence. Initialise or load `blueprint/state.json`.
2. **Classify.** Every incoming statement gets a label and an ID. Evidence
   without a source becomes an ASSUMPTION, visibly.
3. **Ask, at most three times.** Only where a wrong answer changes product
   promise, economics, trust, legal exposure, data ownership, irreversible
   architecture or major scope. Everything else proceeds on a registered
   assumption.
4. **Compile, section by section.** Product definition first, then technical
   definition, then acceptance and release definition. Each section writes
   records, not prose.
5. **Trace.** Link every critical requirement up to its outcome and down to its
   acceptance criteria. Fix orphans as they appear, not at the end.
6. **Gate.** Run `omega-blueprint validate`. A non-zero exit outranks any
   narrative claim of progress.
7. **Checkpoint.** `omega-blueprint checkpoint` before any long pass or any
   risk of context loss.
8. **Freeze.** With every gate green, emit the handoff with version, revision
   and checksum, then print `BLUEPRINT COMPLETE, STEPPER READY` and stop.

## Handoffs

| Receives from | What arrives |
|---|---|
| Validation {OS}, Customer Discovery {OS}, Market Research {OS} | validated demand, the customer, the market, each with a source |
| Business Model {OS} | economics the product must respect |
| Context & Memory {OS} | prior decisions on this project |
| any downstream OS | a change request, entering as a decision, never as an edit |

| Hands to | What it expects |
|---|---|
| Design {OS} (21) | the frozen pack: requirements, actors, permissions, domain objects, AI behaviour contracts, the IDs it must trace back to |
| Stepper {OS} (23), when there is no UX surface | the same frozen pack, read directly |

Design {OS} and Stepper {OS} both read a pinned version. Neither is ever
pointed at a moving latest. A pack that changed after they read it produces a
new version and a delta, and they are told which records moved.
