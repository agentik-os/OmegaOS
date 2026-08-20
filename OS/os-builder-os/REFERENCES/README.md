# OS Builder {OS}: References

The standards this OS builds against. Every other unit of the suite carries
references that are *domain* knowledge: frameworks, checklists, facts about
pricing or nutrition or capital. OS Builder is a meta OS, so its references are
different in kind. They are the **construction standards for an OS itself**: what
a package must contain, what a source must prove before it may be cited, how a
recommendation stays attached to its evidence, how a version number is allowed
to move, and what security posture a generated OS inherits by default.

These are normative. A generated OS that violates one of them does not ship, and
the violation is a named gate item in [`../EVALS/RELEASE-GATE.md`](../EVALS/RELEASE-GATE.md).

## The standards

| Reference | Governs | Enforced at |
|---|---|---|
| [`PACKAGE-STANDARD.md`](PACKAGE-STANDARD.md) | what files a generated OS must contain and what each one owes | `TOOLS/validate_os.py`, `OS/_tools/verify.py` |
| [`REFERENCE-POLICY.md`](REFERENCE-POLICY.md) | what may be cited, how it is captured, how far it is trusted | phase 3 research gate, rubric dimension 3 |
| [`TRACEABILITY.md`](TRACEABILITY.md) | the chain from recommendation back to source, and the assumption register | phase 5 gate, rubric dimensions 6 and 12 |
| [`VERSIONING.md`](VERSIONING.md) | when a change is MAJOR, MINOR or PATCH, and what a version bump obliges | phase 14 release, `CHANGELOG.md` |
| [`SECURITY.md`](SECURITY.md) | sensitivity classification and the minimum controls every OS inherits | phase 10 red team, rubric dimension 10 |
| [`HANDOFF-UPSTREAM.md`](HANDOFF-UPSTREAM.md) | what OS Builder accepts as an intake, and how it is normalised | phase 0 intake |
| [`HANDOFF-DOWNSTREAM.md`](HANDOFF-DOWNSTREAM.md) | what a finished OS must expose so another system can consume it | phase 14 release |

The machine-readable shapes that go with these standards live beside the tool
that checks them, in [`../TOOLS/schemas/`](../TOOLS/schemas/). A schema is not
knowledge, it is a contract, and it travels with its validator so the two cannot
drift apart.

## Trust classification

Every source cited by a generated OS carries exactly one class. The class is not
a compliment, it is a statement about what kind of wrongness the source is
exposed to.

| Class | What it is | What it can carry | What it cannot carry |
|---|---|---|---|
| `foundational` | the originating work of a field or method | the definition of a concept | current numbers |
| `official` | a vendor, standards body or regulator speaking about itself | the authoritative spec, limits, prices | independent judgment of that vendor |
| `regulatory` | statute, rule, published guidance | what is required or forbidden | how it is enforced in practice |
| `academic` | peer reviewed research | a measured effect with its conditions | generalisation past those conditions |
| `technical` | documentation, source code, an API reference | mechanism and interface | intent or roadmap |
| `practical` | practitioner writing, field reports | how the work is actually done | claims of typicality |
| `book` | a long form treatment | a mental model, a vocabulary | recency |
| `case_study` | one documented instance | existence proof, a failure mode | a base rate |

A single `case_study` never supports a general claim. A `practical` source never
supports a number. An `official` source never supports a comparison against a
competitor. These are not style preferences, they are what the rubric checks
under evidence discipline, and a build that breaks them scores 2.

## The rule that makes references worth having

**References must support actual system logic, never decorate the package.**

Every reference in a generated OS carries a `WHERE USED` field naming the file
and the specific decision it supports. A reference with an empty `WHERE USED` is
deleted before release, not left in as evidence of diligence. Decorative
bibliographies are the single most common way a hollow OS looks researched, and
the release gate treats an unused reference as a defect rather than a bonus.

## Reading order for a first build

1. `PACKAGE-STANDARD.md`, so you know the shape you are aiming at.
2. `HANDOFF-UPSTREAM.md`, so the intake is normalised before anything is created.
3. `REFERENCE-POLICY.md` and `TRACEABILITY.md`, before phase 3 research begins.
4. `SECURITY.md`, before any capability touching money, health, employment,
   legal rights, production systems or regulated data is scoped.
5. `VERSIONING.md` and `HANDOFF-DOWNSTREAM.md`, at release.
