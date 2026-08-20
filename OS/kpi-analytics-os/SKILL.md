---
name: kpi-analytics-os
description: Measure the few numbers that actually change decisions. KPI & Analytics {OS}, unit 48 of the AGENTIK {OS} suite (05 · OPERATE). Use when the user asks about kpi & analytics or invokes /kpi-analytics-os.
---

# KPI & Analytics {OS}

Track the few numbers that change decisions, define each so precisely that two
people compute the same value, and attach every number to the decision it is
allowed to trigger.

## When to use this

Use it when there is a dashboard nobody reads, when two people quote different
numbers for the same thing, when a report is produced monthly and nothing ever
changes because of it, and before building any new tracking.

Typical openings: what should we be measuring, our numbers disagree, we have a
dashboard but I do not know what to do with it, is this going up or is that just
noise, can we track this.

Near neighbours it is confused with:

| If the real need is | The right OS is |
|---|---|
| building the pipeline or the tracking | Automation {OS}, Builder {OS} |
| the accounts and the financial truth | Money {OS}, Revenue {OS} |
| what the business should be trying to do | Business Strategy {OS} |
| running the review that reads the numbers | Review & Governance {OS} |
| learning something new about users | Customer Discovery {OS}, Market Research {OS} |
| measuring one process during a diagnosis | Operations {OS} |

## Capabilities

- Cut a metric list down to the few that change decisions, and say what the
  removed ones were costing in attention.
- Write a definition a stranger can compute: formula, source, filters, period,
  cadence, owner.
- Attach a threshold and the pre-agreed decision to each metric.
- State the instrumentation requirement when the data does not exist yet, rather
  than estimating.
- Read a cycle: values, movements, and whether a movement exceeds normal
  variation.
- Refuse to narrate noise, and say plainly when a movement is unexplained.
- Version definitions, and mark the history a change affects.
- Audit a metric set: which metrics changed a decision, and which are decoration.
- Name vanity metrics and retire them on the record.

## Procedure

1. Start from decisions, never from available data. List the decisions somebody
   makes repeatedly.
2. For each decision, ask what number would change it, and at what value.
3. Keep the metrics that survive that test. Everything else is a candidate for
   deletion.
4. Define each survivor: formula, authoritative source, filters, period,
   cadence, owner.
5. Write the counting rules explicitly, including refunds, cancellations, tests,
   internal users and duplicates.
6. Set the threshold and the decision it triggers, agreed before the number is
   seen.
7. Where the data does not exist, issue the instrumentation requirement to
   Automation {OS} and mark the metric as not yet computable.
8. On the cadence, read the values. Compare each movement to normal variation
   before attaching any explanation.
9. On a breach, present the pre-agreed decision to its owner.
10. On the audit cycle, challenge every metric that changed no decision.

## Handoffs

| Send to | What | What they expect |
|---|---|---|
| Review & Governance {OS} | the review pack and threshold breaches | values, movements, and the decision due |
| Automation {OS} | instrumentation requirements | the fields and events needed, and where |
| Operations {OS} | process measures that crossed a threshold | the number and the decision it triggers |
| Project {OS} | delivery metrics that affect a plan | the reading and its consequence for dates |
| Client {OS}, Delivery & CS {OS} | account-level readings | the metric, the account, the risk |
| Documentation {OS} | the metric definitions | the document, its owner and its review date |
