---
name: review-governance-os
description: Turn actions, incidents, metrics and decisions into honest learning, controlled change, explicit policy and continuously improved personal and professional systems. Omega Core function that closes learning loops and governs consequential change across all other OSs (approval authority referenced by Revenue OS for boundary/schema/quality-gate changes). Contains 13 specialist agents, 20 skills, 7 protocols and 7 schemas. Use for incident review, postmortems, policy changes, retrospectives, or governance approval of a consequential change. Trigger words: review, governance, incident, postmortem, policy, retrospective, approval; FR: revue, gouvernance, incident, retour d'experience, politique, retrospective, approbation.
---

# Review & Governance {OS}

Runtime-installed pack (2026-08-11), staged for the OmegaOS repo-level R-SKILLPUB integration by a concurrent session. This SKILL.md is a pointer into the shipped pack; it does not restate or invent the pack's operating contract.

## Load before operating

- [README.md](README.md) for purpose, operating loop, commands and main handoffs.
- [system/SYSTEM_PROMPT.md](system/SYSTEM_PROMPT.md) for the full operating contract.
- [system/PRINCIPLES.md](system/PRINCIPLES.md) and [system/BOUNDARIES.md](system/BOUNDARIES.md) for scope and limits.
- [system/ROUTER.md](system/ROUTER.md) for command/intent routing.
- [MANIFEST.json](MANIFEST.json) for the full inventory (agents, skills, protocols, schemas).
- [OMEGA_INTEGRATION.md](OMEGA_INTEGRATION.md) for registration ID, event types and cross-OS handoffs.
- `agents/*.md` for specialist agent definitions, `skills/*.md` for reusable skill procedures, `protocols/*.md` for multi-step operating protocols, `schemas/*.json` for the data model.

## Commands

| Command | Mode | Purpose |
| --- | --- | --- |
| `/review` | weekly | Open review |
| `/daily-review` | daily | Run daily reflection |
| `/weekly-review` | weekly | Run weekly operating review |
| `/monthly-review` | monthly | Run monthly metrics review |
| `/quarterly-review` | quarterly | Run strategic governance |
| `/postmortem` | postmortem | Analyze an incident or failure |
| `/policy` | policy | Create or audit a policy |
| `/change-request` | change | Submit consequential change |
| `/risk-register` | monthly | Review risks |
| `/ai-governance` | ai-risk | Apply AI risk governance |

## Boundary

This pack is a runtime skill install only. It does not modify the OmegaOS repository, its install.sh, os_products.rs or OS-SUITE.md; that repo-level integration is a separate, coordinated follow-up (see handoff note).

## When to use this

Use it on the review cadence, the moment something fails materially, and
whenever a domain OS proposes a change to its own boundary, policy or controls.

Typical openings: we keep making the same mistake, that incident needs a
postmortem, who is allowed to decide this, can we change this rule, did the
change we made last month actually work.

The rule that routes work here: **a domain OS may not approve its own boundary
or policy change.** Execution {OS} cannot widen its own scope, Operations {OS}
cannot retire a control, Client {OS} cannot create a new exception class, and
KPI & Analytics {OS} cannot retire a metric that other people depend on. Each
proposes; this OS decides.

Near neighbours it is confused with:

| If the real need is | The right OS is |
|---|---|
| the position of the work | Project {OS} |
| the numbers themselves | KPI & Analytics {OS} |
| running the room where a decision is taken | Meeting {OS} |
| the daily and weekly personal loop | Execution {OS} |
| gating a software release | Quality, Evaluation & Release {OS} |
| writing the procedure that a policy implies | Process & SOP {OS} |

## Capabilities

- Run the four cadences at four deliberately different lengths: daily in
  minutes, weekly short, monthly against thresholds, quarterly on policy,
  decision rights and risk.
- Produce a blameless postmortem that names sequences and conditions rather than
  character, and ends with one owned change.
- Write policy with a scope, an owner, its exceptions and a review date.
- Authorise a consequential change with conditions, a reversal path and a
  verification test.
- Maintain a risk register in which every risk has a trigger and a response
  rather than an adjective.
- Assess an AI system that shapes a consequential decision, and name a real
  human oversight point.
- Verify a change after the fact and decide standardise, adjust or revert.
- Keep an append-only audit trail of who decided what, when, on what evidence.

## Procedure

1. Collect the evidence the domain OSes recorded. Memory at the review is an
   input, never the record.
2. Compare what happened against what was intended, and name the gap.
3. Explain the gap in conditions and sequences, not in character.
4. Decide, with an owner and a date on every decision.
5. For a change proposal, check the proposer is not the approver, then take the
   authorisation decision with its conditions.
6. Require a reversal path and a verification test before approving anything
   consequential.
7. Record the decision in the append-only audit trail.
8. Hand the approved change back to the owning OS.
9. On the verification date, run the test and decide standardise, adjust or
   revert.
10. Send what was learned to Context & Memory {OS}, and what was standardised to
    Process & SOP {OS} and Documentation {OS}.

## Handoffs

| Send to | What | What they expect |
|---|---|---|
| the proposing OS | the approved change, its conditions and its verification test | what may now be done, and what will be checked |
| Process & SOP {OS} | anything standardised after verification | the steps and the quality bar |
| Documentation {OS} | policies, postmortems and decision records | the document, its owner and its review date |
| Context & Memory {OS} | what was learned | confirmed, inspectable, so it is not relearned |
| Operations {OS} | control gaps and process causes found in a postmortem | evidence and the process defect |
| Team & Delegation {OS} | approved authority changes | the new level and its conditions |
