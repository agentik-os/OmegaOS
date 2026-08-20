# System evolution

Change the brand system on purpose, with a version number and a migration list,
so the result is one system rather than half of two.

## Trigger

The claim changed and the expression no longer fits it, a drift report shows the
same rule broken everywhere, the audience register moved, an accessibility
constraint fails on a live token, or the operator proposes a rebrand.

## Steps

1. **Brand {OS}** states the reason for the change in one sentence and
   classifies it: correction (the system was wrong), fit (the claim moved), or
   preference (somebody is bored). A preference change is reported as such.
2. **Brand {OS}** reads the current positioning statement. If the claim moved,
   the change is a fit change and Positioning {OS} is cited as its source; if
   the claim did not move, the change may not widen it.
3. **Brand {OS}** produces the proposed diff: which identity elements, voice
   rules or tokens change, and which stay. Anything not in the diff is
   explicitly unchanged.
4. **Brand {OS}** runs the proposed system against a sample of live artifacts
   and produces the impact: what passes today and would fail tomorrow.
5. **Brand {OS}** produces the migration list: every surface carrying the old
   system, ranked by how many people see it, each with an owning unit.
6. **Human** approves the change, the version number and the migration list.
   This approval is required for any identity core or name change without
   exception.
7. **Brand {OS}** publishes the new version with its change history, and marks
   the previous version superseded rather than deleting it.
8. **Brand {OS}** runs `/brand-handoff` to Design {OS}, Content {OS},
   Storyteller {OS}, Sales {OS}, Offer {OS} and Affiliate {OS}, each with the
   part of the diff that touches them.
9. **Owning units** correct their surfaces and confirm, or request a waiver
   with a reason. Brand {OS} records each confirmation or waiver against the
   version.
10. **Brand {OS}** closes the version only when every surface on the migration
    list is confirmed corrected or explicitly waived, and reports the open
    surfaces until then.

## Completion test

The new version exists with a change history and a diff, every surface on the
migration list carries either a correction confirmation from its owning unit or
a waiver with a named human and a reason, and an audit of a fresh sample of live
surfaces returns no artifact mixing the old and new systems. An open surface
with neither state, or a sampled artifact carrying both systems, means the
evolution is incomplete regardless of how good the new system looks.

## Failure and abort

- The change is classified preference at step 1 and the system is under two
  years old: report that recognition compounds and boredom arrives inside the
  company years before recognition arrives outside it, and require an explicit
  human decision to proceed anyway.
- The change would widen the claim at step 2: refuse. Route the question to
  Positioning {OS}, because an expression change that alters what is claimed is
  a positioning decision wearing a design brief.
- Human approval withheld at step 6: the current version stays live, the
  proposal is archived with its impact analysis, and no surface is touched. A
  partially applied rebrand is worse than the system it replaced.
- The migration list cannot be produced because no surface inventory exists:
  proceed only for new surfaces, keep the old system live everywhere else, and
  escalate the missing inventory. A migration with an unknown denominator never
  closes.
- A downstream unit neither corrects nor waives within the agreed window: the
  version stays open and the surface is reported as carrying a superseded
  system. It is never quietly marked done.
