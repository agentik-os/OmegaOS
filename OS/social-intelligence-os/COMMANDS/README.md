# Social Intelligence {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install social-intelligence-os` | Installs this OS into your environment | Once, first |
| `agentik configure social-intelligence-os` | Collects the minimum context it needs | After install |
| `agentik run social-intelligence-os` | Starts the OS | Every session |
| `agentik doctor social-intelligence-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update social-intelligence-os` | Updates to the latest version | When a release lands |
| `agentik eval social-intelligence-os` | Runs its evaluation suite | Before trusting it |

`configure` asks what the user will not do in a conversation (their standing
constraints), and whether observations about named third parties may be stored
at all. The second answer defaults to no.

## OS commands

| Command | What it does | When to use it | Returns |
|---|---|---|---|
| `/social` | Opens the OS and routes to the right mode from what you describe | You are not sure whether you need prep, a read or a repair | The mode it selected and why |
| `/prep` | Builds a preparation brief for one named upcoming conversation | Before a conversation you are dreading or cannot afford to fumble | Aim, likely counter-aim, opening line, what to listen for, walk-away line |
| `/read` | Produces a read of an interaction that already happened | After something went sideways and you cannot tell why | Observed, stated, inferred with confidence, plus an alternative reading and the observation that would separate them |
| `/boundary` | Drafts a boundary statement with its consequence | When you keep saying yes, or a limit keeps being crossed | One or two sentences in your register, the consequence, and a direct question about whether you will apply it |
| `/repair` | Drafts a repair attempt after a rupture | When you want to fix it rather than win it | The acknowledgement, the question, and the list of things not to defend |
| `/debrief` | Closes the loop on an interaction that is over | Right after, while you still remember the sequence | Expected against actual, what the read got wrong, one thing to check earlier next time, sent to Journal {OS} on approval |

### `/social`

The entry point when you do not know which mode you need. Describe the
situation in your own words and it selects `PREP`, `READ`, `BOUNDARY`, `REPAIR`
or `DEBRIEF` and says why. If your description contains no observable facts, it
asks for the sequence and the actual words before producing anything.

```
/social
```

**When to use it:** first time, or whenever the situation is tangled enough that
naming the mode is itself the hard part.
**Returns:** the selected mode, the reason, and the first question of that mode.

### `/prep`

Preparation for one specific conversation. It will not proceed until you can
state your aim as an outcome in one sentence, because a script written around a
vague aim is worse than no script.

```
/prep "asking for equity, with my co-founder, Thursday"
```

**When to use it:** before a raise, a renegotiation, a difficult no, a
resignation, a hard piece of feedback.
**Returns:** a one-page brief: your aim, the other side's likely aim, an opening
line in your own words, the two or three things to listen for, and the line you
will not cross. It also names what it could not find (no prior entries about
this person, no declared values) rather than inventing it.

### `/read`

The read of an interaction that already happened. Output is always four
separated blocks: what was observed, what was stated, what is inferred and at
what confidence, and at least one alternative reading that fits the same
evidence. It never labels a person with a clinical or quasi-clinical category,
at any confidence.

```
/read
/read --thread ./meeting-notes.md
```

**When to use it:** after a meeting, a message thread, or a conversation that
left you with a story you are not sure about.
**Returns:** the four blocks, plus the single observation in the next
interaction that would confirm or kill the leading read. If the evidence
supports two incompatible readings equally, it says so and picks neither.

### `/boundary`

Drafts the limit and the consequence, then tests it. The test is a direct
question: will you actually apply this. A consequence you will not apply is not
a boundary, and the command says so and redrafts.

```
/boundary "weekend messages from work"
```

**When to use it:** when a limit keeps being crossed, or when you keep
agreeing to things you resent afterwards.
**Returns:** one or two sentences in your own register, the consequence, and
the verdict on whether it is real.

### `/repair`

After a rupture. It separates the acknowledgement from the justification and
will not attach the second to the first, because that is the move that turns a
repair back into an argument.

```
/repair
```

**When to use it:** when you want the relationship more than you want to be
right.
**Returns:** what to acknowledge, the one question to ask, and an explicit list
of things not to defend in this conversation.

### `/debrief`

The mode users skip, and the one that makes every later read better. It
compares what you expected to what happened and names what the earlier read got
wrong or left unverified.

```
/debrief
```

**When to use it:** immediately after the interaction, while the sequence is
still recoverable.
**Returns:** expected against actual, the errors in the prior read, one thing
to check earlier next time. On your approval it is sent to Journal {OS} as an
entry, and any durable relational fact is offered to Network {OS}. The
superseded read is discarded rather than kept as evidence about the person.

## Command summary

| Command | Does |
|---|---|
| `/social` | routes your situation to the right mode |
| `/prep` | a brief for one upcoming conversation |
| `/read` | observed, stated, inferred, plus the alternative reading |
| `/boundary` | a limit, its consequence, and whether you will apply it |
| `/repair` | acknowledgement without justification |
| `/debrief` | what the read got wrong, sent to Journal {OS} |
