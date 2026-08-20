# Loop map

Establish, from owned data, which mechanisms in the business actually feed
themselves, and rename the rest.

## Trigger

The operator asks how to grow, asks why growth stalled, or wants to allocate
budget across channels. Re-run whenever the offer, the price or the delivery
model changes, because all three change the loop's economics.

Always runs before any experiment is designed. An experiment on an unmapped
system optimises a step nobody has shown to matter.

## Steps

1. **The OS pulls the metric definitions** from KPI & Analytics {OS} and states
   which definitions it is using. It defines nothing locally.
2. **The OS pulls performance data**: content performance from Content {OS},
   conversion by stage from Sales {OS}, revenue and retention by cohort from
   Revenue {OS}, adoption and retention signals from Delivery & Customer
   Success {OS}. Missing sources are named, not estimated around.
3. **The OS enumerates candidate loops**, each as an ordered list of steps with
   the artifact that moves between them.
4. **The OS applies the closure test to each candidate**: does the output of the
   last step become an input to the first, in the same system, without a person
   deciding to restart it. A candidate that fails is renamed a funnel, and the
   step at which it fails to close is recorded.
5. **For each surviving loop the OS computes** cycle time, coefficient and cost
   per completed cycle, each traced to the source data.
6. **The OS scores each loop on retained cohorts**, not signups, using the
   Revenue and Delivery signals. A loop whose cohorts churn inside the window is
   marked as a leak with a budget.
7. **The OS ranks the loops** by compounding potential against cost, and marks
   the weakest step of the top-ranked loop as the entry point for `/backlog`.
8. **The operator reviews the map** and confirms or corrects the loop
   definitions. Corrections are recorded against the map, with the reason.

## Completion test

Every candidate in the map is classified as either a loop or a funnel, and no
candidate is unclassified. Each loop carries a cycle time, a coefficient and a
cost, each with a named source. Each funnel carries the step at which closure
fails. Each loop carries a retained-cohort score, or an explicit provisional
flag with the date the retention window closes.

A map containing any figure whose source is not one of the five consuming
relationships fails this test.

## Failure and abort

- **A data source is unavailable:** name it, map what can be mapped, and mark
  the affected loops incomplete. Never infer a coefficient from a proxy without
  saying so.
- **A local number disagrees with the KPI & Analytics definition:** stop, report
  both numbers and the canonical definition, and do not reconcile silently.
- **The retention window has not closed for a cohort:** mark the loop's score
  provisional with the date it becomes readable. Do not score on signups.
- **No candidate closes:** that is a valid and important result. Report that the
  business currently has no loops, list the funnels, and move to `/backlog`
  against the funnel step most likely to close one.
