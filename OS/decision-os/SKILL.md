---
name: decision-os
description: Make hard calls well: framing, reversibility, evidence, the decision record. Decision {OS}, unit 08 of the AGENTIK {OS} suite (01 · PERSONAL). Use when the user asks about decision or invokes /decision-os.
---

# Decision {OS}

Takes one hard call, frames it in a single sentence, produces real options,
classifies each by reversibility, scores them against criteria sourced from
other units, and closes with a decision record that carries its own review
trigger.

## When to use this

Reach for this OS when the user says, in any wording:

- "I have been going back and forth on this for two weeks"
- "Do I take the offer or stay"
- "Should I shut this project down"
- "I know what I want to do and I do not trust why I want it"
- "I decided in March and I never checked whether it worked"
- "My gut says leave, the numbers say stay"
- "Write this down so I stop relitigating it"

Near neighbours and the discriminator for each:

| Confused with | Discriminator |
|---|---|
| Alignment {OS} (`alignment-os`) | Alignment says what the call costs in values terms and hands over criteria. Its `/decision` protocol is a lens, not a verdict. The choice, the record and the review are made here. |
| Goal & Life Strategy {OS} (`goal-life-strategy-os`) | That unit picks what you are aiming at across horizons. This unit answers the single question in front of you now, in service of that aim. |
| Intuitive {OS} (`intuitive-os`) | Intuitive logs a pre-verbal signal as a falsifiable prediction and scores its calibration. It never decides. Here the signal is one weighted input among several. |
| Mindset {OS} (`mindset-os`) | If the real blocker is "I am not the kind of person who does this", that is a belief and belongs to Mindset. Deciding around an untouched distorting belief produces a decision you will not execute. |
| Execution {OS} (`execution-os`) | Once the call is made, planning and doing the work belongs there. This unit stops at the record and the handoff. |
| Research {OS} (`research-os`) | Research gathers external evidence. This unit sets the threshold that evidence has to clear and decides when it has. |

## Capabilities

- Reduces a tangled situation to one decision question with a real deadline.
- Sources criteria from Alignment {OS} and the objective from Goal & Life
  Strategy {OS}, and flags any criterion invented mid-session as unsourced.
- Generates at least three options, including doing nothing and the option the
  user is avoiding naming.
- Classes each option as reversible, costly to reverse, or irreversible, and
  prices the undo.
- Sets an evidence threshold, then either meets it or defines the cheapest
  bounded experiment that would.
- Runs a pre-mortem per serious option and surfaces second-order effects.
- Records an intuition signal with its calibration weight, including when it was
  overruled.
- Writes a decision record that preserves what was believed at the time.
- Grades that record later against what happened, with a four-way verdict.

## Procedure

1. Ask what the decision is and when it must be made. Write the question as one
   sentence and get explicit agreement on that sentence.
2. Pull the objective from Goal & Life Strategy {OS}, the weighted values and
   control map from Alignment {OS}, and any prior record on the same call from
   Context & Memory {OS}. Name every source that is missing.
3. Separate what is genuinely under the user's control from what is influence
   and what is neither. Constraints that are not real constraints are removed
   here.
4. Generate at least three options. Add doing nothing. Ask directly for the one
   they have been avoiding saying out loud.
5. Class each option by reversibility and price the undo. This sets the evidence
   bar for the rest of the session.
6. Score options against the criteria and their weights. Run a pre-mortem for
   each serious option: assume it failed, say why.
7. Record the intuition signal from Intuitive {OS} with its calibration weight,
   or as uncalibrated with zero weight if there is no history.
8. Test the evidence against the threshold. If it is short and the deadline
   allows, define one bounded experiment with a date and stop there.
9. Decide, or state plainly that the threshold is unmet and the deadline forced
   it. Ask for approval before recording, and always before an irreversible
   option.
10. Write the record: choice, rationale, discarded options and why, evidence,
    signal and weight, reversibility class, review trigger and date.
11. Hand off the work to Execution {OS} and the record to Journal {OS}. Stop.

## Handoffs

| Receiving OS | Shape of what it gets |
|---|---|
| Execution {OS} (`execution-os`) | the decided course of action as work, with the rationale, the constraints and the review date attached |
| Journal {OS} (`journal-os`) | the decision record as a dated entry, so the reflective stream holds what was decided and when |
| Goal & Life Strategy {OS} (`goal-life-strategy-os`) | a decision that changes an allocation or retires a goal, with the objective it affected named |
| Review & Governance {OS} (`review-governance-os`) | a decision with organisational consequence, with its reversibility class and review trigger |
| Intuitive {OS} (`intuitive-os`) | reads the resolved outcome back out of the record to score its own calibration; this unit does not push it |
| Context & Memory {OS} (`context-memory-os`) | the canonical record and its appended review verdicts, written after explicit approval |
| A qualified human professional | any call carrying clinical, medical or real legal exposure, routed before the call is scored |
