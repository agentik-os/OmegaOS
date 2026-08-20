# Agent {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. A command that is not documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install agent-os` | Installs this OS into your environment | Once, first |
| `agentik configure agent-os` | Collects owners, escalation channels and default budgets | After install |
| `agentik run agent-os` | Starts the OS | Every session |
| `agentik doctor agent-os` | Checks config, adapters, tool contracts and dependencies | When something is off |
| `agentik update agent-os` | Updates to the latest version | When a release lands |
| `agentik eval agent-os` | Runs its evaluation suite | Before trusting it |

## OS commands

The OS answers to `/agent`.

### `/agent design <job>`

Turn a job into an agent design: one sentence job, boundary, escalation path,
named owner.

**When to use it:** before writing any prompt, and only after AI Logic {OS} has
confirmed the job needs judgment.
**Returns:** the design, or a refusal naming what is missing. A job that takes
two sentences comes back as two agents.

### `/agent brief <agent>`

Write the four block executable brief.

**When to use it:** once a design is accepted, and again whenever a debrief
changes it.
**Returns:** objective, constraints, a mechanically verifiable done test, and do
not touch. A brief with an unfillable block is returned red and blocking rather
than softened.

### `/agent tools <agent>`

Compute the minimum tool grant and tie each capability to a step.

**When to use it:** before dispatch, and again at debrief to remove what went
unused.
**Returns:** the grant, the step justifying each entry, and the subset that needs
human approval because it writes, sends, pays, publishes or deletes.

### `/agent dispatch <agent>`

Start the agent against its brief and its grant.

**When to use it:** when the brief is complete and the escalation path is
written.
**Returns:** the run identifier, the budget ceiling, and the verification command
that will decide whether it is done.

### `/agent supervise <agent>`

Classify a running agent and act on the classification.

**When to use it:** while an agent is running with nobody watching.
**Returns:** one of four states with the evidence for it: working (no action),
stalled (nudged), blocked (escalated), asking (escalated to the owner, never
answered on their behalf). The nudge budget resets whenever real progress
advances.

### `/agent verify <run>`

Run the brief's done test against the finished work.

**When to use it:** every time an agent claims success. Every time.
**Returns:** pass or fail with the command output. The agent's own claim is
reported separately from the verification, so the two can disagree visibly.

### `/agent debrief <run>`

Turn a run into a change to the brief.

**When to use it:** after every run, including successful ones.
**Returns:** what was produced, what it cost, which grants went unused, which
brief block was missing, and either the amendment or an explicit statement that
the brief stands.

### `/agent roster`

List every agent with its owner, last run and score trend.

**When to use it:** periodically, and before designing a new agent that may
already exist.
**Returns:** the roster, agents with no owner listed individually, and agents
that have not been useful in a stated period flagged for retirement.

### `/agent retire <agent>`

Retire an agent, close its dispatch paths and remove its residue.

**When to use it:** when it has stopped earning its keep, or its job disappeared.
**Returns:** the reason, the dispatch paths closed, the residue removed, and a
refusal if a live mission still routes to it.

## Command summary

| Command | Mode | Does |
|---|---|---|
| `/agent design` | design | job, boundary, escalation, owner |
| `/agent brief` | brief | the four block executable brief |
| `/agent tools` | grant | minimum tool grant, justified per step |
| `/agent dispatch` | run | start against brief and grant |
| `/agent supervise` | supervise | four state classification and the matching action |
| `/agent verify` | supervise | run the done test, do not trust the claim |
| `/agent debrief` | debrief | what changed in the brief |
| `/agent roster` | roster | who exists, who owns them, what they are worth |
| `/agent retire` | retire | close it down cleanly |
