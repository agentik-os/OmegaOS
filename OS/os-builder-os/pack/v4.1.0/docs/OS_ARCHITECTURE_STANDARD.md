# Ultimate OS architecture standard

## 1. Purpose contract

Every OS defines:

- who it serves;
- what outcomes it helps produce;
- what it does not do;
- what risks require escalation;
- how success is observed.

## 2. Core operating model

The OS must expose its central transformation loop.

Generic pattern:

```text
NOTICE
→ DIAGNOSE
→ PRIORITIZE
→ PLAN
→ ACT
→ OBSERVE
→ REVIEW
→ ADAPT
```

Domain-specific OSs should replace generic labels with their actual mechanisms.

## 3. Domain ontology

Define entities, states, events, relations and artifacts.

No workflow may use an undefined state or ambiguous entity.

## 4. Capability map

Every capability includes:

- purpose;
- supported jobs;
- inputs;
- outputs;
- state read and write behavior;
- dependencies;
- failure modes;
- permissions;
- evidence and rationale;
- owning command and workflow.

## 5. Command contract

Every command includes:

- exact syntax;
- purpose;
- when to use;
- required and optional inputs;
- defaults;
- outputs;
- side effects;
- errors;
- examples;
- capability mapping;
- workflow mapping;
- permission level.

## 6. Workflow contract

Every workflow includes:

- trigger;
- preconditions;
- steps;
- branches;
- loops;
- state transitions;
- artifacts;
- checkpoints;
- failure recovery;
- completion criteria;
- observability events.

## 7. Diagnostics and decision tables

Avoid generic advice. Build explicit diagnostic logic and context-sensitive decisions.

## 8. Memory and state

Define:

- ephemeral state;
- session state;
- project state;
- user-approved durable memory;
- evidence state;
- workflow checkpoints;
- version migrations;
- deletion and reset behavior.

## 9. Metrics and observability

Measure:

- user outcome progress;
- workflow completion;
- command success;
- confidence and uncertainty;
- exceptions;
- escalations;
- drift;
- source freshness;
- regression.

## 10. Boundaries and escalation

The OS must know when to:

- continue;
- ask for a critical missing input;
- reduce confidence;
- provide alternatives;
- route to another OS;
- route to a qualified human;
- refuse unsafe action.

## 11. Inter-OS collaboration

Handoffs must be:

- optional;
- typed;
- discoverable;
- traceable;
- disableable;
- user-controlled;
- safe under dependency failure.

## 12. Continuous update

Every material rule should identify which source or assumption could invalidate it.
