# Traceability Standard

Every material conclusion a generated OS produces stays attached to the thing
that justified it. This is what separates an operating system from a confident
paragraph, and it is scored by rubric dimensions 6 (evidence discipline) and 12
(traceability).

## The chain

```
RECOMMENDATION  ->  FINDING  ->  EVIDENCE  ->  SOURCE
```

Read right to left it is provenance. Read left to right it is accountability.

| Link | What it is | Who may produce it |
|---|---|---|
| `SOURCE` | a captured reference, eight fields, per the reference policy | research phase only |
| `EVIDENCE` | one observation extracted from one source, with its state | research phase only |
| `FINDING` | what the evidence means for this specific case | analysis phase |
| `RECOMMENDATION` | what to do about it | decision phase |

A recommendation with no finding behind it is an opinion. A finding with no
evidence behind it is a guess. Both are permitted **only** when labelled as
such, and neither may pass a decision gate that requires evidence.

## Evidence states

Every evidence item carries exactly one state. The states are ordered by how
much weight they can bear, and the machine-readable shape is
[`../TOOLS/schemas/evidence.schema.json`](../TOOLS/schemas/evidence.schema.json).

| State | Means | May support a decision gate |
|---|---|---|
| `VERIFIED` | observed directly, or confirmed by two independent sources | yes |
| `SUPPORTED` | one credible source states it, within that source's class limits | yes |
| `INFERRED` | derived by reasoning from verified or supported items | yes, if the inference is shown |
| `ASSUMED` | taken as true to proceed, not established | no, and it enters the register |
| `CONFLICTING` | credible sources disagree and the conflict is unresolved | no, resolve or escalate |
| `UNKNOWN` | required and not found | no, and absence is reported |

`UNKNOWN` is a real output. An OS that cannot find what it needs says so and
names what would resolve it. Silence about a gap is the failure; the gap itself
is not.

## Confidence

Separate from state, and never a substitute for it. `HIGH`, `MEDIUM`, `LOW`.

Confidence describes how sure the OS is that its **interpretation** is right.
State describes how well the **underlying fact** is established. A `VERIFIED`
fact can carry a `LOW` confidence interpretation, and saying so is more useful
than collapsing both into one number.

Never fabricate false precision. "Roughly a third" is a better answer than
"31.4 percent" when the underlying evidence is a `practical` source and a
rounding. A decimal point implies a measurement that did not happen.

## The assumption register

Assumptions do not live inside the reasoning where they disappear. They live in
their own register, and the register ships with the OS.

Each entry:

```
ID            A-001
ASSUMPTION    stated so it can be checked, not so it sounds reasonable
WHY           what would otherwise have blocked
IMPACT        what changes if this is wrong
FALSIFIER     the specific observation that would kill it
OWNER         who can resolve it
STATUS        open | confirmed | refuted | superseded
```

An assumption with no `FALSIFIER` is not an assumption, it is a belief, and it
is rewritten until it has one. "Inputs are under ten thousand rows and fit in
memory" is an assumption. "The system should be maintainable" is not.

The register is never emptied at release. Open assumptions ship visible, because
the operator inheriting the OS needs to know which floorboards were never
tested.

## Preserve conflict, never average it

When two credible sources disagree:

1. Both stay in the package with their capture records intact.
2. The evidence item is marked `CONFLICTING`.
3. The OS states which side its logic follows, in one sentence, with the reason.
4. The other side stays visible, so a future reader can re-decide when new
   evidence arrives.

**Never average away disagreement.** Two sources saying 20 percent and 60
percent do not make 40 percent. Forty is a number no source supports, no reader
can defend, and no future evidence can correct, because the disagreement that
produced it has been destroyed. Averaging is not neutrality, it is the
manufacture of a false fact.

The same rule governs a disagreement between the OS and its user. The user's
claim is recorded as reported, the contrary evidence stays, and the conflict is
surfaced rather than resolved by deference.

## What ships

A released OS carries, at minimum:

- the reference set, each with `WHERE USED` filled
- the evidence register, every item with a state and a source
- the assumption register, open items included
- for each material recommendation, the finding it rests on

An OS that produces recommendations and ships no registers fails gate item
"traceability" and scores at most 2 on dimension 12, regardless of how good the
recommendations are. A right answer nobody can audit is a right answer nobody
can trust the next time.
