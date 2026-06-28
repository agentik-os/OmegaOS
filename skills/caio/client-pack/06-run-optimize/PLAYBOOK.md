# Phase 06 — Run, Measure and Expand

> *A delivered system is not a finished system. Measured, it either compounds or it leaks.*

---

## Purpose

Once your team owns the system and the Autonomy-Readiness Gate has passed, Phase 5
begins: we run the live Company AI OS, measure actual ROI from real telemetry, monitor
system health with named alert thresholds, run a weekly and monthly optimization loop,
and operate a deliberately light one-hour-per-week strategic quota. When the system
is healthy and saturating its current scope, we bring a structured expansion proposal
for the next department.

---

## What happens

1. **NSM and baseline confirmation.** We define the system's single North Star Metric
   — one value-received measure that answers "is the system delivering real benefit
   this week?" For a time-saving objective, that is net hours redeployed to higher-
   value work per week, not page views or agent runs. We confirm the pre-go-live
   baseline captured in the build phase is real and complete.

2. **Cohort ROI re-measurement.** Actual savings are measured by go-live wave, not
   as a single average. For each workflow cohort we compute actual hours saved × loaded
   cost × frequency from event logs and receipts, lay the result beside the blueprint's
   projection, and issue one of three verdicts: Proven (actual ≥ 80% of projected),
   Partial (50–80%), or Falsified (below 50%). Falsified projections are named and
   routed back into the loop — they are findings, not embarrassments. Every figure
   cites its source.

3. **Monitoring and health spec.** The operating health dashboard covers five
   dimensions — liveness, cost, usage/adoption, quality, and value — each with an
   alert threshold and a named owner. An alert with no owner is decoration. When a
   scheduled agent misses a run or a workflow's weekly active users drop by more than
   30%, the alert fires before you notice the problem yourself.

4. **Weekly optimization loop (15 minutes).** Read the NSM, last cohort's savings-
   retention, cost versus budget, top adoption mover, and the single most important
   open alert. Triage only — no new projects in this pass.

5. **Monthly re-score (the compounding step).** The blueprint's opportunity backlog
   is re-scored against actual data, not the original estimates. The next-highest-
   value improvement is written as a falsifiable hypothesis: "because of this
   telemetry observation, I believe this change will move the NSM from X to Y within
   this window." Retention fixes are always prioritized above new builds while any
   cohort's savings are still decaying.

6. **1h/week strategic quota.** One hour per week for strategic questions, technical
   arbitrations, and new integration scoping. Never operational firefighting. Any
   work that exceeds the hour — a new workflow, a department expansion — is a scoped
   mini-engagement with its own statement of work, not silently absorbed into the
   retainer.

7. **Quarterly Business Review.** Every quarter the actual ROI is presented to your
   C-Level: NSM trend, cohort savings by wave, cost, adoption, decisions improved,
   and a clear recommendation on whether to renew as-is, optimize an existing wave,
   or expand to the next department. The QBR is the proof that drives renewal — not
   a relationship or a slide.

8. **Expansion proposal.** When wave-1 cohorts are delivering durable savings and the
   system is saturating its current scope, we bring a structured next-department
   proposal. The expansion loop re-enters the blueprint phase with the advantage that
   the centralized server and federation are already in place.

---

## What you receive

Deliverables in this phase's `templates/` folder:

- **`ROI-Measurement-Model.md`** — baseline to actual, by cohort, versus the
  blueprint's projection; every number sourced; Proven / Partial / Falsified verdicts.
- **`Monitoring-Health-Spec.md`** — the operating dashboard (liveness, cost, usage,
  quality, value) with thresholds, owners, and runbook lines per alert.
- **`Optimization-Loop-Cadence.md`** — the weekly one-screen ritual, the monthly
  re-score process, the live re-scored backlog, and each month's improvement
  hypothesis in falsifiable format.
- **`Weekly-Quota-Agenda.md`** — the one-hour strategic-quota agenda template and
  the explicit overage boundary (overage = new mini-SOW, not absorbed hours).
- **`Quarterly-Business-Review.md`** — the C-Level QBR: NSM trend, cohort ROI
  actual versus projected, cost, adoption, decisions improved, and the next-wave ask.
- **`Expansion-And-Referral-Play.md`** — renewal mechanics, next-department land-
  and-expand proposal, net-revenue-retention framing, internal reference path.

---

## What we need from you

- Live telemetry access and your executive sponsor at the quarterly review
- Timesheets or comparable records for the workflows the system handles (so actual
  hours saved can be calculated from receipts, not estimates)
- Honest feedback when the system is or is not delivering as projected
- Decisions on expansion scope when the monthly loop produces an "Expand" verdict

---

## Duration

Ongoing. The weekly quota is one hour. The monthly re-score is typically two to four
hours of combined CAIO and client time. The quarterly review is a 60-minute C-Level
session. Expansion proposals are scoped and contracted separately.

---

## How you will know this phase is compounding

Six signals tell you the run phase is working:

1. Every headline number in the Quarterly Business Review traces to a telemetry
   event or a receipt — zero invented figures.
2. The architect's projected ROI has been proven or honestly falsified — never
   silently smoothed.
3. Wave-1 cohorts are still delivering their savings at month three (plateau, not
   decay).
4. The optimization loop has shipped at least one measured improvement re-scored
   against actual data.
5. An alert threshold caught at least one issue before you noticed a problem.
6. The QBR drove a renewal or an expansion decision.

If fewer than four of these are true at the 90-day mark, the system is drifting —
we return to re-instrumentation and re-measurement before anything else.
