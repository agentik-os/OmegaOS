# Process & SOP {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install process-sop-os` | Installs this OS into your environment | Once, first |
| `agentik configure process-sop-os` | Collects the minimum context it needs | After install |
| `agentik run process-sop-os` | Starts the OS | Every session |
| `agentik doctor process-sop-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update process-sop-os` | Updates to the latest version | When a release lands |
| `agentik eval process-sop-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| `/sop-capture` | Extracts the procedure from the expert | an observed run, or the expert's account | the raw steps, the exceptions, and the judgement calls in their own words |
| `/sop-decisions` | Turns judgement into decisions | the captured judgement calls | named branches with the criteria that select each one |
| `/sop-write` | Drafts the SOP | the capture and the decisions | purpose, trigger, prerequisites, inputs, steps, decisions, quality bar, failure modes, escalation, time estimate |
| `/sop-test` | Runs the novice test | a person who has never done it, and the draft | the output they produced, and every stall, question and guess, timestamped |
| `/sop-fix` | Repairs the SOP | the test record | one change per stall, at the cause; unfixable stalls recorded as accepted judgement points |
| `/sop-release` | Publishes it | a draft that passed the test | a versioned SOP with an owner who accepted, a review date, and a home in Documentation {OS} |
| `/sop-review` | Maintains it | the review date, or a change in the work | a new version with a change note, or a confirmation that the current one still holds |
| `/sop-retire` | Ends it | a procedure whose work is gone or automated | an archived SOP and a pointer to what replaced it |

### When to reach for which

- `/sop-capture` while the expert is available; their time is the scarce input.
- `/sop-decisions` immediately after, while the "it depends" answers are fresh.
- `/sop-test` before anyone believes the SOP. An untested SOP is a draft.
- `/sop-release` only after a novice produced an acceptable output unaided.
- `/sop-review` on the date, and immediately whenever the work changes.

## The rule the commands enforce

A stall in the novice test is a defect in the SOP, never a defect in the novice.
`/sop-fix` is written against that assumption and will ask what the step failed
to say, not what the person failed to understand.

## Command summary

| Command | Does |
|---|---|
| `/sop-capture` | extracts the real procedure |
| `/sop-decisions` | turns "it depends" into criteria |
| `/sop-write` | the SOP in the house shape |
| `/sop-test` | a novice runs it, unaided |
| `/sop-fix` | one repair per recorded stall |
| `/sop-release` | versioned, owned, dated, published |
| `/sop-review` | keeps it true as the work changes |
| `/sop-retire` | archives it with a pointer forward |
