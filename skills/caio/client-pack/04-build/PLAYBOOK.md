# Phase 04 — Build

> *Design the federation before you build it. Ship value in week one, not a demo in month three.*

---

## Purpose

This phase realizes the approved blueprint into a live, client-owned system. One
dedicated server. One function-specific micro-SaaS dashboard per C-Level in scope.
An inter-dashboard API contract so the system behaves as a federated Company AI OS
rather than a set of isolated screens. Six Composio connectors proven by live data
reads. Automated reports at the exact cadence your executives need. Built-in
monitoring and a baseline for ROI measurement. Each component ships when its
acceptance test passes against your real production data — never when a demo
looks good.

---

## What happens

1. **Architecture realization and design gate.** Before any server is provisioned,
   the approved blueprint is translated into a concrete build specification: which
   dashboards get built (only for C-Level roles that exist in your org), the exact
   inter-dashboard API contract (which metric triggers which alert, with threshold and
   human approval if the alert is sensitive), the six connectors mapped to your real
   systems of record, and the monitoring baseline plan. Your sponsor approves this
   spec in writing. This is the gate between design and build.

2. **Server provisioning.** One dedicated server is stood up in your required region
   with your data staying on it. The stack is readable, documented, and migratable —
   the export path (how your team takes the keys and leaves) is documented from day
   one.

3. **Per-C-Level micro-SaaS build.** Each dashboard is built for the real job that
   executive does — a CFO dashboard is a finance instrument, not a repainted template.
   Each ships through its own acceptance gate against live data before it is
   considered delivered.

4. **Inter-dashboard federation.** The API contract is implemented on the shared
   event bus so one dashboard's metric can trigger another's alert. At least one
   real cross-dashboard rule fires and is tested before the federation is marked
   complete.

5. **Composio integration wiring.** The six connectors that map to your real systems
   of record are configured and each proves a live data read — real authentication,
   a real record returned, rate-limit headroom confirmed. An enabled connector that
   never returns real data does not count.

6. **Automated reports.** Each report is specified — frequency, format, indicators,
   recipients — and shipped through its own acceptance test. Reports fall out
   automatically at the scheduled cadence.

7. **Monitoring and instrumentation.** The operating health layer is built at
   construction time, not bolted on later. Per dashboard: a North Star event (the
   value metric), a cost/usage event, and a value-delivered event. Thresholds are set.
   Alert owners are named. The baseline fires from go-live so ROI can be re-measured
   in Phase 6.

8. **Ship-gate ledger and go-live.** Every deliverable is logged — acceptance
   criteria (pulled from the blueprint's feature specs), run date, verdict, and
   evidence. Green = ships. The go-live announcement goes to your sponsor with a
   brief that shows real numbers from the live system, not a scripted demo.

---

## What you receive

Deliverables in this phase's `templates/` folder:

- **`Architecture-Realization-Spec.md`** — the design gate: generic blueprint
  translated into the centralized federated topology, approved by sponsor.
- **`Server-Provisioning-Runbook.md`** — the dedicated client-owned server: stack,
  region, data-residency posture, export path.
- **`MicroSaaS-Build-Checklist.md`** — per C-Level dashboard: build plan, agents
  delegated, ship-gate criteria.
- **`Inter-Dashboard-API-Contract.md`** — the exposes/consumes contract map: which
  dashboard publishes which metric, which dashboard subscribes and what it triggers.
- **`Integration-Wiring-Guide.md`** — Composio: the six connectors, live-read
  evidence for each, auth, rate-limit notes.
- **`Automated-Report-Spec.md`** — frequency, format, indicators, recipients, and
  the acceptance test for each automated report.
- **`Monitoring-Setup.md`** — the observability layer: North Star events, cost
  meters, value-delivered events, and the baseline start timestamp.
- **`Ship-Gate-Checklist.md`** — per deliverable: acceptance criteria, run date,
  verdict, and evidence (log or screenshot).
- **`Sponsor-Communication-Plan.md`** — milestone demo cadence, approval gate
  schedule, progress brief template, go-live announcement.

---

## What we need from you

- Read-only API credentials for your core tools (we document every permission
  requested and why)
- Your executive sponsor available for the realization gate approval and each
  milestone demo
- Decisions when the build surfaces a choice: data residency region, integration
  priority, human-approval thresholds for sensitive alerts
- A technical contact (internal or IT) who can provision or grant access if your
  systems have internal access controls

---

## Duration

Typically two to four weeks for a standard engagement covering three to five C-Level
dashboards. Single-dashboard builds can ship in under a week. Complex multi-dashboard
builds with many integrations may extend to six weeks.

---

## How you will know this phase is complete

Six tests must all be true:

1. The architecture realization spec was sponsor-approved before build started.
2. The dedicated server is standing, client-owned, with the export path documented.
3. At least one micro-SaaS shipped in week one with its acceptance test green on
   real production data.
4. At least one inter-dashboard alert actually fires (a real metric on one dashboard
   raises a real alert on another).
5. All wired Composio connectors return real data from your live systems.
6. The baseline monitoring events are firing and the start timestamp is recorded.

If any of these six are false, the build phase is not complete.
