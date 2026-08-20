---
name: growth-os
description: Loops, experiments and the channels that compound. Growth {OS}, unit 37 of the AGENTIK {OS} suite (04 · GROW). Use when the user asks about growth or invokes /growth-os.
---

# Growth {OS}

Find and run the loops that compound, and kill the ones that only look like
they do.

## When to use this

Use Growth {OS} when the question is structural: which mechanism in the
business feeds itself, where it leaks, what to test next, and whether a channel
has earned more investment. It is the right unit when the operator says "this
is not working" about a system rather than about an artifact.

Near neighbours it is confused with:

- **Content {OS}** owns editorial strategy, the calendar and publishing. If the
  question is what to post, it is Content. If the question is whether posting is
  a loop at all, it is Growth.
- **Pricing {OS}** owns the price. A pricing experiment is designed here and
  executed there, under Pricing's approval gate.
- **Sales {OS}** owns the pipeline. Growth reads conversion by stage; it does
  not work a deal.
- **KPI & Analytics {OS}** owns the metric definitions. Growth uses them and
  never redefines one locally.
- **Affiliate {OS}** owns partner selection. Growth measures the promotion as a
  channel; it does not choose the partner.

The discriminating question: **does the output return to the input?** If the
operator cannot answer it, that is the first job here.

## Capabilities

- Map the business's loops and separate them from funnels wearing the name.
- Compute cycle time, coefficient and cost per loop, from owned data.
- Build a backlog ranked partly on how cheaply a hypothesis can be killed.
- Design an experiment with a pre-registered threshold, sample, duration,
  stopping rule and guardrail metric.
- Read a result against the threshold that was fixed beforehand, unchanged.
- Score a channel on cost per retained cohort, not on signups.
- Produce a change proposal in the shape the owning OS expects.
- Write a kill record that survives the next person who likes the channel.

## Procedure

1. **Map before testing.** Pull content, pipeline, revenue and retention data.
   Trace each candidate loop's output back to its own input. Anything that does
   not close is renamed a funnel, out loud.
2. **Locate the weakest step** by its effect on cycle time and coefficient, not
   by which step is most annoying.
3. **Write hypotheses against that step.** Each names the step, the mechanism,
   the expected effect size, and the cheapest test that could falsify it.
4. **Rank the backlog** on expected information per unit of cost and time.
5. **Design the selected experiment**: target metric, guardrail metric, sample,
   duration, success threshold, stopping rule. Fix all of them now. Pull metric
   definitions from KPI & Analytics {OS} rather than restating them.
6. **Check the approval boundary.** Spend, price changes and anything touching
   live customers stop here for an explicit human decision.
7. **Hand the change to its owning OS.** Growth does not apply it. Record that
   the owner's own gate was passed.
8. **Run to the stopping rule.** Do not read the result early against a moving
   threshold; the stopping rule is the only early exit.
9. **Read the verdict** against the pre-registered threshold, and check the
   guardrail with equal seriousness. Underpowered means no verdict.
10. **Scale, kill, or return to the backlog**, and write the record either way.

## Handoffs

| Receiver | What it gets | What it expects |
|---|---|---|
| Content {OS} | a change proposal for an editorial surface | a specific change, its hypothesis and its guardrail; Content's publishing gate still applies |
| Pricing {OS} | a pricing experiment design | the threshold and the customer population; Pricing owns the price change and its approval |
| Offer {OS} | evidence that scope or guarantee affects conversion | the loss reasons behind the claim |
| Sales {OS} | conversion findings by stage | the stage, the sample and the definition used |
| Affiliate {OS} | channel performance of a promotion | the stated threshold and the stop condition it was measured against |
| KPI & Analytics {OS} | experiment verdicts | the pre-registered threshold, the sample, and both target and guardrail movements |
| Business Strategy {OS} | which loops compound and at what cost | cost per retained cohort, not signups |

Growth {OS} hands nothing directly to a customer. Any customer-facing text
involved in an experiment is written and approved inside Content {OS}.
