# Alignment {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install alignment-os` | Installs this OS into your environment | Once, first |
| `agentik configure alignment-os` | Collects the minimum context it needs | After install |
| `agentik run alignment-os` | Starts the OS | Every session |
| `agentik doctor alignment-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update alignment-os` | Updates to the latest version | When a release lands |
| `agentik eval alignment-os` | Runs its evaluation suite | Before trusting it |

## OS commands

This unit is conversational. There is no state engine: `omega-align` opens a
session running the master agent, and every command below is a protocol or a
skill the agent executes from `pack/protocols/` or `pack/skills/`.

| Command | What it does | When to use it | Returns |
|---|---|---|---|
| `/alignment-os` | Opens the OS with the full operating contract loaded | Any alignment work in Claude | An open session, one question back |
| `/align` | Alias of `/alignment-os` | Same, shorter to type | Same |
| `/coach` | Alias of `/alignment-os` | When you want the coaching register by name | Same |
| `omega-align` | Opens the master agent in a terminal session, from the OS folder | Working outside a chat client | A session with the ledger path available |
| `/morning` | Runs the seven minute morning protocol | Start of day | State, virtue, outcome, obstacle, rehearsed response, first move |
| `/evening` | Runs the evening review | End of day | What held, what fell short, what was never yours, one adjustment |
| `/weekly` | Runs the weekly council across ten domains | Week close | Stop, start, continue, plus next week's governing principle |
| `/reset` | Runs the three minute reset | Acute overwhelm | Reality, agency, next honorable move |
| `/true_north` | Elicits or rebuilds the value set and the aim behind it | No value set, or the old one no longer fits | A written value set with priority order |
| `/virtue_check` | Scores a choice against wisdom, courage, justice, temperance | A choice that feels off but you cannot say why | The virtue conflicts, named |
| `/dichotomy_control` | Sorts a situation into control classes | Effort is going somewhere it may not land | Four buckets and one authored action |
| `/wu_wei` | Applies the right effort test | You are forcing something | Force, wait, or redirect, with the reason |
| `/decision` | Runs the values and control lens over a pending call | Before deciding, not instead of deciding | Criteria and constraints for Decision {OS} |
| `/reframe` | Re-describes an event stripped of story | After rejection or failure | The event, the story, and what is actually left |
| `/shadow` | Surfaces the motive you are not saying out loud | A repeated reaction you dislike in yourself | The disowned motive, stated plainly |
| `/belief_audit` | Tests one belief for precision under challenge | A belief that keeps producing the same result | The belief, sharpened or falsified |
| `/fear` | Separates the feared event from the feared meaning | Fear before an action you intend to take | The real exposure and the rehearsed response |
| `/meaning` | Connects the current situation to a chosen aim | The work is going fine and feels empty | The aim, restated, or the honest absence of one |
| `/personal_philosophy` | Traces results back to activities, attitudes and beliefs | Recurring results contradict stated values | Exactly one philosophy rule proposed for update |
| `/manifestation` | Runs intention, visualization and action as preparation | You want to work on a desired outcome | A rehearsal plus a concrete action, labelled E4 where it is metaphysical |
| `/quantum_truth` | Corrects quantum claims with accurate guardrails | A physics claim is being used as a life rule | What the physics says, what it does not say |
| `/anti_dependency` | Transfers agency back to you | You have asked for reassurance on the same point repeatedly | The principle, and the choice handed back |

### `/alignment-os`, `/align`, `/coach`

Three names for one entry point. They load `pack/system/SYSTEM_PROMPT.md`,
`PRINCIPLES.md`, `ROUTER.md` and `config/os.yaml`, then route your request
across the twelve council voices and answer with one integrated response. They
never open with a menu. `/omg-alignment-os` is the OmegaOS-namespaced alias.

```
/align I said family comes first and I worked both weekend days again
```

### `omega-align`

The terminal entry point. It opens a Claude session with the master persona
appended, working from `~/.omega/os/alignment-os/` so the ledger under that
folder persists between sessions. There is no subcommand: this OS has no state
CLI, and the wrapper says so rather than pretending to.

```bash
omega-align
```

### `/morning`

Eight steps in about seven minutes: physical state in one line, one specific
thing that is actually true and good, the identity you are practising today, one
virtue, one meaningful outcome, the obstacle you expect, an if-then response
rehearsed in advance, and the first move. Run it before the first work block,
not after.

### `/evening`

The counterpart. What you did well, where you acted below your own standard,
what was never yours to control, where you forced, where you avoided necessary
effort, what reality taught you, what you release tonight, and the single
adjustment for tomorrow. It produces one adjustment, not a list.

### `/weekly`

The weekly council: wins, failures, energy, habits, relationships, work, money,
meaning, learning, then stop, start and continue, then one governing principle
for the coming week. This is the command that feeds `weekly-values-audit.md`.

### `/reset`

Three minutes, three questions, one per minute: what happened stripped of story,
what is mine right now, what is the next honorable useful move. Use it when you
would otherwise lose the afternoon.

### `/true_north`

The protocol that creates the object this OS owns. It runs truth, regulation,
ultimate aim, ethics, the control map, options, right effort, a tiny action and
the harvest. Nothing else in this unit works without it: an audit with no
declared value set has nothing to measure against.

```
/true_north I am rebuilding this after the move, the old list is two years stale
```

### `/virtue_check`

Evaluates a choice through wisdom, courage, justice and temperance, and surfaces
the conflicts between them rather than averaging them. A choice that is
courageous and unjust is reported as exactly that.

### `/dichotomy_control`

Sorts every element of a situation into choose, influence, cannot control, and
unknown, then moves attention to an action you actually author. Unknown is a
real bucket: pretending something is controllable is the failure this prevents.

### `/wu_wei`

The right effort test. Not passivity: it asks whether the resistance you are
meeting is the wrong door, the wrong timing, or the correct cost of the right
thing, and returns force, wait or redirect with the reason.

### `/decision`

A values and control lens run over a pending call. It produces the control map
for that call, the virtue conflicts, the opportunity cost in values terms, the
right effort verdict, and the 10 day, 10 month, 10 year perspective. It does not
produce the choice. The options, the reversibility class, the decision record
and the review belong to Decision {OS} (`decision-os`): take these criteria
there and run `/decide`.

```
/decision leave the contract in March or renew for another year
```

### `/reframe`

Separates the event from the narrative wrapped around it, then asks what remains
true. Used after rejection and failure, and deliberately not used to make a real
loss sound acceptable.

### `/shadow`

Surfaces the motive you are not saying out loud, in your own words, without
diagnosis. It names a disowned motive as a hypothesis with a confidence, never
as a fact about you.

### `/belief_audit`

Takes one belief and tries to make it more precise under challenge. A belief
that survives comes back sharper; a belief that does not comes back falsified,
with the evidence that killed it. Beliefs themselves are owned by Mindset {OS};
this command hands its output there.

### `/fear`

Splits the feared event from the meaning attached to it, sizes the real
exposure, and rehearses the response. It ends in the action, not in comfort.

### `/meaning`

Connects the present situation to a chosen aim, and reports honestly when there
is no aim it can connect to rather than manufacturing one.

### `/personal_philosophy`

Maps recurring results back through activities, attitudes and beliefs, and
proposes exactly one rule to update. One, because a philosophy audit that
proposes five rules changes nothing.

### `/manifestation`

Runs intention, visualization and action as preparation and rehearsal. Every
metaphysical claim in the output is labelled E4 and is never stated as
established science. The command always ends in a concrete action.

### `/quantum_truth`

Corrects the physics when a quantum claim is being used as a life rule. It
states what the experiment shows (E1) and what it does not license (E4),
separately.

### `/anti_dependency`

The agency transfer. When you have asked for reassurance on the same point
repeatedly, this stops the supply of new reasons, restates the principle you
already have, and asks you to choose. It is the guardrail that keeps this OS
from becoming the thing you need before acting.

## Command summary

| Command | Does |
|---|---|
| `/alignment-os`, `/align`, `/coach` | opens the OS, routes across the council |
| `omega-align` | opens the master agent in a terminal session |
| `/morning` | seven minute morning pass, ends in a first move |
| `/evening` | evening review, ends in one adjustment |
| `/weekly` | ten domain weekly council, ends in a governing principle |
| `/reset` | three minute reality, agency, action |
| `/true_north` | builds the value set and the aim |
| `/virtue_check` | scores a choice on four virtues, names the conflict |
| `/dichotomy_control` | sorts a situation into four control classes |
| `/wu_wei` | force, wait or redirect |
| `/decision` | values lens over a call, criteria for Decision {OS} |
| `/reframe` | event minus story |
| `/shadow` | the motive not said out loud |
| `/belief_audit` | one belief, sharpened or falsified |
| `/fear` | real exposure plus rehearsed response |
| `/meaning` | the aim, or its honest absence |
| `/personal_philosophy` | one philosophy rule to update |
| `/manifestation` | rehearsal plus action, labelled E4 |
| `/quantum_truth` | what the physics does and does not license |
| `/anti_dependency` | hands the choice back to you |
