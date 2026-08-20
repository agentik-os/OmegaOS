---
name: validation-os
description: Kill or confirm an idea with the cheapest sufficient test. Validation {OS}, unit 18 of the AGENTIK {OS} suite (02 · DISCOVER & DECIDE). Use when the user asks about validation or invokes /validation-os.
---

# Validation {OS}

Settle one claim at a time, with a threshold agreed before the data exists.

## When to use this

Reach for Validation when:

- You are about to commit weeks of build time to something nobody has tested.
- A plan, deck or blueprint contains a sentence like "users will pay for this"
  and nothing behind it.
- Two people disagree about a claim and neither can say what would change their
  mind.
- Someone said "validated" and you want to know what was actually measured.
- A market research pack came back GO and you want the three riskiest
  assumptions in it settled before funding.
- You have limited test budget and several candidate tests competing for it.

Near neighbours, and the line between them:

| Confused with | Difference |
|---|---|
| Research {OS} | Research answers a question with sources. Validation settles a claim with a pre-registered threshold. Research can end in "it depends"; Validation cannot. |
| Market Research {OS} | Market Research compiles the market and customer evidence body and issues a market decision. Validation takes single claims out of that body and tests the ones desk work could not settle. |
| Customer Discovery {OS} | Discovery talks to people to learn what is true about them, open ended. Validation runs an instrument to decide, with the stopping rule written first. Discovery can change its questions mid round; Validation cannot change its threshold. |
| Decision {OS} | Decision handles choices under irreducible uncertainty and values. Validation reduces the uncertainty first, where reduction is affordable. |
| Quality & Evaluation {OS} | That unit tests whether a built thing works. Validation tests whether it is worth building. |

## Capabilities

- Extract the implicit claims from a plan, deck, concept or blueprint and
  rewrite each one so it can be false.
- Rank claims by cost of being wrong times probability of being wrong, so the
  test budget goes to the claim that can actually hurt you.
- Design the cheapest instrument that can still produce a kill: smoke test,
  pre-sale, concierge run, fake door, price ladder, sample of one workflow done
  manually, letter of intent, paid pilot.
- Compute the sample size a threshold requires, and say plainly when the
  affordable sample cannot support the claim as stated.
- Pre-register threshold, stopping rule and kill criteria, and capture the
  owner's signature on them.
- Run the test, log every deviation, and stop when the rule fires.
- Issue a verdict: CONFIRMED, KILLED, INCONCLUSIVE or INVALID, always named
  against the signed threshold.
- Audit a claim someone else declared validated, and report exactly what was
  measured versus what is being claimed.
- Maintain a test queue ordered by expected information gain per unit of cost.

## Procedure

1. **Recover.** Pull the concept, plan or evidence pack and any prior verdicts
   on the same claims. Never re-test a settled claim without saying why.
2. **Frame.** List every claim the plan depends on. Rewrite each as a statement
   that could be false, with a subject, a magnitude and a window. "People want
   this" becomes "at least 8 of 40 contacted operations managers will book a
   30 minute call within 5 business days."
3. **Rank.** Score each claim by cost of being wrong and by current confidence.
   Test the expensive uncertain ones. Leave the cheap certain ones alone and say
   you left them.
4. **Design.** For the top claim, choose the cheapest instrument that can still
   return a kill. State sample, threshold, stopping rule, cost, calendar time,
   and what specifically dies if the claim dies.
5. **Sign.** Get the owner to accept the threshold before any data exists. If
   the owner will not commit to a threshold, stop here and record why. This is a
   legal stop, not a failure.
6. **Approve.** Anything touching a real person, a public surface, or money
   goes to the human approval boundary before it runs.
7. **Run.** Execute the spec. Log what was actually done, to whom, and every
   deviation as it happens rather than at the end.
8. **Verdict.** Compare the result to the signed threshold. Issue exactly one
   of CONFIRMED, KILLED, INCONCLUSIVE, INVALID. Do not soften a kill and do not
   round an inconclusive up.
9. **Propagate.** State what the verdict changes: which parts of the plan die,
   which survive, which claim is now the riskiest one and what its next test
   costs.
10. **Record.** Write the claim, spec, run log and verdict to canonical state so
    the next session cannot silently re-litigate it.

## Handoffs

| To | What it receives | What it does with it |
|---|---|---|
| Strategy & Portfolio {OS} | `validation.verdict.issued` | funds, pauses or kills the bet the claim supported |
| Business Model {OS} | verdicts on revenue, willingness to pay and cost claims | updates the model and its viability assessment |
| Blueprint {OS} | the set of CONFIRMED claims a definition may rest on | writes them in as evidence, not assumption |
| Market Research {OS} | verdicts on claims its desk work left open | closes or reopens its decision |
| Context & Memory {OS} | every canonical record | makes verdicts durable across sessions and OS units |

Received from: Brainstorm {OS} (`brainstorm.concept.selected`), Customer
Discovery {OS} (`discovery.insight.confirmed`), Market Research {OS}
(`market.validation.completed`), Research {OS} (`research.evidence.compiled`).
