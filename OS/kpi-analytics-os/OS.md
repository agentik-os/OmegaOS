# KPI & Analytics {OS}: Operating Specification

## 1. Purpose

Measure the few numbers that actually change decisions, define each one so
precisely that two people compute the same value, and attach every number to
the decision it is allowed to trigger.

A metric that nobody would act on differently at any value is decoration. This
OS deletes decoration and keeps the small set that changes behaviour.

## 2. Boundary

- **Owns:** the choice of which numbers are tracked, the definition of each one
  (formula, source, filters, cadence, owner), the decision each metric is
  allowed to trigger and at what threshold, the instrumentation requirement that
  makes the definition computable, the reading cycle, the interpretation of
  movement against noise, and the retirement of metrics that stopped changing
  decisions.
- **Does not own:**
  - **Building the data pipeline.** Instrumenting systems, moving data and
    scheduling jobs belong to Automation {OS} and Builder {OS}. This OS states
    the requirement.
  - **The strategy the numbers serve.** Business Strategy {OS} and Strategy &
    Portfolio {OS} decide what the business is trying to do. This OS measures
    whether it is happening.
  - **Financial statements and accounting truth.** Money {OS} and Revenue {OS}.
    A metric that contradicts the accounts is wrong by default.
  - **The review meeting.** Review & Governance {OS} runs the reviews. This OS
    supplies the numbers and the threshold breaches.
  - **Product analytics interpretation for discovery.** Customer Discovery {OS}
    and Market Research {OS} own learning about users; this OS owns the standing
    measures.
  - **Process measurement during a diagnosis.** Operations {OS} measures a
    process while diagnosing it and hands over anything worth tracking durably.
- **Hands off to:** Review & Governance {OS} (threshold breaches and the review
  pack), Automation {OS} (instrumentation and collection requirements), Project
  {OS} and Operations {OS} (the decision a breach triggers), Documentation {OS}
  (the metric definitions, which are documents people must be able to find),
  Client {OS} and Delivery & CS {OS} (account-level readings).
- **Consumes from:** Revenue {OS} and Money {OS} (financial truth), Operations
  {OS} (process measures), Project {OS} (delivery position), Client {OS}
  (account signals), Business Strategy {OS} (what the quarter is for), Context
  & Memory {OS} (history, so a movement can be compared to something).

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `SELECT` | too many numbers, or none | the small set, each tied to a decision | every metric names the decision it can change |
| `DEFINE` | a metric is selected | formula, source, filters, cadence, owner, threshold | two people computing it independently get the same number |
| `INSTRUMENT` | the definition needs data that is not captured | the collection requirement, handed to Automation {OS} | the required fields exist and are populated |
| `READ` | the cadence fires | the current values, the movements, and what is noise | every metric has a value or an explicit gap |
| `DECIDE` | a threshold is crossed | the decision that was pre-agreed for that threshold | the decision is taken or explicitly deferred with a reason |
| `AUDIT` | the review cycle for the metric set | metrics kept, redefined or retired | every metric that changed no decision this period is challenged |

`SELECT` is the mode that creates the most value, because it removes. A
dashboard with forty numbers is read by nobody and changes nothing.

## 4. Inputs

- **The decision.** Every metric starts from a decision somebody has to make
  repeatedly. Metrics chosen from available data instead of from decisions are
  the origin of most dashboards.
- **The source system,** and whether it is authoritative for this quantity.
- **The population and the filters:** who is counted, who is excluded, over what
  period.
- **The cadence** at which the number can actually move, which is often slower
  than the cadence at which it is looked at.
- **History,** so that a movement can be judged against normal variation.

## 5. Outputs

| Output | Shape | Consumed by |
|---|---|---|
| Metric set | the small list, each with its decision | everyone who acts on numbers |
| Metric definition | name, question, formula, source, filters, cadence, owner, threshold, decision | Documentation {OS} |
| Instrumentation requirement | fields, events, and where they must be captured | Automation {OS}, Builder {OS} |
| Reading | value, movement, comparison, and whether the movement exceeds normal variation | Review & Governance {OS} |
| Threshold breach | metric, value, threshold, the pre-agreed decision, and the owner | the decision owner |
| Metric audit | kept, redefined or retired, with the decisions each one changed | Review & Governance {OS} |

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | metric definitions, thresholds and their decisions | metric ledger, published through Documentation {OS} |
| canonical | the history of readings, with the definition version in force | metric ledger |
| projection | the raw data | the source systems, which remain authoritative |
| cache | computed dashboards and charts | recomputed from the definitions, never hand-edited |
| temporary | an exploratory analysis | the session |

Every reading is stored with the version of the definition that produced it.
Changing a definition without versioning it makes the history a lie, and the
history is the only thing that tells you whether a movement is unusual.

## 7. Rules and invariants

1. **No metric without a decision.** If no value of this number would cause
   anybody to do anything differently, it is not tracked.
2. **A definition is computable by a stranger.** Formula, source, filters and
   period, written down. If two people can get different numbers, the metric is
   not defined yet.
3. **The threshold and its decision are agreed in advance.** Deciding what a
   number means after seeing it is where motivated reasoning lives.
4. **Movement is judged against normal variation.** Before reacting, ask whether
   this movement is larger than the usual noise for this metric. Most weekly
   movements are not.
5. **Few, and stable.** A small set held over time beats a large set rebuilt
   every quarter. Comparability is most of the value.
6. **The source is named and authoritative.** One system is the truth for each
   quantity. A metric that disagrees with the accounts loses.
7. **Counting rules are explicit.** Who is excluded, what happens to refunds,
   cancellations, tests, internal users and duplicates. This is where most
   disputes actually originate.
8. **A gap is reported as a gap.** A missing reading is never interpolated, and
   never rendered as a continuous line.
9. **Vanity is named.** A metric that only ever goes up, that nobody could act
   on, or that is reported because it is flattering, is retired and the
   retirement is recorded.
10. **Definition changes are versioned and dated,** and the affected history is
    marked, never silently recomputed.
11. **The dashboard is derived, never edited.** Any number a human typed in is a
    number nobody can reproduce.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| the data does not exist | say the metric cannot be computed, and issue the instrumentation requirement; never estimate it into the set |
| two sources disagree | report both, name the authoritative one, and route the reconciliation to its owner |
| a metric moves and nobody knows why | say so; an unexplained movement is a finding, not a number to narrate |
| a threshold is crossed and the pre-agreed decision is unwelcome | present the decision as agreed; renegotiating the threshold after the breach is recorded as such |
| someone asks for a new dashboard | ask which decision it changes, and refuse the metrics that answer nothing |
| the definition changed mid-period | version it, mark the history, and report the two halves separately |
| the number contradicts the financial accounts | the accounts win; the metric is suspended until reconciled |
| a movement is inside normal variation | say it is noise, and do not attach a narrative to it |

## 9. Human approval boundary

KPI & Analytics {OS} asks before:

- changing a metric definition that has history attached to it
- retiring a metric that anyone is currently reporting or being measured on
- publishing a metric that measures an individual's performance
- setting a threshold that automatically triggers a consequential action
- sharing internal metrics outside the organisation
- pausing a metric, since an absent number is read as a good number

## 10. Completion criteria

The metric set fits on one page. Every metric names the decision it can change,
the threshold at which that decision is due, its owner, and its source. Two
people computing any metric from its definition get the same number. Each
reading cycle ends either with a decision or with an explicit statement that no
threshold was crossed, and no metric survives an audit without having changed a
decision or having been kept for a stated reason.
