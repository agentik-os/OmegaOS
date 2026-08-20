# Workflow: Value driver review

Produces the value driver table and the readiness gap against the operator's
stated standard, with every unverified input named.

## Trigger

Any of:

- The review cadence set at configuration has elapsed (quarterly by default).
- The owner-dependence audit has just completed.
- KPI & Analytics {OS} or Revenue {OS} re-measured a metric that sits under a
  driver.
- The operator is preparing for a conversation with a buyer, an investor, a bank
  or an adviser.
- A single customer's share of revenue has moved materially.

## Steps

1. **Read the standard.** Retrieve the readiness standard from Context & Memory
   {OS}. If none is on file, ask for it, offering the common ones and how they
   differ. Do not proceed on an assumed standard.
2. **Pull the verified facts.** Read `kpi.metric.verified` from KPI & Analytics
   {OS} and the concentration and retention facts from Revenue {OS}. Record each
   value with its source and measurement date. Read `ownership.entity.registered`
   from Ownership {OS} and `ipasset.registered` from IP & Asset {OS}.
3. **Measure each driver.** Work the table in order: customer concentration,
   revenue retention and recurrence, margin quality, transferability of
   relationships, documented process coverage, key person risk, contract
   assignability, data and IP position. Each gets a value, a source and a date.
4. **Label every driver.** Verified where it traces to an upstream metric with a
   date. Unverified where it rests on the operator's account. A confident
   operator statement is still unverified, and is labelled so.
5. **Name the missing measurement.** For each unverified driver, state the
   specific metric that would verify it and which OS owns that metric. This is
   what makes the label actionable rather than a disclaimer.
6. **Check the ages.** Any driver whose underlying measurement predates the
   cadence is flagged stale, with its age attached. Ask the operator whether to
   re-measure or carry forward. Carrying forward is explicit or it does not
   happen.
7. **Reconcile disagreements.** Where KPI & Analytics {OS} and Revenue {OS} give
   different figures for the same fact, report both with sources and dates,
   refuse to pick, and route the reconciliation to the OS that owns the metric.
   Mark the driver blocked until it is resolved.
8. **Compute the gap.** Driver by driver against the stated standard. Never a
   single headline number on its own.
9. **Compute it again without the unverified inputs.** Present both, side by
   side, with the list of excluded drivers. A gap that changes materially between
   the two is itself the finding.
10. **Refresh the advantage register.** Run the copy test on each claimed
    advantage against the current driver values. Strike what no longer survives
    and record why.
11. **Emit and route.** Write the table to Context & Memory {OS}, emit
    `strategy.value_driver.measured` and, if a gap was recorded,
    `strategy.readiness.flagged`. On the operator's approval, hand the drivers
    and the gap to Exit & Liquidity {OS} and the remediation to Execution {OS}.

## Completion test

All of the following hold:

- Every driver in the table has a value, a source and a measurement date.
- Every driver carries a verified or unverified label, and every unverified one
  names the metric that would verify it and the OS that owns it.
- Every stale driver is either re-measured or explicitly carried forward with its
  age recorded.
- Any figure disagreement between upstream OS units is reported unresolved rather
  than silently averaged or picked.
- The readiness gap is presented per driver, and alongside the version computed
  without the unverified inputs, with those inputs listed.
- No valuation figure left this workflow without its internal working range label
  and its method exposed.
