# Customer Discovery {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install customer-discovery-os` | Installs this OS into your environment | Once, first |
| `agentik configure customer-discovery-os` | Collects the minimum context it needs | After install |
| `agentik run customer-discovery-os` | Starts the OS | Every session |
| `agentik doctor customer-discovery-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update customer-discovery-os` | Updates to the latest version | When a release lands |
| `agentik eval customer-discovery-os` | Runs its evaluation suite | Before trusting it |

## OS commands

Commands below the `/interview` line touch a real person. Every one of them
stops at the human approval boundary before anything is sent, offered, recorded
or stored.

### `/discovery [<learning goal>]`

The root command. With a learning goal, it runs `PLAN` and returns a round plan.
With nothing, it asks the only question that matters first: what decision are
you stuck on. With an existing round in progress, it reports where that round
stands.

**When to use it:** at the start of anything involving users.
**Returns:** a round plan awaiting approval, or the state of the current round:
recruited, interviewed, coded, saturated, confirmed.

### `/round [--goal <text>] [--decision <text>] [--n <target>]`

Plan one discovery round: learning goal, the decision it feeds, segment
definition, target N, stopping rule, budget, consent and retention policy.

**When to use it:** before contacting anybody. This is the command that stops a
round from becoming twelve unstructured chats.
**Returns:** the round plan. If no decision changes based on the result, it
returns that objection instead of a plan, and stops.

### `/screener`

Write the screening questions that decide who is in the sample, selecting on
observable recent behaviour rather than on title or self-description.

**When to use it:** immediately after the round plan, before recruiting.
**Returns:** the screener questions, the qualifying answers, the disqualifying
answers, and the traps that catch professional respondents.

### `/recruit [--channel <name>]`

Plan recruiting across the channels you actually have, label the bias each one
introduces, and draft the outreach message.

**When to use it:** once the screener exists.
**Returns:** per channel: expected yield, the bias it introduces, and a draft
message. Sending anything requires approval. If the only viable channel is your
own network, it says so and marks the round biased before it starts.

### `/guide [--goal <text>]`

Write the interview guide: opening, past-behaviour questions, probes, kill
questions designed to disprove your own hypothesis, closing and referral ask.

**When to use it:** before the first interview, and again whenever the guide
changes mid round.
**Returns:** the versioned guide. Any future-tense question is flagged and must
be justified as a deliberate exception or rewritten in past tense.

### `/interview [<participant-id>]`

Run a session against the current guide. Walks consent first, then holds the
interviewer to the guide while following surprises, and warns in real time when
the interviewer starts pitching.

**When to use it:** during the call, live, or immediately around it.
**Returns:** the live session record: consent status, recording status, coverage
of the guide, notes, and the moments worth returning to.

### `/debrief <participant-id>`

Capture the debrief while the session is still fresh: what surprised you, the
quotes to keep, what changes in the guide.

**When to use it:** within one hour of the interview. Not tomorrow.
**Returns:** the debrief record, and a proposed guide change if the session
produced one. If nothing surprised you, it says so and flags the session as a
probable pitch.

### `/transcribe <participant-id>`

Turn the recording or the notes into an attributed transcript, keeping the
participant's exact wording, hedges and contradictions intact.

**When to use it:** after the interview, before coding.
**Returns:** the transcript with speaker turns. A notes-only session produces a
notes-only record and is labelled as lacking verbatim evidence.

### `/code [<participant-id> | --all]`

Code transcripts against the versioned codebook, adding new codes with written
definitions where something genuinely does not fit.

**When to use it:** after each interview, not in one batch at the end. Coding as
you go is what makes saturation measurable.
**Returns:** the coded transcript, the codes added this pass with their
definitions, and the updated codebook version.

### `/saturation`

Report the code growth curve across the round and whether the stopping rule
fired.

**When to use it:** after every interview from the minimum N onward.
**Returns:** new codes per interview, consecutive interviews with zero new
codes, and a plain verdict: saturated, or not saturated and how far off. It
never rounds "it felt like enough" up to saturated.

### `/insight [--min-n <n>]`

Build insight records: the finding, the count of independent participants, their
ids, and a verbatim quote per participant counted.

**When to use it:** once coding is done and saturation has been checked.
**Returns:** confirmed insights with N and quotes, and separately the candidates
that fell under the floor, kept as named anecdotes with what N they would need.
Emits `discovery.insight.confirmed` for the confirmed set only.

### `/jtbd`

Extract jobs to be done from the confirmed evidence: circumstance, the progress
wanted, current solution, workaround, what is being fired.

**When to use it:** before handing anything to Blueprint {OS}.
**Returns:** one job statement per job, each with the evidence behind it and the
workarounds observed. Workarounds people built by hand are listed first: they
are the strongest signal in the round.

### `/segment`

Group participants into segments by behaviour and job, and profile each.

**When to use it:** when the round covered more than one kind of participant, or
when quotes contradict each other in a patterned way.
**Returns:** segment profiles, each with the behaviour that defines it, its
pains, its workarounds, its N and the evidence per claim. A group that differs
only demographically is reported as not a segment. Emits
`discovery.segment.profiled`.

### `/discovery-audit <artifact>`

Inspect research someone else is presenting as evidence: a deck, a report, a
set of transcripts, a persona.

**When to use it:** whenever you inherit research, or before a number from a
research deck enters a plan.
**Returns:** per claim, how many people it rests on, how they were recruited and
with what bias, whether the questions were past tense, whether quotes exist, and
the gap between what was measured and what is being asserted.

## Command summary

| Command | Does | Returns |
|---|---|---|
| `/discovery` | entry point: decision to round plan, or round status | a round plan, or where the current round stands |
| `/round` | plan one round: goal, decision, N, stopping rule, consent | the round plan, or a refusal if no decision depends on it |
| `/screener` | select participants on observable behaviour | screener questions, qualifying and disqualifying answers |
| `/recruit` | plan channels, label their bias, draft outreach | yield and bias per channel, draft message, approval gate |
| `/guide` | write the past-tense interview guide | versioned guide with probes and kill questions |
| `/interview` | run the session under consent and guide | live session record, coverage, pitch warnings |
| `/debrief` | capture what surprised you, within the hour | debrief record and proposed guide change |
| `/transcribe` | attributed transcript, wording intact | the transcript, or a labelled notes-only record |
| `/code` | code against the versioned codebook | coded transcript, new codes, codebook version |
| `/saturation` | measure the code growth curve | saturated or not, with the numbers behind it |
| `/insight` | confirm findings with counts and quotes | confirmed insights, plus the anecdotes below the floor |
| `/jtbd` | jobs, circumstances, workarounds | job statements with their evidence |
| `/segment` | behavioural segments and their profiles | segment profiles, or a statement that no real segment exists |
| `/discovery-audit` | check research someone else calls evidence | measured versus asserted, per claim |
