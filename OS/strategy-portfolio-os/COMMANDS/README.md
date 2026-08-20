# Strategy & Portfolio {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install strategy-portfolio-os` | Installs this OS into your environment | Once, first |
| `agentik configure strategy-portfolio-os` | Collects the minimum context it needs | After install |
| `agentik run strategy-portfolio-os` | Starts the OS | Every session |
| `agentik doctor strategy-portfolio-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update strategy-portfolio-os` | Updates to the latest version | When a release lands |
| `agentik eval strategy-portfolio-os` | Runs its evaluation suite | Before trusting it |

## OS commands

These ten are **internal router modes of this pack, not separately registered
OmegaOS slash commands**. You say the word or type the command inside a running
session and the OS resolves it itself, through `system/ROUTER.md` and
`config/router.json`. The default mode, when nothing is named, is `diagnose`.

Routing priority, in order: safety, legal and privacy boundary first, then an
explicit command, then user intent, then evidence availability, then the
cheapest reversible action, then a handoff when another OS owns the next
responsibility.

### `/strategy` (mode: design)

Open strategic design and build the strategy kernel: diagnosis, guiding policy,
coherent actions, allocation, assumptions, metrics and review triggers. Runs the
strategy kernel protocol.

**When to use it:** at the start of a new venture, a new season, or after a
diagnosis has been accepted and the policy still has to be chosen.
**Returns:** the kernel, with the guiding policy stated as what it rules out,
and each action shown to reinforce the others.

### `/diagnosis` (mode: diagnose)

Define the critical challenge behind the ambition, with the evidence and the
obstacles that make it hard. This is the default mode, and the one most often
skipped.

**When to use it:** whenever a goal is being treated as a strategy, or when
several problems are competing and nobody has said which one is the problem.
**Returns:** the critical challenge as a single obstacle a policy could act on,
the rejected candidate challenges with the reason, and every material claim
labelled E1 to E5.

### `/portfolio` (mode: portfolio)

Inventory every active and proposed bet, including hidden maintenance and
half-finished work still consuming attention. Score fit, evidence, upside,
learning value, cost and downside, check capacity, then choose fund, experiment,
hold, pause or kill per item. Runs the portfolio council protocol.

**When to use it:** when you cannot say how many things you are actually
running, or when the calendar disagrees with the stated priorities.
**Returns:** the scored inventory with a status, owner, resource cost and kill
criteria per item, plus the not-doing list.

### `/prioritize` (mode: portfolio)

Rank competing initiatives against the kernel and the real constraint set, and
state the opportunity cost of each funded one.

**When to use it:** when there are more candidates than the period can carry.
**Returns:** the ordered list, the excluded candidates named rather than
silently dropped, and, where two candidates cannot be separated on the available
evidence, the one piece of evidence that would separate them and its cost.

### `/scenario` (mode: scenario)

Identify the critical uncertainties, build two or more distinct plausible
worlds, describe the implications, define signposts, and prepare no-regret moves
and contingent options. Runs the scenario planning protocol.

**When to use it:** when a decision hangs on an uncertainty that cannot be
resolved before the deadline.
**Returns:** the scenarios, each with an observable signpost and a named
watcher. It will not return a single predicted future with false precision.

### `/strategic-decision` (mode: decision)

Structure one consequential choice: decision, deadline and authority; facts,
unknowns and assumptions; alternatives; expected value and risk; reversibility;
ethical constraints; recorded dissent; the decision; the review trigger. Runs
the strategic decision memo protocol.

**When to use it:** before any choice you would want a written record of in six
months.
**Returns:** the decision memo, with the reversibility class and the review
trigger stated. Committing capital, killing or pausing a major project, or any
people decision inside it still requires human approval before execution.

### `/quarter-plan` (mode: quarter)

Review the previous period's evidence, select a small number of strategic
outcomes, assign resources and owners, define leading and lagging signals,
specify the exclusions, and hand off to Execution {OS} and the relevant product
OS units. Runs the quarterly strategy protocol.

**When to use it:** at the start of a period, or immediately after the previous
one closes.
**Returns:** the quarterly plan plus the execution packet emitted as
`strategy.execution_packet.created`. If the allocation exceeds real capacity it
returns the overcommitment instead of the plan, and names what must be cut.

### `/kill-review` (mode: review)

Compare the original thesis and thresholds against the actual evidence, identify
sunk-cost reasoning, choose continue, narrow, pivot, pause or kill, capture the
reusable learning and release the resources. Runs the kill review protocol.

**When to use it:** when a review trigger fires, a signpost is hit, or a project
is limping and nobody agreed in advance what would stop it.
**Returns:** the verdict per item, the learning captured, and the released
resources either re-assigned or explicitly banked. A consequential pause or kill
waits for `change.approved` before its event is emitted.

### `/one-page-strategy` (mode: design)

Compress the kernel to one page: critical challenge, guiding policy, coherent
actions, allocation, metrics and kill triggers.

**When to use it:** when the strategy has to survive being read by someone who
was not in the session, or when you suspect the kernel does not actually fit on
a page because it is not yet a choice.
**Returns:** the one-page memo. If it does not compress, that is the finding.

### `/not-doing` (mode: portfolio)

Define and publish the exclusions: what is deliberately not being done, why, and
the condition that would reopen each one.

**When to use it:** at the end of every prioritization and every quarterly plan,
and whenever focus is quietly eroding.
**Returns:** the not-doing list, each entry with its reason and its reopening
condition, plus the low-cost options being deliberately preserved.

## Reference runtime

The pack ships a provider-neutral, standard-library-only reference runtime. It
is not a production database or an LLM adapter: it exists to prove the pack is
self-describing and integrity-checkable.

```bash
python ~/.omega/skills/strategy-portfolio-os/runtime/os_runtime.py info
python ~/.omega/skills/strategy-portfolio-os/runtime/os_runtime.py route "/strategy"
python ~/.omega/skills/strategy-portfolio-os/runtime/os_runtime.py validate
python ~/.omega/skills/strategy-portfolio-os/runtime/os_runtime.py event note '{"example": true}'
```

| Invocation | What it does | Returns |
|---|---|---|
| `os_runtime.py info` | reports the pack identity and inventory | name, version, counts of agents, skills, protocols and schemas |
| `os_runtime.py route "<command>"` | resolves a router command to its mode | the mode and purpose, or the default mode `diagnose` |
| `os_runtime.py validate` | sha256 integrity check of every shipped file | per-file pass or fail, never a single summary badge |
| `os_runtime.py event <type> <json>` | appends a typed event record | the recorded event |

Durable strategy records follow the seven JSON schemas under `schemas/`:
strategic objective, strategic bet, project portfolio item, resource allocation,
scenario, strategic decision, strategic metric.

## Command summary

| Command | Mode | Does | Returns |
|---|---|---|---|
| `/strategy` | design | open strategic design, set the kernel | diagnosis, guiding policy, coherent actions, allocation, metrics, review triggers |
| `/diagnosis` | diagnose | define the critical challenge | one obstacle a policy can act on, with E1 to E5 labels |
| `/portfolio` | portfolio | review all projects and bets | scored inventory with status, owner, cost and kill criteria |
| `/prioritize` | portfolio | rank competing initiatives | ordered list plus the stated opportunity cost of each funded bet |
| `/scenario` | scenario | build future scenarios and signposts | distinct plausible worlds, signposts, no-regret and contingent moves |
| `/strategic-decision` | decision | structure a consequential choice | decision memo with authority, reversibility and review trigger |
| `/quarter-plan` | quarter | create the quarterly strategy | outcomes, owners, allocation, signals, exclusions, execution packet |
| `/kill-review` | review | decide continue, narrow, pivot, pause or kill | verdict per item, learning captured, resources released |
| `/one-page-strategy` | design | produce a concise strategy memo | the kernel on one page |
| `/not-doing` | portfolio | define the exclusions | the not-doing list with reopening conditions |
