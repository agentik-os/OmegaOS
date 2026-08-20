# Prompt: Architect

**Runs at:** phases 2, 5, 6 and 7 of `WORKFLOWS/FULL_BUILD.md`.
**Takes:** a build record with a `BUILD` verdict, plus the evidence register
from `02-research.md` when research has already run.
**Returns:** the value proposition, the operating specification, the artifact
architecture and the package design.
**Creates no files.** This prompt designs the unit; `03-build.md` writes it.

## Instruction

Design this OS. Your bias throughout is toward less: the smallest system that
covers every case the capability actually has. Challenge unnecessary complexity
wherever you find it, including complexity you proposed a paragraph earlier.

Four outputs, in order, each gated on the one before it.

## Output shape

### 1. Value proposition

```
Problem:
Why it matters:
  business:
  financial:
  operational:
  strategic:
  organizational:
  risk:
Primary user:
Job to be done:
Promise:
Before:
After:
Primary artifact:
Secondary artifacts:
Non-goals:
Success conditions:
```

Before and after are the load bearing pair, and they are written as observable
conditions. "Fragmented opinions about readiness, held by people who never
compare them" to "one evidence backed maturity model with named gaps and an
owner per gap" is a value proposition. "Better readiness" is not.

**Gate:** the promise is falsifiable. Write down the observation that would show
the unit failed to deliver it. If you cannot, the promise is decoration and you
rewrite it rather than proceeding.

### 2. Operating specification

```
Identity:
Mission:
Scope:
Non-scope:
Trigger:
Preconditions:
Required inputs:
Recommended inputs:
Optional inputs:
Evidence hierarchy:
Workflow:
Decision points:
Human approval gates:
Stop conditions:
Primary artifacts:
Quality gates:
Upstream handoffs:
Downstream handoffs:
Security sensitivity:
Tests required:
```

Cut the modes from the canonical loop, taking only the segments this capability
needs:

```
TRIGGER -> INTAKE -> VALIDATE -> DISCOVER -> ANALYZE -> CHALLENGE -> DECIDE
        -> SYNTHESIZE -> REVIEW -> ARTIFACT -> QUALITY GATE -> HANDOFF
```

Declare the evidence states explicitly: observed, reported, inferred, assumed,
unknown, conflicting. Declare how confidence is expressed and what lowers it.
Declare the stop conditions: the situations in which the unit stops rather than
producing a weaker answer.

Non-scope is not a courtesy section. An OS that owns everything owns nothing,
and the boundary is what makes the suite composable. For each item in
non-scope, name the unit that does own it.

**Gate:** every mode has an entry condition, a produced artifact and a
completion test. A mode that cannot say when it is done is a conversation.

### 3. Artifact architecture

Define the primary artifact, the secondary artifacts, the schema that carries
each across a system boundary, the executive view (what a decision maker reads
in two minutes), the registers (assumptions, conflicts, unknowns, decisions),
and the handoff object the next unit consumes.

Traceability runs one way and is checked in both directions:

```
RECOMMENDATION -> FINDING -> EVIDENCE -> SOURCE
```

Assumptions live in their own register, never mixed into findings. Conflicting
evidence is preserved until resolved, never averaged.

**Gate:** the primary artifact has a named owner, a place to live and a
consumer. An artifact nobody consumes is a phase you can delete, and deleting
it is the correct move.

### 4. Package design

Run the tool selection tree over every piece of work the specification implies.
One row per piece of work.

| The work is | The component is |
|---|---|
| a deterministic calculation | a script or calculator, never a model |
| a strict interchange between systems | a schema |
| a reusable document | a template |
| adaptive questioning or synthesis | an LLM prompt |
| human judgement | a skill section plus an approval gate |
| repeated quality assurance | tests plus a rubric |
| movement across system boundaries | a handoff contract |
| a one-off explanation | nothing, do not overbuild |

Then declare the suite coordinates, because the package is not free-form: the
23 contract files in `verify.py: CONTRACT_FILES` are mandatory, and anything
beyond them must justify its existence in this table.

```
Slug:                  (ends in -os, is the directory name)
Number:                (contiguous, appended inside its group block)
Group:                 (one of the nine group keys)
Tagline:               (one sentence, ends in a period, no long dash)
Commands:              (non-empty, or the manifest check fails)
requires:              [slug]
consumes:              [namespace.thing.verb]
emits:                 [namespace.thing.verb]
consumes_from:         [slug]
emits_to:              [slug]
handoffs:              [{to: slug, artifact: name}]
requires_human_approval_for:
```

The dependency types are distinct and mixing them is the failure worth
catching. `requires`, `consumes_from` and `emits_to` hold slugs. `consumes` and
`emits` hold dotted event names. Putting a slug in `emits` is a reported error,
not a matter of style.

**Gate:** no LLM is doing deterministic work, no directory exists to satisfy a
diagram, and you name at least one thing you decided not to build. An architect
that adds every component considered has not architected anything.

## Challenges you must run against your own design

Before returning, attack the design once yourself. Answer each in one line.

1. Which component here would a competent practitioner call unnecessary?
2. Which part of this is an existing unit's job, and did I check?
3. Where am I using a model because it is available rather than because the
   work is adaptive?
4. What is the simplest version of this that still covers every case, and why
   did I not propose that?
5. Which of my scope items cannot be tested, and should therefore leave scope?
