# OS Builder {OS}: Memory Policy

Most OSes remember things about a user. OS Builder remembers things about
**other OSes**, and that changes what memory is for. Its durable state is the
accumulating knowledge of what the suite already owns, what has been tried and
rejected, and how good each unit actually turned out to be. Without that, every
build re-derives the boundary map from scratch and the suite grows overlapping
units that each believe they own the same decision.

Canonical durable state routes through Context & Memory {OS} via
`memory.record.staged`. This policy declares what OS Builder is allowed to stage
in the first place.

## Tiers

| Tier | Example | Lifetime | User can delete |
|---|---|---|---|
| Temporary | a token count while computing package size | the turn | n/a |
| Session | the build ledger of the OS being built right now | the build | yes |
| Build | the intake record, spec decisions, phase gate results | the built OS's lifetime | yes |
| Suite | the boundary map: which OS owns which capability | until a boundary moves | yes |
| Preference | how this operator wants OSes built (depth, tone, tiering) | until changed | yes |
| Outcome | release scores, repair history, recurring defects | durable | yes |

## What it remembers, and why each one earns its place

**Spec decisions, with the alternative that was rejected and the reason.** The
single most valuable thing OS Builder can carry between builds. A decision
recorded without its rejected alternative is worth almost nothing six months
later, because the next builder cannot tell whether the alternative was
considered and beaten or never considered at all. Stored as: decision, chosen
option, rejected options, the reason, the phase it was made in, and what would
reopen it.

**The boundary map.** For every registered unit: what it owns, what it
explicitly does not own, and where the seam with its neighbours falls. This is
what makes adjacent duplication catchable at intake instead of at review. It is
a projection of every unit's `OS.md` section 2, recomputed when a boundary
moves, never hand-edited.

**Reference sources with their trust class and their `WHERE USED`.** A source
captured once is reusable across builds, which is the only legitimate way to
make research cheaper. The capture record travels; the source text does not.

**Release scores and gate verdicts.** Per unit, per version: the sixteen
dimension scores, the average, the gate result, the date, and who or what
scored it. This is the only way to answer "is the suite getting better" with
anything other than an impression.

**Repair history.** Which dimension failed, what was changed, and whether the
re-score moved. A repair that did not move the score is more informative than
one that did, because it means the diagnosis was wrong.

**Recurring defect patterns.** Defects that appeared in three or more builds get
promoted into the preflight checklist. This is how the OS learns: not by
remembering more, but by turning a repeated mistake into a gate.

**Refusals.** Capability requests that were declined, with the tree branch that
declined them. Otherwise the same request returns in three weeks, phrased
differently, and gets built.

## What it updates

- A boundary map entry, when the owning unit's `OS.md` section 2 changes. The
  old entry is superseded with a pointer, never deleted, because a boundary that
  moved is exactly what explains a stale handoff elsewhere.
- A release score, when a new version is scored. Scores are versioned, not
  overwritten: the trend is the point.
- An assumption's status, when it is confirmed or refuted.
- A preference, when the operator states a different one. Inferred preferences
  are held at lower weight than stated ones and are never presented as stated.

## What it forgets

- The session build ledger, once the OS reaches a terminal state (released, or
  abandoned with a recorded reason).
- Draft package trees and intermediate file contents. The finished package is on
  disk; a memory of an earlier draft is noise that competes with it.
- Scores for a version that was never released, unless the build was abandoned
  and the reason is worth keeping. In that case the reason survives and the
  scores do not.
- Candidate paths under a temporary directory, once the candidate is registered
  or discarded.

## Never stored

- **Credentials, tokens, keys or connection strings**, in any tier, whatever the
  operator's intent. If one appears in an intake it is refused and the intake is
  returned, per [`../REFERENCES/SECURITY.md`](../REFERENCES/SECURITY.md).
- **Real client or personal data from a requester's domain.** Examples are
  anonymised at intake, before anything is staged. A real name that reaches
  memory will eventually reach a shipped example.
- **The text of a licensed or paid corpus.** The capture record and the quoted
  load bearing line are memory; the corpus is not.
- **A score OS Builder did not itself compute.** A claimed quality level with no
  scorecard behind it is stored as a claim by whoever made it, never as a score.
- **Anything the operator marks private**, and nothing at all from a build
  explicitly run in throwaway mode.

## Retrieval

Only what the current phase needs is loaded, and each phase pulls a different
slice:

| Phase | Loads |
|---|---|
| 0 Intake | the boundary map, prior refusals, operator preferences |
| 3 Research | reference sources with a matching trust class and domain |
| 5 Operating model | spec decisions and rejected alternatives from similar units |
| 9 Test, 10 Red team | recurring defect patterns, prior red team escapes |
| 11 Score, 12 Repair | this unit's score history and repair history |

Loading everything is the failure mode this section exists to prevent. A build
that opens with the full boundary map of all seventy three units, every source
ever captured, and every score ever computed has spent its context before phase
0 asks its first question.

## Inspection and removal

Everything durable is inspectable by unit and by build. The operator may delete
any entry. Deleting a boundary map entry re-derives it from disk on next use;
deleting a spec decision is permanent and the build ledger records that a
decision was removed, without its content, so a future reader knows the record
is incomplete rather than believing it is whole.
