---
name: os-builder-os
description: Build an operative system itself: intake, spec, research, build, red team, score, release. OS Builder {OS}, unit 00 of the AGENTIK {OS} suite (00 · RUNTIME). Use when the user wants to design, build, rebuild or audit an OS of the suite, or invokes /os-builder-os.
---

# OS Builder {OS}

Turn a capability request into a contract-complete, tested, scored, installable
OS, or into a defensible refusal to build one. Unit 00: the OS that produces
the others.

## When to use this

Reach for OS Builder when the object of the work is an OS itself:

- "I want an OS for X" (intake, then a build verdict, then a build)
- "This capability keeps coming back and I keep re-explaining it"
- "Turn this prompt or this legacy pack into a real unit"
- "Raise `<slug>` from scaffold to contract" (rebuild)
- "Grade `<slug>` against the contract without changing it" (audit)
- "Should this be an OS at all, or a checklist"
- "Why does `verify.py` fail on `<slug>` and what does it take to pass"

Do not use it for the subject matter of anything it builds. A question about
pricing goes to Pricing {OS}, not here, even while the Pricing unit is being
built.

Near neighbours, and the discriminator for each:

| If the ask is | The owner is | Because |
|---|---|---|
| "install / configure / run / update / evaluate an OS" | Agentik Runtime {OS} (`agentik-runtime`) | it owns the lifecycle on a machine; OS Builder stops at the package |
| "which systems do I need for my objective" | Agentik Runtime {OS} | composing a stack is a runtime job, not a build |
| "define this product before we code it" | Blueprint {OS} (`blueprint-os`) | it compiles a product definition pack; OS Builder compiles an operative system |
| "plan and execute the implementation steps" | Stepper {OS} (`stepper-os`) then Builder {OS} (`builder-os`) | they own a software build; OS Builder authors a capability package |
| "certify that what was built conforms" | Quality & Evaluation {OS} (`quality-evaluation-os`) | independence from the builder is the point of its verdict |
| "gather the evidence on this domain" | Research {OS} (`research-os`) | OS Builder consumes a source base, it is not a research engine |
| "score this AI output with a rubric" | Evaluation {OS} (`evaluation-os`) | it owns rubric and grader conventions; OS Builder applies them to a package |
| "design the agents inside this system" | Agent {OS} (`agent-os`) | agent design is its object; OS Builder decides only whether an agent is warranted at all |

The reliable test: if the deliverable is a directory under `OS/<slug>/`, it is
OS Builder. If the deliverable is anything that directory would produce, it is
the unit inside it.

## Capabilities

- Normalise a request of any shape into a complete OS request, filling what is
  derivable and marking the rest unknown rather than guessing it.
- Answer BUILD, SPLIT, REUSE or DO NOT BUILD, with the reason and the lighter
  artifact named when the answer is not BUILD.
- Read the suite registry before anything else, so an existing owner of the
  capability is found before a duplicate is created.
- Write a value proposition that is specific and falsifiable, with the before
  state, the after state, the primary artifact and the explicit non-goals.
- Build a source base where each source carries why it matters, the ideas
  actually used, its limitations and where it is used, and separate that
  cleanly from original synthesis.
- Design the human skill layer: mental models, principles, the questions an
  expert asks, the signals they read, the mistakes they avoid, a practice
  ladder and a proficiency rubric.
- Design the operating model: trigger, intake, validation, analysis, challenge,
  decision, synthesis, review, artifact, quality gate, handoff, with the
  evidence states, the stop conditions and the approval gates attached.
- Arbitrate deterministic code against model judgment on the simplicity ladder,
  and justify every rung above the lowest one that works.
- Materialise the file contract with `OS/_tools/scaffold.py`, author wave 1 and
  then wave 2, and grade with `OS/_tools/verify.py` at both tiers.
- Write a manifest whose every declared edge resolves to a real slug read off a
  neighbour's own manifest.
- Run the adversarial suite past the happy path and record every real result.
- Red team the package: overclaiming, fabricated ROI, unnecessary AI, boundary
  violations, unsupported conclusions, skipped approvals, sensitive data.
- Score 16 dimensions with the evidence behind each number, repair every
  mandatory dimension below 4, and re-score from the re-read files.
- Assemble the release record, the changelog entry and the proposed registry
  line, then stop and ask for the human approval that release requires.
- Audit an existing unit read-only, citing a file and a line for every finding.

## Procedure

1. **Read the registry first.** `OS/_registry.json` and the target group. Find
   out whether this capability already has an owner. This step costs a minute
   and prevents the most expensive mistake this OS can make.
2. **Intake.** Normalise into name, capability, primary operator, environment,
   problem, desired outcome, primary artifact, upstream, downstream,
   constraints, research depth and security sensitivity. Ask only the questions
   whose wrong answer would throw work away, and state your default for each.
3. **Rule on it.** BUILD, SPLIT, REUSE or DO NOT BUILD. Say why. If it is not
   BUILD, name the lighter artifact or the existing slug and stop here. Ending
   the session at this step is a successful run.
4. **Define.** Value proposition, scope, non-scope, primary artifact. The gate:
   you can explain the capability without naming a folder or a prompt.
5. **Research.** Build the source base. Label every load-bearing claim as
   source-backed or original synthesis. Preserve conflicts as conflicts and
   record gaps as UNKNOWN.
6. **Architect.** Write the OS spec before any file: identity, trigger, inputs,
   evidence hierarchy, workflow, decision points, approval gates, stop
   conditions, artifacts, quality gates, handoffs, and the package plan with a
   stated reason for every component. Challenge every rung of complexity above
   the lowest one that works.
7. **Get the spec agreed,** then reserve the slug. A slug and a number are a
   registry change, which is a human decision (`OS.md` section 9).
8. **Scaffold.** `python3 OS/_tools/scaffold.py build <slug>`. It is additive
   and idempotent: it creates what is missing and overwrites nothing.
9. **Author wave 1:** `OS.md` (all ten sections), `SKILL.md`,
   `manifest.json`, `COMMANDS/README.md`, `WORKFLOWS/README.md`. Then run
   `python3 OS/_tools/verify.py <slug>` and read the failures.
10. **Author wave 2:** the remaining 18 files, the surfaces that make the unit
    teachable, portable and reviewable. Then run
    `python3 OS/_tools/verify.py --full <slug>`.
11. **Test.** Happy path, missing data, conflict, weak evidence, out of scope,
    security, adversarial pressure, regression. Record the real result of each,
    including the ones that did not run.
12. **Red team.** Attack the package. For each attack: expected safe behaviour,
    actual behaviour, severity, repair. Leave no attack open without a repair.
13. **Score.** All 16 dimensions, each with its evidence. Never a bare number.
14. **Repair and re-score.** Every mandatory dimension below 4, and every red
    grader check. Re-read the files before re-scoring. The threshold: no
    mandatory dimension below 4, average at or above 4.3, with evidence,
    operating logic, artifact quality, security and testability each at 4 or
    better.
15. **Assemble the release record** and the proposed registry line, state the
    open risks and the named unknowns, and stop for human approval.
16. **Hand off** to Agentik Runtime {OS} and close. Say what was built, what
    was scored, what was approved, and what is still unknown. Do not install
    it, and do not run it.

## Handoffs

| Receives | What OS Builder sends | What that OS does with it |
|---|---|---|
| Agentik Runtime {OS} (`agentik-runtime`) | a contract-complete package plus a resolvable manifest and its permission boundary | installs, configures, runs, updates and evaluates it |
| Context & Memory {OS} (`context-memory-os`) | the approved OS spec, the build ledger and the release record | holds the canonical durable record |
| Quality & Evaluation {OS} (`quality-evaluation-os`) | the generated OS's evaluation suites, contracts and traceability | certifies conformance independently when the unit ships as a product |
| Security {OS} (`security-os`) | the sensitivity classification and the declared controls | threat models and issues or refuses a clearance |
| Release {OS} (`release-os`) | the release candidate and its gate evidence | decides whether it ships, and owns rollout and rollback |

What comes back in: a source base and contested claims from Research {OS}
(`research-os`), definition conventions from Blueprint {OS} (`blueprint-os`),
the code against judgment arbitration from AI Logic {OS} (`ai-logic-os`),
rubric conventions from Evaluation {OS} (`evaluation-os`), and the Runtime's
own doctor and eval reports, which are the only real proof that a package built
here actually works on a machine.
