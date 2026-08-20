# Validation {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install validation-os` | Installs this OS into your environment | Once, first |
| `agentik configure validation-os` | Collects the minimum context it needs | After install |
| `agentik run validation-os` | Starts the OS | Every session |
| `agentik doctor validation-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update validation-os` | Updates to the latest version | When a release lands |
| `agentik eval validation-os` | Runs its evaluation suite | Before trusting it |

## OS commands

### `/validate <plan or claim>`

The root command. Given a plan, it runs `FRAME` and returns the claim register.
Given a single claim, it goes straight to `DESIGN`.

**When to use it:** whenever you are about to build on a belief.
**Returns:** either the ranked claim register, or a test spec awaiting your
signature.

### `/claims [--from <artifact>]`

Extract every load-bearing claim from a plan, deck, concept or blueprint and
rewrite each one so it can be false.

**When to use it:** before any funding, build or launch decision.
**Returns:** the claim register: claim, owner, current confidence, cost of being
wrong, and whether it is already settled by a prior verdict.

### `/rank`

Order the open claims by cost of being wrong times probability of being wrong.

**When to use it:** when you have more claims than test budget.
**Returns:** the ordered list, with the claims deliberately left untested named
and justified.

### `/design <claim-id>`

Design the cheapest instrument that can still produce a kill for that claim.

**When to use it:** once a claim is selected.
**Returns:** instrument, sample, threshold, stopping rule, cost in money and
calendar days, kill criteria, and what dies if the claim dies.

### `/sample <claim-id>`

Compute the sample the stated threshold actually requires, and what the
affordable sample can support instead.

**When to use it:** before signing a threshold you cannot afford to measure.
**Returns:** required sample, affordable sample, and the weaker claim the
affordable sample could settle honestly.

### `/sign <claim-id>`

Freeze the threshold and stopping rule and record the owner's acceptance.

**When to use it:** immediately before running, never after.
**Returns:** the signed spec with a timestamp. After this, the spec is
immutable; changing it invalidates the run.

### `/run <claim-id>`

Execute the signed spec and open the run log. Every step that touches a real
person, a public surface or money pauses for approval first.

**When to use it:** once the spec is signed and approved.
**Returns:** the run log, live, including every deviation as it happens.

### `/verdict <claim-id>`

Compare the result to the signed threshold and issue the verdict.

**When to use it:** when the stopping rule fires or the sample completes.
**Returns:** CONFIRMED, KILLED, INCONCLUSIVE or INVALID, the number it was
measured against, and what changes in the plan.

### `/kill <claim-id>`

Write the kill note: what dies, what survives, and the next cheapest claim.

**When to use it:** after a KILLED verdict, or when you decide to stop paying
for a claim you cannot afford to settle.
**Returns:** the kill note and the updated claim register.

### `/audit <artifact>`

Inspect something already declared validated.

**When to use it:** when you inherit a deck, a research pack or a blueprint that
uses the word "validated".
**Returns:** per claim, what was actually measured, what is being asserted, and
the gap between them.

### `/queue`

Show the test queue: what is designed, what is signed, what is running, and
what is waiting on approval.

**When to use it:** at the start of a session, and in any review.
**Returns:** the queue, ordered by expected information gain per unit of cost.

## Command summary

| Command | Does | Returns |
|---|---|---|
| `/validate` | entry point: plan to claims, or claim to test | claim register or test spec |
| `/claims` | extract implicit claims, made falsifiable | the claim register |
| `/rank` | order claims by cost of being wrong | ordered list plus the deliberate exclusions |
| `/design` | cheapest instrument that can still kill the claim | full test spec |
| `/sample` | what sample the threshold requires | required, affordable, and the honest weaker claim |
| `/sign` | freeze the threshold before the data | immutable signed spec |
| `/run` | execute the spec, log deviations | live run log |
| `/verdict` | measure result against the signed threshold | one of four verdicts, plus plan impact |
| `/kill` | record what a kill removes | kill note, updated register |
| `/audit` | check a claim someone else called validated | measured versus asserted, per claim |
| `/queue` | state of every test | ordered test queue |
