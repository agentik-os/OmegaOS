# Sales {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install sales-os` | Installs this OS into your environment | Once, first |
| `agentik configure sales-os` | Collects the minimum context it needs | After install |
| `agentik run sales-os` | Starts the OS | Every session |
| `agentik doctor sales-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update sales-os` | Updates to the latest version | When a release lands |
| `agentik eval sales-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| `/sales` | Opens the OS and reports what needs attention today | nothing | opportunities with a next action, and those that have gone quiet |
| `/sales-pipeline` | Shows pipeline state by stage, grounded in buyer actions | nothing | every open opportunity with stage, owner, next action and last buyer action |
| `/qualify` | Qualifies a lead on fit, need, authority, timing and budget | a lead | a qualification verdict with every unknown marked unknown |
| `/disqualify` | Records a disqualification and the reason given to the prospect | an opportunity | a disqualification record and the message to send |
| `/sales-call` | Prepares one specific conversation | an opportunity, a meeting purpose | an agenda, the permitted claim set, and the questions to ask |
| `/sales-discovery` | Runs discovery and captures the need in the buyer's words | a qualified opportunity | a need record, quoted, with the decision process named |
| `/sales-objection` | Answers an objection using only sourced claims | an objection | a response with a source per claim, or an explicit abstention |
| `/sales-proposal` | Drafts a proposal against a live offer and price book version | a need record | a proposal naming offer version, price book version and claim sources |
| `/sales-negotiate` | Settles price and terms inside the discount policy | a requested position | a settled position, or an escalation to the Pricing floor owner |
| `/sales-close` | Runs the close and records the signed commitment | agreed terms | a signed commitment with the exact approved contract text |
| `/closed-won` | Produces the handoff carrying every promise made | a signed commitment | the closed won handoff, emitted to Revenue and Delivery |
| `/loss-review` | Records why a deal was really lost | a closed lost opportunity | a loss review with the stated reason and the believed reason |
| `/sales-review` | Audits pipeline hygiene and claim discipline | the open pipeline | findings: stale stages, unsourced claims, unrecorded promises |

---

### `/sales`

```
/sales
```

Opens Sales {OS} and reports what actually needs attention: opportunities with
a next action due, and opportunities where the buyer has done nothing for
longer than the stage allows. Silence is surfaced, not averaged away.

**When to reach for it:** at the start of every selling day.
**Returns:** the attention list, plus any blocking condition, such as a
missing offer definition or an expired price book version.

### `/sales-pipeline`

```
/sales-pipeline
```

Shows the pipeline by stage. A stage is grounded in something the buyer did, a
date, a document, a signature, never in how the last call felt.

**When to reach for it:** weekly, and before any forecast conversation.
**Returns:** every open opportunity with stage, owner, next action and last
buyer action. Opportunities whose stage rests only on seller optimism are
flagged rather than counted.

### `/qualify`

```
/qualify lead-42
```

Works through fit, need, authority, timing and budget.

**When to reach for it:** before any time is invested in a lead.
**Returns:** a verdict with each dimension answered or explicitly unknown.
Unknowns stay unknown: assuming in the seller's favour is how a pipeline fills
with deals nobody can close.

### `/disqualify`

```
/disqualify lead-42 --reason "no budget this fiscal year, revisit in Q1"
```

Ends it, plainly, and tells the prospect why.

**When to reach for it:** the moment the fit fails, from any stage.
**Returns:** the disqualification record and the message to send. This is a
first class outcome, not a failure path: a bad fit close costs the refund, the
support load, the reference that never comes, and the story they tell.

### `/sales-call`

```
/sales-call opp-17 --purpose "technical evaluation with the CTO"
```

Prepares one specific conversation: the agenda, the questions worth asking,
and the permitted claim set assembled from the claim ledger and Storyteller
{OS} truth verdicts.

**When to reach for it:** before every meaningful conversation.
**Returns:** the brief, and an explicit list of what may not be said, so the
line is visible before the call rather than crossed during it.

### `/sales-discovery`

```
/sales-discovery opp-17
```

Runs the discovery conversation and captures the need in the buyer's own
words, quoted rather than translated into offer language.

**When to reach for it:** after qualification, before any proposal.
**Returns:** a need record: the problem quoted, its cost to the buyer, and the
decision process with the actual decision maker named.

### `/sales-objection`

```
/sales-objection opp-17 --objection "you have never done this at our scale"
```

Answers using only claims that trace to a source.

**When to reach for it:** in preparation, and immediately after a call where
an objection landed.
**Returns:** a response with a source per claim. Where no source exists, it
returns the abstention text instead: saying that you do not know, and will not
guess, is a legitimate answer in a sales call.

### `/sales-proposal`

```
/sales-proposal opp-17
```

Drafts the proposal against a live offer and a specific price book version.

**When to reach for it:** once the need maps to an offer.
**Returns:** the proposal naming the offer version, the price book version,
the guarantee and a source for every claim in the document. It sends nothing.
A human approves the exact text first, always.

### `/sales-negotiate`

```
/sales-negotiate opp-17 --requested 0.78
```

Settles price and terms inside the discount policy from Pricing {OS}.

**When to reach for it:** when the prospect pushes on price.
**Returns:** a settled position if the number is inside the policy, otherwise
an escalation to the named floor owner. It never produces a number below the
floor: the floor is not the seller's to move.

### `/sales-close`

```
/sales-close opp-17
```

Runs the close and records the commitment.

**When to reach for it:** when terms are agreed and the contract is ready.
**Returns:** the signed commitment with the exact approved contract text
attached. It refuses to complete on any scope, deliverable or date that is not
in the offer and has not been separately approved by a human.

### `/closed-won`

```
/closed-won opp-17
```

Builds the handoff by walking back through every conversation on the deal and
recording every promise, including the casual ones.

**When to reach for it:** immediately on close, before anything is forgotten.
**Returns:** the closed won handoff, emitted to Revenue {OS} and Delivery &
Customer Success {OS}: agreed scope, price and terms, and the full promise
list. A promise made in a call and not written here is the single most common
source of delivery failure, so the command blocks while any known promise is
missing.

### `/loss-review`

```
/loss-review opp-09
```

Records why the deal was lost.

**When to reach for it:** on every loss, including the ones that feel obvious.
**Returns:** two separate fields, the reason the buyer gave and the reason you
believe, plus the stage at which it was really lost. The pattern is emitted to
Growth {OS}. Collapsing the two reasons into one is how a business spends a
year fixing a price problem that was a positioning problem.

### `/sales-review`

```
/sales-review
```

Audits the open pipeline for hygiene and claim discipline.

**When to reach for it:** on a cadence, and before any forecast is trusted.
**Returns:** findings: stages resting on no buyer action, claims made without
a source, promises recorded in call notes but absent from a handoff, and
introductions used beyond their consent. It changes nothing on its own.

## Command summary

| Command | Does |
|---|---|
| `/sales` | what needs attention today |
| `/sales-pipeline` | the pipeline, grounded in buyer actions |
| `/qualify` | fit, need, authority, timing, budget |
| `/disqualify` | end it plainly, and say why |
| `/sales-call` | prepare one conversation, and what may not be said |
| `/sales-discovery` | the need, in the buyer's words |
| `/sales-objection` | answer with a source, or abstain |
| `/sales-proposal` | scope, price version, guarantee, claim sources |
| `/sales-negotiate` | settle inside the policy, escalate below the floor |
| `/sales-close` | the signed commitment, on approved text |
| `/closed-won` | the handoff, carrying every promise |
| `/loss-review` | the stated reason and the real one |
| `/sales-review` | pipeline hygiene and claim discipline |
