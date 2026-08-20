# Network {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install network-os` | Installs this OS into your environment | Once, first |
| `agentik configure network-os` | Collects the minimum context it needs | After install |
| `agentik run network-os` | Starts the OS | Every session |
| `agentik doctor network-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update network-os` | Updates to the latest version | When a release lands |
| `agentik eval network-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| `/network` | Reviews the network as a portfolio | the relationship records | reciprocity, diversity, dormancy and neglect, each named |
| `/person` | Builds a person brief with provenance | a person | what is known, what is missing, one thing to contribute |
| `/meeting-prep` | Prepares you for a specific meeting | a person and an occasion | context, open loops, and the outcome you actually want |
| `/interaction` | Captures what happened and what was promised | a conversation | an interaction record and commitments with owners |
| `/follow-up` | Drafts a loop-closing follow up in your voice | an open loop | an unsent draft for approval on the exact text |
| `/intro` | Runs a double opt-in introduction | two people and a reason | two consent asks, then the introduction |
| `/nurture` | Designs a relationship rhythm | a set of relationships | a cadence with a reason per contact point |
| `/difficult-conversation` | Prepares a truthful conversation | a situation and the other party | a plan naming the outcome, the words, and the walk-away |
| `/boundary` | Writes and reinforces a boundary | a crossed line | a boundary script and the consequence you will actually hold |
| `/gathering` | Designs a gathering worth attending | an intention and a guest pool | guest logic, shape, and hospitality detail |

---

## Portfolio and preparation

### `/network`

Open the relationship overview and look at the whole network at once:
reciprocity (who gives more than they receive), diversity (whether every tie
comes from the same room), dormancy (who has gone quiet), and neglect (a
commitment you made and did not keep).

```text
/network
/network --dormant
```

**When to reach for it:** on a review cadence, or when the network feels
one-sided and you cannot say why.
**Returns:** a portfolio review that separates relationships that are breathing
by design from relationships that are neglected by accident. The first is a
state, the second is a to-do.

### `/person`

Build a brief on one person from what is actually recorded, with provenance on
each line and the gaps stated plainly.

```text
/person "Amina Rahal"
```

**When to reach for it:** before you need to know someone well and do not.
**Returns:** what is known, where each fact came from, what is inference rather
than fact, and one relevant thing you could contribute to them.

### `/meeting-prep`

Prepare for a specific occasion rather than a person in the abstract: the open
loops between you, what was promised last time, and the outcome you want.

```text
/meeting-prep "Amina Rahal" --occasion "coffee Thursday"
```

**When to reach for it:** the day before, not the hour after.
**Returns:** context, open loops, one thing to give, and the question you would
regret not asking.

## Capture and follow through

### `/interaction`

Record what happened while it is still accurate, and pull out every commitment
either side made.

```text
/interaction "coffee with Amina, she is hiring a data lead, I offered to introduce Jonas"
```

**When to reach for it:** immediately after, always. Memory degrades fastest in
the first hour.
**Returns:** an interaction record with provenance and timestamp, plus each
commitment with an owner and a date. Commitments you own are handed to
Execution {OS}.

### `/follow-up`

Draft the message that closes the loop, in your voice, carrying real context.

```text
/follow-up "Amina" --loop "the intro to Jonas"
```

**When to reach for it:** when you owe something, not when a calendar slot came
up empty.
**Returns:** an unsent draft. If there is genuinely nothing relevant to say, it
tells you that and recommends silence rather than filling the gap.

### `/nurture`

Design a rhythm for the relationships you want to keep warm, and name the ones
you are deliberately letting breathe.

```text
/nurture --tier close
```

**When to reach for it:** when staying in touch has become guilt instead of a
system.
**Returns:** a cadence where each contact point carries a reason, plus the
explicit breathing list with a reactivation condition per relationship.

## Connection

### `/intro`

Run an introduction the only way it should be run: consent from both sides
first, connection second.

```text
/intro "Amina Rahal" "Jonas Vidal" --because "she is hiring a data lead, he is looking"
```

**When to reach for it:** when both people genuinely gain, not when one of them
asked.
**Returns:** the two consent asks, each written for its recipient and each
easy to decline. Nothing is sent until both say yes, and the connection message
still needs your approval on the exact text.

### `/gathering`

Design a gathering: who is in the room, why each person is there, the shape of
the evening, and the hospitality detail that people remember.

```text
/gathering --intention "connect three founders with two operators" --size 8
```

**When to reach for it:** before you send a single invitation.
**Returns:** guest logic with a reason per invitee, the shape, and the detail
that makes it hospitable rather than merely organised.

## Repair and boundaries

### `/difficult-conversation`

Prepare a conversation you are avoiding: what is actually true, what outcome
you want, and how to say it without either flinching or attacking.

```text
/difficult-conversation "Jonas" --about "he committed to the deck twice and missed both"
```

**When to reach for it:** early. A conversation held late costs more trust than
the problem did.
**Returns:** the plan, the opening words, the outcome you are aiming for, and
the point at which you stop.

### `/boundary`

Write a boundary and the consequence you are actually willing to hold.

```text
/boundary --with "a client texting at midnight"
```

**When to reach for it:** the second time, not the fifth.
**Returns:** a script in your voice, and an explicit consequence. A boundary
without a consequence you will hold is a preference, and it is labelled as one.

---

## Command summary

| Command | Does |
|---|---|
| `/network` | reviews the network as a portfolio |
| `/person` | builds a person brief with provenance |
| `/meeting-prep` | prepares you for a specific meeting |
| `/interaction` | captures what happened and what was promised |
| `/follow-up` | drafts a loop-closing follow up in your voice |
| `/intro` | runs a double opt-in introduction |
| `/nurture` | designs a relationship rhythm |
| `/difficult-conversation` | prepares a truthful conversation |
| `/boundary` | writes a boundary and its consequence |
| `/gathering` | designs a gathering worth attending |
