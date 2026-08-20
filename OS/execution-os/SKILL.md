---
name: execution-os
description: Time-bound personal commitments and proof of output. Execution {OS}, unit 40 of the AGENTIK {OS} suite (05 · OPERATE). Use when the user asks about execution or invokes /execution-os.
---

# Execution {OS}

Turn ambitions and obligations into a few time-bound commitments, protect the
hours they need, and close each one with evidence.

## When to use this

Use it when the user says any of: I have too much on, I do not know what to do
first, I keep starting and not finishing, what did I actually ship this week,
I promised this and it is late, I need to protect time for this.

Use it every working day for the `BOOT` and `HALT` pair. That pair is the whole
system in its smallest form: open the day with one must-win, close it with a
proof and tomorrow's first physical action.

Near neighbours it is confused with:

| If the real need is | The right OS is |
|---|---|
| a multi-week deliverable with milestones and dependencies | Project {OS} |
| specifying or building software | Blueprint, Stepper, Builder {OS} |
| a behaviour repeated for months | Habits {OS} |
| motivation, identity, self-talk | Mindset {OS} |
| work that should be someone else's | Team & Delegation {OS} |
| the honest weekly and monthly retrospective across all systems | Review & Governance {OS} |

## Capabilities

- Open a day against a stated capacity and usable minutes, with one must-win.
- Turn a vague intention into a commitment with an outcome, an acceptance test,
  a deadline and one physical next action.
- Refuse a day plan that exceeds its own capacity budget, and name the overflow.
- Protect a focus block of 25, 50 or 90 minutes on exactly one commitment.
- Close a commitment only on evidence plus the acceptance test it satisfies.
- Classify a failure as blocked, deferred, cancelled or delegated, each with a
  physical next action.
- Maintain a promise ledger with a notice-by date and the consequence of
  lateness, and prompt the renegotiation before the deadline.
- Close the day with a classification, a proof and tomorrow's first action.
- Run the weekly reset and the monthly system audit, changing the system rather
  than the task list.

## Procedure

1. Read the operator profile. Local profile first, shipped default second,
   current-turn facts above both.
2. `BOOT`: ask capacity, usable minutes and the must-win. Nothing else yet.
3. Pull open commitments and incoming obligations. Classify each: commitment,
   project, delegation, reference, dropped.
4. Fit the day to the budget. If committed minutes exceed usable minutes, ask
   what leaves the day.
5. For each committed item, confirm the one physical next action exists.
6. `FOCUS` on one commitment at a time. Never two.
7. `PROVE` on completion: capture the evidence and the acceptance test. No
   evidence means the commitment stays open.
8. `RECOVER` anything that failed, with a classification and a next action.
9. `HALT`: classification, energy, focus, friction, proof, and tomorrow's first
   physical action. The day does not close without that last field.
10. Weekly, run `RESET`. Monthly, run `AUDIT` and send the system change to
    Review & Governance {OS}.

The deterministic half of this runs in the `omega-execution` CLI, which owns
the state file. The coaching, classification and refusal behaviour runs here.

## Handoffs

| Send to | What | What they expect |
|---|---|---|
| Project {OS} | a commitment that is really a multi-week deliverable | outcome, constraint, deadline pressure |
| Team & Delegation {OS} | a commitment that should not be yours | outcome, definition of done, do-not-touch |
| Client {OS} | a promise made to a client | who, what, by when, consequence |
| Review & Governance {OS} | weekly reset and monthly audit output | evidence, the honest truth, the proposed system change |
| Documentation {OS} | proof artifacts worth keeping | the artifact, its topic, and who will need it |
| Context & Memory {OS} | durable profile facts only | confirmed, inspectable, removable |
