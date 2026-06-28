# Phase 05 — Enablement and Transfer

> *Your teams are not spectators of the system — they are its operational guardians.*

---

## Purpose

A working system that nobody uses is worse than no system — you paid for it twice.
This phase ensures adoption is real (measured by actual usage weeks later, not
attendance at a demo) and that transfer is proven (your team can extend the system
unaided). It covers two sub-phases: Phase 3 — Adoption, where every audience is
onboarded and completes their own first validated task through the system; and
Phase 4 — Transfer, where three specific extension motions are taught and proven
under real conditions.

---

## What happens

**Phase 3 — Adoption**

1. **Readiness check.** We verify the system's live acceptance gate is green before
   any training begins. Training on a broken system fills a leaky bucket. If the
   system is not working, we route back to build.

2. **Internal announcement.** A simple message goes to your team: we removed the
   tedious parts of the job, not the job. The frame is set before the first session:
   this system is built around you, not dropped on top of you.

3. **Audience-specific onboarding.** Four audiences each get a session pitched at
   their real level:
   - **Executives / sponsor** — the strategic view, the dashboards, the approval
     queue (what they need to review and when).
   - **Managers / department heads** — their team's workflows, reading the dashboard
     for their department, the local reinforcement role.
   - **Operators / end-users** — hands-on with their own real task through the
     system, guided first then unaided. The aha-moment is the goal.
   - **Internal technical owner** — the runbook, code and configuration pointers,
     the extension paths, the evolution process.

4. **First validated use cases.** Each operator completes their own real task through
   the system unaided, in real conditions. This is logged with evidence. A session
   where someone watched a demo does not count.

5. **Adoption tracking.** Usage is tracked by cohort — who is using which feature,
   how often, whether the retention curve is holding. If it is sagging, we re-onboard
   before declaring adoption complete.

**Phase 4 — Transfer**

6. **Internal documentation pack.** The system is fully documented for its owners:
   a system map, a per-dashboard explanation, per-agent runbooks, code and
   configuration pointers (commented, not just a link), and the evolution process
   (how to propose, test, and ship a change safely).

7. **Extension Playbook.** Three motions, taught step by step calibrated to your
   team's actual technical level (config-only, can-edit-prompts, or can-write-code):
   - Add an agent
   - Connect a new tool
   - Adjust a report

8. **Ownership handover.** Every component has a named owner. Zero CAIO-only
   credentials remain — all are rotated to your team's vault before sign-off.
   The weekly guardian routine (read the health dashboard, note one improvement) is
   run by your team at least twice without us present.

9. **Autonomy-Readiness Gate.** A named member of your team performs all three
   extension motions unaided, in real conditions, while we observe with our hands
   off the keyboard. Evidence is captured (PR, commit, or recording) for each motion.
   If the person had to ask us how during a motion, the motion does not pass — and
   the fix is usually the documentation, not the person.

---

## What you receive

Deliverables in this phase's `templates/` folder:

- **`Onboarding-Session-Plan.md`** — per audience: session format, agenda, the
  demo-to-adoption arc tailored to AI literacy level.
- **`Internal-Documentation-Pack.md`** — system map, per-dashboard how-it-works,
  per-agent runbook, code/config pointers, evolution process.
- **`End-User-Training-Curriculum.md`** — the operator curriculum: knowledge topics,
  guided-run plan, unaided-run criteria.
- **`Extension-Playbook.md`** — add-agent, connect-tool, adjust-report: step-by-step
  paths at each technical level (config / prompt / code).
- **`Ownership-Handover-Checklist.md`** — named owners, credential rotation log,
  escalation path, guardian routine adopted.
- **`Autonomy-Readiness-Gate.md`** — the objective gate: adoption metrics (NSM,
  cohort retention, validated use cases, skeptic status) and the three transfer
  motions with their evidence.
- **`Adoption-Tracker.md`** — active users per feature by cohort, week by week;
  handed to Phase 6 as the usage baseline.

---

## What we need from you

- Named owners per system component before we begin the handover
- Your team's genuine participation in the unaided extension motions (not a courtesy
  run with us coaching at every step)
- The weekly guardian routine adopted and run twice without us present before sign-off
- Any skeptics named and a re-onboard session scheduled for each one who did not
  convert to an active user

---

## Duration

Three to eight weeks for a full engagement covering all audiences and the Autonomy-
Readiness Gate. Champion-only enablement (one or two internal champions trained to
guardian level) can complete in approximately one week.

---

## How you will know this phase is complete

Five conditions must all be true:

1. Operators from each trained cohort are still using the system for real work at
   week four — the retention curve is not collapsing.
2. The Validated-Use-Cases log holds at least five real, evidenced runs.
3. All three extension motions in the Autonomy-Readiness Gate were completed unaided
   by a named member of your team, with evidence captured.
4. Zero CAIO-only credentials remain in any system.
5. The weekly guardian routine has been run at least twice by your team without us
   present.

If any of these five are not true, the phase is not complete.
