# OS Builder {OS}: Generative UI Interface

Interface generated for the situation rather than fixed in advance, for the
moments in a build where the right control depends on data that does not exist
until the build produces it.

Most of OS Builder is deliberately **not** this. The intake is twelve known
fields, the rubric is sixteen known dimensions and the gate is eighteen known
items: all three are fixed forms, and generating them per session would make
them inconsistent across builds, which is the one property that makes a rubric
worth having. Generative UI is reached for only where the shape of the decision
genuinely varies.

## The rule for when to generate

Generate a control when **the set of options is produced by this build and
cannot be enumerated in advance**. Do not generate one when the options are
known and the only variation is which one applies. A generated version of a
fixed form is not adaptivity, it is drift.

| Situation | Fixed or generated | Why |
|---|---|---|
| the twelve intake fields | fixed | consistency across builds is the point |
| the sixteen rubric dimensions | fixed | a rubric that varies cannot produce a trend |
| the eighteen gate items | fixed | a gate that varies is not a gate |
| the boundary collision resolver | generated | the colliding units are discovered at intake |
| the mode designer | generated | the number and shape of modes is the build's own output |
| the dimension repair form | generated | which dimensions failed is not known in advance |
| the tool ladder picker | generated | the rungs are fixed, the candidate steps are not |
| the evidence conflict resolver | generated | the conflicting sources are discovered in research |
| the handoff wiring surface | generated | the neighbours depend on the capability |

## The six generated controls

### 1. Boundary collision resolver

**Appears when** the intake's capability overlaps an existing registry unit
above the duplication threshold.

Renders the overlapping unit beside the proposed one, capability claim against
capability claim, and offers exactly the four legal resolutions: extend the
existing unit, narrow the new one and declare a handoff, split the capability in
two, or refuse. Each option shows what it costs. "Build both anyway" is not
offered, because two units owning one decision is worse than either alternative
and an interface that offers it will see it chosen.

### 2. Mode designer

**Appears when** phase 5 opens with a workflow and no modes.

One row per proposed mode with four cells: name, entry condition, produces, done
when. The control refuses to accept a row whose `done when` is empty or whose
`done when` restates `produces`. That is the specific defect it exists to catch:
a completion test that says "the report is produced" tests nothing, and it is
invisible in prose review because it reads like a sentence.

### 3. Tool ladder picker

**Appears when** a step is proposed for automation or for a model call.

Renders the ladder as eight rungs, from `DO NOTHING` up to `MULTI AGENT`, with
the candidate step's own facts filled in beside each rung: what it would cost,
what it would fail like, what it would need. The default selection is always the
**lowest** rung that could work, and choosing a higher one requires a written
reason that is kept in the spec decisions record with its rejected alternatives.

The control's job is to make climbing the ladder feel like what it is, a cost,
rather than what it feels like unaided, ambition.

### 4. Evidence conflict resolver

**Appears when** two captured sources disagree on a load bearing point.

Renders both capture records side by side with their trust classes and their
`LIMITATIONS` fields, then asks which side the operating logic follows and why.

It offers three outcomes: follow A, follow B, or hold as unresolved and mark the
dependent conclusion `CONFLICTING`. **There is no fourth control, and in
particular there is no numeric field between the two values.** Averaging is the
default temptation here and the interface removes the affordance rather than
warning against it.

### 5. Dimension repair form

**Appears when** `score_os.py` returns `BLOCKED`.

One card per failing dimension, CRITICAL first, each showing: the current score,
the evidence sentence that produced it, the anchor text for the next score up,
and a field for the specific change proposed. The card cannot be closed by
re-scoring; it closes when files change and the re-score follows the change.

This is the surface that enforces "do not re-score an unchanged package". A form
whose only affordance is a number field will get a number.

### 6. Handoff wiring surface

**Appears when** phase 7 declares dependencies.

Renders the candidate at the centre and the named neighbours around it, each
edge typed: `requires`, `consumes`, `emits`, `consumes_from`, `emits_to` or a
named `handoff` artifact. Slugs are resolved live against `OS/_registry.json`,
so an unknown neighbour cannot be drawn. An `emits` edge that no unit consumes
renders faded and informational; a `consumes` edge that nothing emits renders as
an error, because that is an orphan and it means the boundary does not join.

The edge label for a handoff is written in the **receiver's** vocabulary, and
the control shows the receiving unit's own `OS.md` section 4 beside the field
while it is being written, so the producer can see what the receiver actually
looks for.

## Generation rules

1. **The data decides the control, never the model's sense of what would be
   nice.** Every generated control above is triggered by a specific machine
   detectable condition, listed with it.
2. **Never generate a control for a decision that has one legal answer.** Offer
   the answer and its reason instead. A choice that should not be a choice
   invites the wrong one to be made.
3. **Remove the affordance for the failure mode**, do not warn about it. No
   average field on the conflict resolver, no re-score button on the repair
   form, no "build both" on the collision resolver. A warning is a suggestion; a
   missing control is a constraint.
4. **Every generated control writes to the same record a typed answer would.**
   The spec, the evidence register, the scorecard. A control whose output lives
   only in the interface has produced nothing.
5. **Defaults are the safe option**, always: the lowest rung, the narrower
   boundary, the unresolved conflict, the gate held. An operator who accepts
   every default gets a conservative, correct build.

## Degradation

Every generated control degrades to the same question asked in prose, with the
same constraint enforced in words and then checked at the gate. The conflict
resolver becomes "which source does the logic follow, and why: A, B, or
unresolved", with averaging refused if attempted. The repair form becomes a
per dimension list of what a 4 requires. The ladder picker becomes the ladder
printed with the lowest workable rung named as the recommendation.

The controls are faster and harder to get wrong. They are never the only thing
standing between the build and a defect: each failure mode they remove the
affordance for is also a scored rubric dimension or a gate item, so a surface
that cannot render them loses convenience, not safety.
