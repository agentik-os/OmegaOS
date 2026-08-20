---
name: ai-logic-os
description: When to use deterministic code and when to use model judgment. AI Logic {OS}, unit 64 of the AGENTIK {OS} suite (08 · AI & SYSTEMS). Use when the user asks about ai logic or invokes /ai-logic-os.
---

# AI Logic {OS}

Decide, for one named job, whether it should be done by deterministic code or by
model judgment, and prove the decision with numbers. Default bias: no.

## When to use this

Use it when:

- Someone proposes an automation, an agent or a model call and you want to know
  whether it should exist before anyone builds it.
- A step is being done by a model and you suspect a rule would do it better,
  cheaper and more predictably.
- A model output with a real consequence has no way of being checked.
- An irreversible action is about to lose its human gate.
- An existing agentic system (a pipeline, an agent, a skill, a coding tool, a
  whole OS) feels expensive or unreliable and nobody can say where.
- A number is being claimed for a process nobody has measured.

Do not use it when you already know what to build and only need it built. This
OS argues; it does not implement.

**Near neighbours, and why this is not them.** Automation {OS} governs an
automation that is being built and run; AI Logic decides whether it deserves to
be built. Agent {OS} briefs and supervises one agent; AI Logic decides whether
the job needs an agent at all. Evaluation {OS} measures how good a model output
is; AI Logic decides whether a model should be producing that output.
Orchestration {OS} composes systems into a mission; AI Logic has no position in
any pipeline and runs only when invoked.

## Capabilities

- Arbitrate one step into exactly one bin: codify, augment, keep human, delete.
- Map a process that exists only in people's heads into numbered steps with an
  owner and a duration each.
- Establish a baseline (volume, time, error rate, cost) or specify the
  measurement device that would produce one.
- Triage a whole mapped process, announcing the deletions first.
- Compute annual gain against build plus maintenance, with the arithmetic
  visible, and say no when it does not clear.
- Name the falsifier a consequential model output requires, or refuse the step.
- Challenge an existing agentic system against five questions, every finding
  cited.
- Specify the first move only, build ready, with a done test and a rollback.
- Produce a never empty list of what is not recommended, and why.

## Procedure

1. **Scope.** Restate the step, process or system under arbitration in your own
   words. If two readers would scope it differently, stop and narrow it.
2. **Read what already governs it.** Existing rules, prior verdicts in Context &
   Memory {OS}, and the system's own logs. Most gaps are already covered.
3. **Get the baseline.** Volume, time, error rate, cost. If they do not exist,
   switch to producing a measurement device and stop there.
4. **Map the real process**, not the described one. Numbered steps, owner and
   duration each, exceptions included. The happy path is not the process.
5. **Triage every step into one bin.**
   - *Codify:* the input is structurable and the decision is a rule.
   - *Augment:* the input is unstructured or the decision needs judgment, and
     the output can be checked quickly.
   - *Keep human:* irreversible, high consequence, or requiring accountability.
   - *Delete:* it compensates for a defect elsewhere, or nobody reads the output.
     This is the most profitable bin and it is announced first.
6. **For every augment step, name the falsifier.** A deterministic check, a
   schema, a citable source, or a human who can reject it in seconds. No
   falsifier means the step moves to keep human or to codify.
7. **For every irreversible action, confirm the human gate.** Statistics may
   replace a gate only when they exist and are shown.
8. **Do the arithmetic.** Annual gain against build plus maintenance. Show the
   inputs. Count nothing that is not already in production.
9. **Rank the moves**, costliest gap first, and specify only the first one.
10. **Write what you do not recommend, and why.** Never leave this empty.

For `CHALLENGE` mode, replace steps 4 to 7 with the five questions, in order:
(1) where does a model do the job of a conditional? (2) where does a
consequential output go unverified? (3) where does an irreversible action lack a
human gate? (4) where is the feedback loop missing? (5) what primitive is absent
and is being re-derived by hand every time? Every finding cites a file and line,
a rule, or a log.

## Handoffs

| Receives your output | What it expects |
|---|---|
| Automation {OS} | a scored candidate with its arithmetic, its exceptions and its named owner, only after Operations {OS} has simplified the process |
| Agent {OS} | the verdict that a step genuinely needs judgment, plus the falsifier its output must carry |
| Evaluation {OS} | the falsifier specification, which becomes a rubric criterion |
| Operations {OS} | the process map when the honest answer is simplify before automating |
| Tool & Integration {OS} | the statement that a step needs an external system, never the contract itself |
| Context & Memory {OS} | the verdict and its baseline, staged as canonical records |

This OS never hands off a build. It hands off a decision, and the decision is
frequently no.
