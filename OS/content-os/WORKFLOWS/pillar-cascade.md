# Pillar cascade

Turn one real piece of captured material into a pillar asset and a coherent set
of native packages, without any of them inventing anything.

## Trigger

`/story-mine` surfaces a candidate that clears the bar, or the calendar reaches
a pillar slot whose job is already stated.

## Steps

1. **The OS pulls the constraints** and names the versions in use: the claim
   from Positioning {OS}, the voice rules and visual system from Brand {OS}.
2. **The OS classifies the candidate.** If it is narrative, it is referred to
   Storyteller {OS} and this workflow waits for a story object carrying a truth
   class, a consent record and a release verdict. Content does not deepen,
   restructure or fill gaps in the narrative itself.
3. **The OS labels every material claim** on the E1 to E5 scale. Claims that
   cannot be labelled are dropped and the drop list is reported.
4. **The OS builds the pillar** (`/pillar`), stating the asset's one job in the
   audience journey.
5. **The OS runs QA**: does the pillar contradict the claim, does it break a
   Brand voice rule, does any sentence assert an E4 or E5 claim as settled.
   Each failure is named against the rule it breaks.
6. **The OS clears rights**: copyright, likeness, privacy, platform rules,
   music and image licences, advertising disclosure, accessibility. Anything
   unresolved emits `content.rights.blocked` and the cascade does not start.
7. **A human approves the exact pillar text.**
8. **The OS cascades** (`/cascade`) into one native package per selected
   surface. Each package is rebuilt for that surface's grammar: its own hook,
   its own structure, its own length logic.
9. **The OS checks each package against the crop test**: if the package is the
   pillar shortened, it is rejected and rebuilt.
10. **The OS produces briefs** (`/visual-brief`, `/video-brief`,
    `/sound-brief`) where the surface needs them, each carrying rights,
    licences and accessibility.
11. **A human approves each package's exact text and assets**, per surface.
12. **The OS schedules the packages** into calendar slots whose job matches the
    package's job, and hands the calendar update to Growth {OS} and Sales {OS}.

## Completion test

The pillar exists, carries a stated job, and every material claim in it has an
E1 to E5 label. If it derived from a narrative candidate, a Storyteller story
object with a release verdict is attached, and that verdict is not
`DO NOT PUBLISH` or `NEEDS TRUTH CHECK`.

Every package in the cascade has: a surface, its own hook and structure, a
recorded rights clearance, an accessibility statement, and a human approval
record carrying the exact text that will ship. No package passes the crop test
as a crop.

A cascade in which any package shares the pillar's hook verbatim fails this
test.

## Failure and abort

- **Story object missing or its verdict is `NEEDS TRUTH CHECK` or
  `DO NOT PUBLISH`:** abort the cascade, leave the slot empty, report why. The
  calendar does not override the verdict.
- **Rights, licence, likeness or consent unresolved:** emit
  `content.rights.blocked`, stop before publication, and name exactly what
  would clear it.
- **The pillar contradicts the Positioning claim:** stop, state the
  contradiction, escalate to Positioning {OS}. Do not reword the pillar to fit.
- **A Brand voice rule is broken and the fix would break the surface's
  grammar:** report the conflict to Brand {OS} and Content, do not silently
  pick one. The package waits.
- **A claim cannot be labelled:** drop it. Never soften an unlabelable claim
  into a vaguer version of itself.
- **Human approval refused on a package:** that surface is dropped from the
  cascade and the refusal is recorded. The other surfaces proceed.
