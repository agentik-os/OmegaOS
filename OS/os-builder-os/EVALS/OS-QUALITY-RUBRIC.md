# OS Quality Rubric

Sixteen dimensions, scored 0 to 5, every score carrying written evidence that
cites a file. This is the judged half of the release decision. The mechanical
half is [`RELEASE-GATE.md`](RELEASE-GATE.md) and the suite's own grader.

Scoring a package that has not yet passed `validate_os.py` at CORE tier is
scoring a draft. Grade the contract first, then judge the quality.

## The scale

| Score | Meaning |
|---|---|
| 5 | exceptional: better than the operator would have produced unaided |
| 4 | strong professional release quality |
| 3 | usable but incomplete: a competent operator can work with it and will have to fill gaps |
| 2 | weak: present but does not do its job |
| 1 | superficial: the heading exists, the content does not |
| 0 | absent |

There is no half point. A dimension sitting between two anchors takes the lower
one. Half points are how an average creeps over a threshold it did not earn.

## The threshold, and how the upstream wording was resolved

The source material stated the threshold three times, in three slightly
different ways: "no mandatory score below 4 and average >= 4.3"; "no mandatory
score below 4/5, average mandatory score >= 4.3/5, and evidence, operating
logic, artifact quality, security and testability must each be >= 4"; and a
manifest carrying `minimum_dimension: 4.0, minimum_average: 4.3`. Those three
readings differ on what "mandatory" covers.

**Resolved to the strict reading**, which is not weaker than any of the three:

1. All sixteen dimensions are scored. A dimension left unfilled makes the
   scorecard malformed; it is not a zero, because treating an unfilled field as
   a zero lets an incomplete review pass itself off as a harsh one.
2. Every dimension must be **4 or above**.
3. Five dimensions are **CRITICAL** and admit no waiver at all: evidence
   discipline, operating logic, artifact quality, security, testability. An OS
   weak in one of these is not weak, it is wrong.
4. The mean over all sixteen must be **4.3 or above**.
5. A waiver may lift rule 2 for **one non critical dimension**, and only with a
   named approver and a written reason recorded in the changelog. It can never
   lift rule 3 or rule 4.

The arithmetic is executed by [`../TOOLS/score_os.py`](../TOOLS/score_os.py) so
that it cannot be rounded in prose.

## The sixteen dimensions

Each entry gives the question, the anchors, and how the score is evidenced.
Where a mechanical check exists it is named: a mechanical check can only ever
**disprove** a high score, never award one. Passing `verify.py` does not make a
package good, it makes it eligible to be judged.

### 1. Value proposition

*Does this OS promise something specific, falsifiable and worth having?*

- **5** the promise names the before state, the after state and the artifact, and a skeptic could design a test that would show it failed.
- **4** specific and falsifiable, stated in the operator's language.
- **3** clear but generic: true of several units in the suite.
- **2** a category description standing in for a promise.
- **0** absent, or "helps you with X".

Evidenced by: `README.md` opening, `OS.md` section 1. Red flag: the promise
survives unchanged if you swap in a different capability's name.

### 2. Scope

*Is the boundary drawn tightly enough that another OS could sit next to it?*

- **5** owns, does not own, hands off to and consumes from are each specific, and the near neighbour it is most confused with is named and distinguished.
- **4** all four boundary statements present and specific.
- **3** owns is clear, the negative side is thin.
- **2** boundary stated in a way that would absorb adjacent requests.
- **0** no non scope.

Evidenced by: `OS.md` section 2, and the intake's `non_scope`. Cross checked
against the boundary map in memory: an overlap above roughly 70 percent with an
existing unit is a scope failure regardless of how well written it is.

### 3. Domain depth

*Does it know the field, or does it know the vocabulary of the field?*

- **5** carries the distinctions a practitioner would recognise and an outsider would miss, including at least one thing the obvious approach gets wrong.
- **4** source backed operating logic, sources meeting the sufficiency table for its risk class.
- **3** correct but shallow: nothing here a careful generalist could not produce.
- **2** vocabulary without mechanism.
- **0** no references, or references with no `WHERE USED`.

Evidenced by: `REFERENCES/`, and the capture records against
[`../REFERENCES/REFERENCE-POLICY.md`](../REFERENCES/REFERENCE-POLICY.md).

### 4. Human skill

*Can a person learn the capability from this, independently of the AI?*

- **5** mental models, principles, the questions an expert asks, the signals they read, the mistakes they have already made, a practice ladder and a proficiency rubric.
- **4** the skill layer teaches, with a proficiency ladder from unexposed to expert.
- **3** describes what to do without teaching why.
- **2** a checklist labelled as a skill.
- **0** no skill layer: the OS augments nothing, it substitutes.

Evidenced by: `SKILL.md`. The test: remove the model, hand the file to a
competent operator, and ask whether they get better at the work.

### 5. Operating logic (CRITICAL)

*Is there a real machine here, or a sequence of suggestions?*

- **5** every mode has an entry condition, a completion test and a stop condition; gates are placed where a wrong answer is expensive; the failure paths are as specified as the happy path.
- **4** the full operating chain is present with decision gates and stop conditions.
- **3** a workflow with no gates: it describes order, not control.
- **2** a checklist presented as a workflow.
- **0** prose.

Evidenced by: `OS.md` sections 3, 7, 8, 9, and `WORKFLOWS/`. Mechanical floor:
`verify.py` `SUBSTANCE` proves the ten sections exist. It cannot tell whether
section 3 contains modes or a placeholder table, which is precisely why this
dimension is judged and critical.

### 6. Evidence discipline (CRITICAL)

*Does the OS know the difference between what it saw, what it was told, what it
worked out and what it assumed?*

- **5** the six evidence states are used in anger, confidence is tracked separately from state, conflicts are preserved unresolved, and there is a worked path where the correct output is `UNKNOWN`.
- **4** states and confidence defined and applied to every material conclusion.
- **3** states defined and applied unevenly.
- **2** states listed and never used.
- **0** conclusions with no provenance, or fabricated precision.

Evidenced by: `OS.md` sections 4 and 8, the evidence register, and the abstention
suite in [`README.md`](README.md). One fabricated citation caps this dimension
at 0 and blocks the release outright.

### 7. Decision quality

*When the OS decides, is the decision defensible?*

- **5** every gate names what it is deciding, on what evidence, what happens on each branch, and who can overrule it.
- **4** decision gates are explicit, with criteria rather than judgment calls.
- **3** decisions happen but the criteria are implicit.
- **2** the OS recommends and never decides.
- **0** decisions with no stated basis.

Evidenced by: `OS.md` sections 3 and 9, `WORKFLOWS/`.

### 8. Artifact quality (CRITICAL)

*Is the thing it produces something a professional would put their name on?*

- **5** the primary artifact is complete, structured, has an executive view for a reader who will not read it all, and states its own limitations.
- **4** the primary artifact is defined, shaped and demonstrated end to end in `EXAMPLES/`.
- **3** the artifact is defined but never shown finished.
- **2** the artifact is a transcript of the conversation.
- **0** chat only: no artifact.

Evidenced by: `OS.md` section 5, `EXAMPLES/`. An OS whose output only exists
inside the session scores at most 2 here, which blocks release, because this is
a critical dimension.

### 9. Executive usability

*Can a busy reader get the decision in under a minute?*

- **5** the artifact opens with the answer, the confidence and what would change it, before any working.
- **4** an executive view exists and is genuinely short.
- **3** the artifact is well organised but front loads method.
- **2** the answer is somewhere in the middle.
- **0** the reader must reconstruct the conclusion.

Evidenced by: `EXAMPLES/`, the artifact interface spec.

### 10. Security (CRITICAL)

*Does the blast radius match the controls?*

- **5** every input and output classified, domain specific controls present and tested, the approval boundary non empty and enforced in a test, and one worked path where the correct output is refusal.
- **4** classification, minimum controls and an approval boundary, all specific to this capability.
- **3** controls stated generically, not derived from what this OS actually touches.
- **2** a security heading with no controls behind it.
- **0** absent, or a credential anywhere in the package.

Evidenced by: `OS.md` section 9, `MEMORY/policy.md`, the security suite, and
[`../REFERENCES/SECURITY.md`](../REFERENCES/SECURITY.md). A credential in a
package file is an automatic 0 and an automatic block.

### 11. Testability (CRITICAL)

*Would these tests catch the OS being wrong?*

- **5** the seven case families are covered, the adversarial cases actually attack, each case names its fail signature, and at least one test is a regression from a defect that really occurred.
- **4** coverage beyond the happy path: missing input, conflict, weak evidence, out of scope, security, adversarial.
- **3** happy path plus one or two others.
- **2** happy path only.
- **0** no evals, or evals that cannot fail.

Evidenced by: `EVALS/`, [`TEST-PLAN.md`](TEST-PLAN.md). The question that
settles it: name the change to the OS that would make a test go red. If nobody
can, the tests are decoration.

### 12. Traceability

*Can a reader get from a recommendation back to a source?*

- **5** the full chain, an assumption register with falsifiers, and open assumptions shipped visible.
- **4** recommendations link to findings link to evidence link to sources, and the assumption register exists.
- **3** sources present, the chain is partly reconstructable.
- **2** a bibliography.
- **0** conclusions with no provenance.

Evidenced by: the registers, against
[`../REFERENCES/TRACEABILITY.md`](../REFERENCES/TRACEABILITY.md).

### 13. Reusability

*Does it work for the next case, or only the one it was built on?*

- **5** parameterised throughout, with the one worked example clearly marked as an instance rather than the definition.
- **4** general, with the specifics of the originating case pushed into `EXAMPLES/`.
- **3** mostly general with a few leaked specifics.
- **2** the originating case is baked into the operating logic.
- **0** a one off dressed as a system.

Evidenced by: reading `OS.md` and `PROMPTS/` for names, figures and
organisations that belong to one situation.

### 14. Installability

*Can a competent operator who never spoke to the builder get it running?*

- **5** installs, configures and produces its first artifact with no question the package does not already answer.
- **4** `SETUP.md` names the minimum required inputs and `manifest.json` is complete and correct.
- **3** installable with one piece of tacit knowledge.
- **2** installable only by the builder.
- **0** does not satisfy the file contract.

Evidenced mechanically by: `validate_os.py --full`, `graph.py --strict`, and a
complete `manifest.json`. This is the most mechanically checkable dimension in
the rubric, and a package failing `STRUCTURE`, `MANIFEST` or `DEPS` cannot score
above 2 whatever the prose says.

### 15. Handoffs

*Can another OS consume this one without a conversation?*

- **5** every handoff names the artifact in the receiver's vocabulary, the graph joins, and the receiving unit could act on it without asking a question.
- **4** handoffs declared in `manifest.json` and described in `OS.md` section 2, all slugs resolving.
- **3** handoffs described in prose, absent from the manifest.
- **2** "integrates with" and a list of names.
- **0** no boundary contract.

Evidenced mechanically by: `graph.py --strict` reporting no orphan consume, and
`verify.py` `DEPS` resolving every slug. A handoff that reads well and does not
join is worth less than no handoff, because it looks connected.

### 16. Adapters

*Does it run where the operator actually works, and does it say what it cannot
do there?*

- **5** each of the four adapters names the specific capability it uses, the exact installation, and the honest degradation with its fallback.
- **4** all four present and target specific.
- **3** present but interchangeable: swap the product name and nothing else changes.
- **2** one adapter, the others stubs.
- **0** absent.

Evidenced by: `ADAPTERS/`. The test: does each file contain a sentence that
would be **false** if pasted into a different adapter? If not, it scores 3 at
best. Silent degradation is a defect; declared degradation is a pass.

## Scoring procedure

1. Confirm `validate_os.py <path> --full` passes. If it does not, fix the
   contract before judging quality.
2. Score each dimension against the anchors, writing the evidence sentence
   **before** choosing the number. Choosing the number first produces evidence
   that justifies rather than evidence that measures.
3. Fill [`scorecard.example.json`](scorecard.example.json)'s shape and run
   `score_os.py`.
4. On `BLOCKED`, repair the named dimensions and re-score. **Do not re-score
   without changing the package**: a second opinion on the same artifact is not
   a repair, and it is the most common way a scorecard gets talked upward.
5. Record the scorecard against the version. Scores are versioned, never
   overwritten. The trend across versions is the only honest answer to whether
   the suite is improving.

## Automatic rejection, before any scoring

The upstream quality standard names six conditions that stop a build regardless
of dimension scores. Each maps to a dimension that would already have failed,
but they are listed separately because they are recognisable at a glance and
save a full scoring pass:

| Condition | Dimension it fails |
|---|---|
| generic: true of any capability | 1 value proposition |
| unbounded: no non scope | 2 scope |
| chat only: no artifact | 8 artifact quality, CRITICAL |
| unsupported: claims with no evidence | 6 evidence discipline, CRITICAL |
| insecure: no controls for what it touches | 10 security, CRITICAL |
| untested: happy path only or no evals | 11 testability, CRITICAL |
| mostly placeholders | 14 installability, and `verify.py` `AUTHORED` |
