# Review & Governance {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install review-governance-os` | Installs this OS into your environment | Once, first |
| `agentik configure review-governance-os` | Collects the minimum context it needs | After install |
| `agentik run review-governance-os` | Starts the OS | Every session |
| `agentik doctor review-governance-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update review-governance-os` | Updates to the latest version | When a release lands |
| `agentik eval review-governance-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | Mode | What it does | Input | Output |
|---|---|---|---|---|
| `/review` | weekly | Opens a review at the default cadence | the period's evidence | findings and decisions with owners and dates |
| `/daily-review` | daily | Runs the short daily reflection | the day's record from Execution {OS} | what happened against what was intended, in minutes |
| `/weekly-review` | weekly | Runs the operating review | weekly resets, project status, open decisions | what moved, what did not, and this week's decisions with owners |
| `/monthly-review` | monthly | Runs the metrics review | readings and breaches from KPI & Analytics {OS} | a decision or a recorded deferral for every breach, plus systemic findings |
| `/quarterly-review` | quarterly | Runs strategic governance | policy set, decision rights, risk register, change history | an updated policy set, decision rights map and risk register |
| `/postmortem` | postmortem | Analyses an incident or a material failure | the sequence and the people involved | a blameless account, the contributing conditions, and one change with an owner |
| `/policy` | policy | Creates or audits a written policy | the rule and its scope | scope, rule, exceptions, owner, review date, and the decision rights it grants |
| `/change-request` | change | Authorises a consequential change | the proposal, its risk and its reversal path | approved, rejected or deferred, with conditions and a verification test |
| `/risk-register` | monthly | Reviews the risks | open risks and new ones | each risk with a trigger, a response, an owner and a review date |
| `/ai-governance` | ai-risk | Applies AI risk governance | an AI system that shapes a consequential decision | what it may decide, what it may not, its named human oversight point, and its failure behaviour |
| `/verify-change` | verify | Judges an approved change after the fact | the change and its verification test | did it do what it claimed: standardise, adjust or revert |

### When to reach for which

- On the cadence: `/daily-review`, `/weekly-review`, `/monthly-review`,
  `/quarterly-review`. Each is deliberately a different length.
- The moment something fails materially: `/postmortem`, while the sequence is
  still recoverable.
- Whenever a domain OS proposes a change to its own boundary, policy or control:
  `/change-request`. That OS may not approve it itself.
- After every approved change: `/verify-change`. Approval without verification
  is a wish.

## The separation this OS enforces

A domain OS proposes. This OS decides. Execution {OS} cannot widen its own
scope, Operations {OS} cannot retire a control, Client {OS} cannot create a new
exception class, KPI & Analytics {OS} cannot retire a metric others depend on.
Every one of those arrives here as a `/change-request` with its evidence, its
risk and its reversal path.

Where the same person holds both roles, the separation is temporal and written:
the proposal is recorded before the decision, and the record names the conflict.

## Command summary

| Command | Does |
|---|---|
| `/review` | the default review at the current cadence |
| `/daily-review` | minutes, on the day's record |
| `/weekly-review` | the operating review and its decisions |
| `/monthly-review` | metrics against thresholds |
| `/quarterly-review` | policy, decision rights, risk |
| `/postmortem` | blameless account and one owned change |
| `/policy` | a written rule with an owner and an expiry |
| `/change-request` | authorisation with conditions and a test |
| `/risk-register` | risks with triggers, not adjectives |
| `/ai-governance` | limits and a real human oversight point |
| `/verify-change` | standardise, adjust or revert |
