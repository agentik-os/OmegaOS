# Prompt: Intake

**Runs at:** phases 0 and 1 of `WORKFLOWS/FULL_BUILD.md`.
**Takes:** a capability request in any shape (one sentence, a paragraph, a
meeting note, an old package, a workflow somebody runs by hand).
**Returns:** the build record, the assumption register, the blocking questions,
and the viability verdict.
**Creates no files.** Intake that has already written a file has skipped the
decision about whether to write anything.

## Instruction

Normalise this request into a build record. Do not design, do not propose a
package, do not name components. Your only job is to find out what is actually
being asked for, and then to decide whether an OS is the right answer.

Read the request twice. The first read gets what was said. The second read gets
what was assumed by the person saying it, which is usually where the real
capability is hiding.

## Output shape

### 1. Build record

Fifteen fields, every one present. Blank where genuinely unknown, and marked
unknown with an owner, never filled with a plausible guess.

```
Name:
Capability:
Primary operator:
Target environment:
Business problem:
Desired outcome:
Primary artifact:
Upstream systems:
Downstream systems:
Shared systems:
Constraints:
Research depth:
Security sensitivity:
Required modes:
Packaging target:
```

Mark every field `[stated]` or `[inferred]`. An inferred field is a hypothesis
you are asking the requester to confirm by not objecting, and it must be
visible as such.

### 2. Blocking questions

At most three. Each carries your recommended default, so the requester can
answer "yes to all".

A question is blocking only when a wrong answer means throwing work away rather
than adjusting it. It is not blocking when a minute of reading the repository
would answer it. Never ask which group the unit belongs to, which files the
contract requires, or whether a slug is taken: read `OS/README.md` and
`OS/_tools/suite.py`.

If nothing is genuinely blocking, say so and list zero.

### 3. Assumptions

Numbered, specific, falsifiable, each with a reversal trigger: the observation
that would tell you the assumption was wrong.

"The operator runs this monthly, not daily, so state may live in a file rather
than a database" is an assumption. "The system should be maintainable" is not.

Cover whichever the capability actually touches: the shape and volume and trust
level of the data, what a malformed input looks like, what happens on timeout
or partial write, who calls this and what backwards compatibility it owes,
concurrency and idempotency, the runtime environment and what it may reach,
what you are deliberately not doing, and what will be left untested.

### 4. Security classification

One of: public, internal, confidential, sensitive, regulated. Then the domain
triggers, if any: money, legal rights, production systems, health, employment,
compliance, or personal data. Any trigger present means the capability may not
take the fast path, and you say so here rather than discovering it later.

### 5. Viability verdict

Run the tree. Answer each node, do not jump to the leaf.

```
Repeatable professional capability?
  no  -> NOT A CAPABILITY. Stop.
  yes -> recurring decisions, workflow, or artifacts?
    no  -> USE A LIGHTER ARTIFACT. Name which: prompt, checklist, template
           or skill. Stop.
    yes -> can it be bounded?
      no  -> SPLIT. Name the halves. Re-enter intake once per half. Stop.
      yes -> does reusable operating infrastructure add value?
        no  -> USE A LIGHTER ARTIFACT. Stop.
        yes -> BUILD.
```

Before returning `BUILD`, run one more check the tree does not: search the
existing suite for a unit that already covers most of this. If one exists,
return `ALREADY COVERED` and name the slug. Duplication is the most expensive
defect this OS can ship, because it stays invisible until two units disagree
and nobody knows which is canonical.

Four of the six verdicts stop the pipeline. Returning "a checklist is the right
size for this" is a successful run of this prompt, not a failure of it. An
intake that returns `BUILD` every time is an intake that is not deciding
anything.

### 6. Capability statement

One paragraph, for the `BUILD` verdict only. Explain the capability to a
competent stranger without mentioning folders, prompts, files or models. If the
explanation needs the package to make sense, the capability is not yet a
capability, and you return to the tree rather than proceeding.

## Refusals

- Do not accept a solution as a problem. "We need an OS that scores vendors" is
  a solution; the problem is underneath it, and it is usually different.
- Do not accept "better decisions", "more efficiency" or "alignment" as a
  desired outcome. Ask what would be observably different.
- Do not fill an unknown field to make the record look complete. An honest
  eleven-field record with four owned unknowns outranks a complete-looking
  fifteen.
