# Phase 03 — Blueprint: Your Company AI OS

> *Legible company first, then automatable, then agentic — in that order.*

---

## Purpose

With every discovery dossier in hand, this phase maps the complete picture of your
company's tools, data, and workflows — then designs the system that will serve your
C-suite. The output is a ten-file Company AI OS blueprint that specifies exactly what
will be built, in what order, at what projected cost and ROI. It must be reviewed and
approved by your executive sponsor before a single line of code is written.

---

## What happens

1. **Tool and integration mapping.** We catalogue every tool each executive uses,
   its role as a system of record or secondary tool, whether it exposes a usable API,
   and where data currently gets re-typed or copy-pasted by hand.

2. **Data and permission mapping.** We document every data source that would feed
   the system — format, location, sensitivity, PII exposure, GDPR / compliance
   posture, access controls required.

3. **Opportunity scoring.** Every automation opportunity surfaced in discovery is
   scored on ten criteria: business impact, time saved, frequency, pain intensity,
   data readiness, integration feasibility, risk level, change resistance, agent
   suitability, and dashboard fit. Each opportunity receives a score out of 100 and a
   verdict — build now, build later, park 90 days, data cleanup required, executive
   decision required, or refused (for sensitive HR, legal, or financial decisions no
   agent should make autonomously).

4. **System architecture design.** The approved opportunities are translated into the
   offer's signature topology: the single centralized client-owned server, which
   C-Level micro-SaaS dashboards get built (only for executives who actually exist in
   your org), the inter-dashboard API contract (which metric on one dashboard triggers
   which alert on another), the six Composio connectors mapped to your real systems
   of record, and the observability layer wired in from day one.

5. **30/60/90 roadmap and ROI projection.** Each phase of the build is sequenced
   with cost, team composition, and a projected ROI per workflow — calculated as
   hours × loaded cost × frequency with every number cited. No invented returns.

6. **Executive sponsor review and approval gate.** The architecture is presented to
   your sponsor. Questions are answered. The design is approved in writing before
   build begins. This gate is non-negotiable: it protects you from building the wrong
   thing at full cost.

---

## What you receive

The ten deliverables of the `company-ai-os/` output:

- **`00-Executive-Summary.md`** — one page for the CEO: top opportunities, projected
  impact, key decisions, and the 30/60/90 roadmap at a glance.
- **`01-Stakeholder-Interview-Plan.md`** — the interview record and consent log.
- **`02-Role-And-Workflow-Inventory.md`** — per-role: mission, tasks, tools,
  frictions, and ideal workflow.
- **`03-Tool-And-Integration-Map.md`** — tool inventory, system of record per data
  type, current automations, integration priorities.
- **`04-Data-And-Permission-Map.md`** — sources, sensitive data, PII/GDPR, access
  controls, vendor risk.
- **`05-Automation-Opportunity-Backlog.md`** — every opportunity scored and
  classified, the prioritized build table.
- **`06-Agentic-System-Blueprints.md`** — per agent: problem, workflow, tools,
  human-in-the-loop gates, memory, knowledge base, evaluation criteria, logs,
  permissions.
- **`07-Dashboard-Feature-Specs.md`** — per feature: user, problem, ideal state,
  input, action, output, UI, permissions, acceptance criteria.
- **`08-Implementation-Roadmap.md`** — phases 0–5, the 30/60/90 detail, cost, team,
  stack decisions.
- **`09-ROI-Governance-And-Risks.md`** — ROI per workflow, AI usage policy,
  human-in-the-loop rules, security, compliance, change management.

---

## What we need from you

- Access to tool documentation, API credentials (read-only scope), or a technical
  contact who can answer integration questions
- Your executive sponsor available for a one-hour architecture review
- A written approval (email or signed document) on the architecture before build
  starts — this is the gate that protects your investment
- Decisions on any open questions the blueprint surfaces (region, compliance posture,
  integration priority)

---

## Duration

Three to seven working days for the full blueprint. The architecture review and
approval add one to three days depending on sponsor availability.

---

## How you will know this phase is complete

The ten `company-ai-os/` files exist, the opportunity backlog is scored, every
projected ROI figure is traceable to a calculation (no invented numbers), and your
executive sponsor has reviewed and approved the architecture in writing. The build
cannot begin until that approval is on record.

If an opportunity in the backlog does not have a sourced evidence quote and a
calculated ROI, the blueprint for that opportunity is not complete.
