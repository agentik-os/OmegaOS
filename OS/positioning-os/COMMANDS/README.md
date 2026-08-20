# Positioning {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install positioning-os` | Installs this OS into your environment | Once, first |
| `agentik configure positioning-os` | Collects the minimum context it needs | After install |
| `agentik run positioning-os` | Starts the OS | Every session |
| `agentik doctor positioning-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update positioning-os` | Updates to the latest version | When a release lands |
| `agentik eval positioning-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| `/positioning` | Orients: reads the ledger and names the next decision | nothing | current position, live claims, the next action |
| `/position-map` | Builds the competitive set with each rival's quoted claim | competitor names or a market | the competitive map with sources and capture dates |
| `/position-language` | Extracts the customer's own vocabulary for the problem | discovery transcripts or notes | ranked verbatim terms with frequency and source |
| `/position-category` | Decides the category, or declares an invented one | the competitive map, demand evidence | the category decision with its demand verdict |
| `/position-claim` | Drafts a candidate claim and forces its exclusion | the category, customer language | a candidate claim, its exclusion, its proof requirement |
| `/position-test` | Tests a candidate against customer language and rival claims | a candidate claim | pass or fail per test, with the failing comparison named |
| `/position-statement` | Assembles the positioning statement | a tested claim | category, target, claim, exclusion, proof in one paragraph |
| `/position-ledger` | Reads, writes and audits the claim ledger | a claim, or nothing to read | ledger entries with evidence, expiry, tester, date |
| `/position-conflict` | Detects and reports claims that contradict each other | the ledger | contested pairs with the evidence on each side |
| `/position-review` | Retests claims that reached their expiry window | the ledger | per claim: holds, contested or expired |
| `/position-retire` | Retires a claim and notifies its consumers | a claim id and the killing evidence | a retirement record and the downstream notice list |
| `/position-brief` | Packages the position for a downstream unit | a target OS slug | the statement and claim subset that unit needs |

### `/positioning`

```
/positioning
```

Reads the claim ledger and tells you where the position actually stands: which
claims are live, which are contested, which are past their expiry, and what
decision is waiting on you. It never opens by proposing a slogan.

**When to reach for it:** at the start of any positioning session, and before
any downstream unit acts on a claim.
**Returns:** the current statement, the ledger state by class, and one named
next action.

### `/position-map`

```
/position-map
/position-map "project management for agencies"
```

Builds the competitive set from how buyers compare, then pulls each rival's
live claim verbatim with a source and a capture date. A rival whose claim
cannot be verified at source is dropped from the map rather than paraphrased.

**When to reach for it:** first, before any claim work, and again whenever a
new entrant shows up in a sales call.
**Returns:** the map, each rival with claim, exclusion, source and date, plus
the ground nobody is currently contesting.

### `/position-language`

```
/position-language --from discovery
```

Extracts the words customers actually use for the problem, ranked by frequency,
each traced to the utterance it came from. This is the corpus every later test
runs against.

**When to reach for it:** after any batch of customer interviews, and before
drafting a claim.
**Returns:** ranked verbatim terms with counts and sources, and the terms the
operator uses that no customer used.

### `/position-category`

```
/position-category
/position-category --invent "agentic operations"
```

Decides which category you are competing in. If demand evidence exists, it says
so and names the evidence. If it does not, the category is recorded as invented
and the cost of creating demand is stated plainly rather than assumed away.

**When to reach for it:** once the map exists, and whenever a category change
is being considered.
**Returns:** the category decision, its demand verdict, and what changes
downstream if the category moves.

### `/position-claim`

```
/position-claim
```

Drafts a candidate claim in the customer's vocabulary and forces the exclusion:
the sentence naming what you are deliberately worse at. If that sentence comes
back empty, the claim is refused, because a claim that excludes nothing is not
a position.

**When to reach for it:** after the category is decided.
**Returns:** the candidate claim, its exclusion, and the proof the claim will
have to carry into `/position-test`.

### `/position-test`

```
/position-test --claim c-014
```

Runs the candidate against two adversaries: the recorded customer language
(does anybody talk like this) and the published rival claims (could a
competitor say this without lying). Both must pass.

**When to reach for it:** before a claim is allowed into the ledger, and again
at every review.
**Returns:** pass or fail per test, and on failure the exact utterance or rival
claim that killed it.

### `/position-statement`

```
/position-statement
```

Assembles the one paragraph every downstream unit reads: category, target,
claim, exclusion, proof. It only assembles claims that passed `/position-test`.

**When to reach for it:** once a claim is tested, and before any handoff.
**Returns:** the statement, plus the list of units that will be notified it
changed.

### `/position-ledger`

```
/position-ledger
/position-ledger --write --claim c-014 --expiry "a rival ships same-day onboarding"
```

The canonical artifact of this OS. Every live claim with the evidence that
supports it, its expiry condition, and who last tested it. Writes are refused
when any of those four fields is empty.

**When to reach for it:** every time a claim is created, tested, contested or
retired.
**Returns:** the ledger, or the written entry with its next review date.

### `/position-conflict`

```
/position-conflict
```

Scans the ledger for claims that cannot both be true, marks the pair contested,
and puts the evidence for each side next to the other. It does not choose.

**When to reach for it:** after any ledger write, and before a launch.
**Returns:** contested pairs, the evidence on each side, and the decision the
human has to make.

### `/position-review`

```
/position-review
/position-review --due
```

Retests every claim that reached its expiry window. A claim that nobody
retested is marked expired, not assumed still true.

**When to reach for it:** on the cadence the ledger set, and before any
campaign that will repeat old claims.
**Returns:** per claim, holds, contested or expired, with the retest evidence.

### `/position-retire`

```
/position-retire --claim c-009 --because "the benchmark was rerun and we lost"
```

Retires a claim, records what killed it, and lists every downstream surface and
unit still using it so the retirement actually lands.

**When to reach for it:** the moment evidence stops supporting a live claim.
**Returns:** the retirement record and the notification list, which a human
must approve before it goes out.

### `/position-brief`

```
/position-brief brand-os
/position-brief sales-os
```

Packages the statement and the claim subset a specific downstream unit needs,
in the shape that unit expects. Brand gets the claim and the exclusion, Sales
gets live and contested status, Storyteller gets the evidence class.

**When to reach for it:** at every handoff, and after every statement change.
**Returns:** the brief for that unit, and what that unit is expected to do
with it.

## Command summary

| Command | Does |
|---|---|
| `/positioning` | where the position stands, and the next decision |
| `/position-map` | the competitive set, rivals quoted from source |
| `/position-language` | the customer's own words, ranked and sourced |
| `/position-category` | decide the category, or declare it invented |
| `/position-claim` | draft a claim, force its exclusion |
| `/position-test` | test the claim against customers and rivals |
| `/position-statement` | assemble the one paragraph everyone reads |
| `/position-ledger` | the claim ledger: evidence, expiry, tester, date |
| `/position-conflict` | find claims that contradict, escalate, never pick |
| `/position-review` | retest on expiry, or mark expired |
| `/position-retire` | retire a claim and notify its consumers |
| `/position-brief` | package the position for one downstream unit |
