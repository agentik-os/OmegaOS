# OS Builder {OS}: Dashboard Interface

A persistent view for someone who returns to a build repeatedly, or who is
running several. It answers three questions and refuses to answer anything else,
because a dashboard that shows everything is read by nobody.

1. **Where is this build?** Which of the fifteen phases, and is its gate open.
2. **What is blocking it?** The specific failing item, not a status colour.
3. **Is it getting better?** The score trend across versions, and the repair
   cycles spent.

## Applies when

A build spans more than one session, several builds run at once, or the suite
itself is being audited. A single session build does not need this surface: the
chat ledger already carries the state, and duplicating it into a dashboard
creates a second place for it to be wrong.

## The pipeline row

The primary element. Fifteen phases, 0 through 14, with the public twelve stage
banner mapped onto them so the two vocabularies never diverge:

| Banner stage | Phases | Gate to leave it |
|---|---|---|
| IDEA | 0 Intake, 1 Capability Definition | the capability is explainable without the words folder, prompt or model |
| VALUE | 2 Value Proposition | the promise is specific and falsifiable |
| RESEARCH | 3 Research | operating logic is source backed or labelled original synthesis |
| SKILL | 4 Skill Architecture | a human can learn the capability without the AI |
| WORKFLOW | 5 Operating Model | every mode has an entry condition and a completion test |
| ARTIFACTS | 6 Artifact Architecture | the primary artifact has a defined shape |
| PACKAGE | 7 Package Design, 8 Build | `validate_os.py` passes at CORE tier |
| TEST | 9 Test | the seven families run, none skipped |
| RED TEAM | 10 Red Team | the twelve adversarial cases run, escapes recorded |
| SCORE | 11 Score | sixteen dimensions scored with evidence |
| REPAIR | 12 Repair | no dimension below 4, no CRITICAL waived |
| RELEASE | 13 Adapters, 14 Release | eighteen gate items `YES`, ZIP hash reproducible |

Each phase renders in one of five states, and the states are distinguishable
without colour: `done`, `doing`, `blocked`, `todo`, `skipped with reason`. There
is no sixth state, and in particular there is no "in progress but fine", which
is how a blocked phase stays invisible for a week.

## The gate strip

Three gates, always visible, each showing its last real result and when it was
produced. A gate result older than the most recent file change renders as
**stale**, not as passing. That single rule is what stops a dashboard reporting
green over a package that has been edited since it was last graded.

```
GATE A  CONTRACT   FULL 19/23   FAIL 4      validate_os.py, 12 min ago
GATE B  QUALITY    4.19         BLOCKED     score_os.py, STALE
GATE C  BEHAVIOUR  10/12 cases  2 escapes   manual, 2 days ago
```

Gate A expands to the failing items verbatim, in `verify.py`'s own vocabulary
(`STRUCTURE`, `AUTHORED`, `MANIFEST`, `DEPS`, `NODASH`, `SUBSTANCE`), because
that is the string the builder will grep for. Paraphrasing a machine failure
into friendlier words means the reader cannot find it.

Gate B expands to the dimensions below 4, CRITICAL ones first.

Gate C expands to the adversarial cases that escaped, each with its fail
signature, so the reader sees what the OS actually did wrong rather than that
something failed.

## The tier meter

Two bars, never merged, matching the grader:

```
CORE   #####  5/5     the OS exists and is usable
FULL   ###..  19/23   the OS is releasable
```

Merging them into one percentage is the specific misreport this surface exists
to prevent. A unit at CORE 5/5 and FULL 12/23 is finished wave 1 work, and one
merged bar at 52 percent describes it as failing.

## The register panel

Three counts, each a link to its list:

- **Open assumptions**, with how many have a falsifier. An assumption with no
  falsifier renders in an error state: it is a belief, and it will never resolve.
- **Unresolved conflicts**, evidence items in state `CONFLICTING`. These block
  release, so the count is never decoration.
- **Open red team escapes**, each with the case that produced it and whether a
  regression test now exists.

## The trend panel

Only appears from the second scored version onward, and it is the only element
here that looks backward. Per version: the date, the average, the lowest
dimension, the gate verdict, and the number of repair cycles spent.

Two derived signals, because they are what the panel is actually for:

- **A dimension that has been repaired twice and not moved.** The diagnosis is
  wrong, and a third cycle on the same theory will also fail.
- **An average that rose while the lowest dimension did not.** The build is
  improving where it was already strong, which is how a package reaches 4.4
  average with a 3 in security.

## Multi build view

When several builds are open, one row each: slug, phase, the three gate states,
and the single most blocking item as text. Sorted by **blocked first**, then by
phase ascending, so the build nearest the start of the pipeline and stuck is at
the top. Sorting by recency puts the build somebody just touched at the top,
which is the build that needs attention least.

## What the dashboard never shows

- A green state derived from anything other than a command's real exit code.
- An aggregate score for the suite. Seventy three units averaged into one number
  hides the one unit at 2 in security, and that unit is the entire reason to
  look.
- A percentage complete for a phase. A phase is done or it is not; a phase that
  is 70 percent done is a phase whose gate has not been tested.
- Any real name, figure or organisation from a requester's domain.

## Degradation

With no persistent surface, the dashboard degrades to the ledger block printed
at each phase boundary in [`chat.md`](chat.md), plus the three gate commands run
on demand. The degradation is honest as long as the staleness rule survives: a
printed gate result carries the timestamp of the run that produced it, and a
reader comparing it against a later file change can still see that it is stale.
