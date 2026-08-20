# Storyteller {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

These are modes the SKILL.md command router reaches, not separately registered
slash commands. Natural language always works, and the user is never forced to
learn a command. Each command's completion contract lives in
`references/commands.md`.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install storyteller-os` | Installs this OS into your environment | Once, first |
| `agentik configure storyteller-os` | Collects the minimum context it needs | After install |
| `agentik run storyteller-os` | Starts the OS | Every session |
| `agentik doctor storyteller-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update storyteller-os` | Updates to the latest version | When a release lands |
| `agentik eval storyteller-os` | Runs its evaluation suite | Before trusting it |

## OS commands

| Command | What it does | Input | Output |
|---|---|---|---|
| `/story` | Orients: reads intent, contract, story class and routes | a request, or nothing | the routing decision and the next question or action |
| `/story-setup` | Establishes the operator's context, register and boundaries | operator context | the working defaults for voice, privacy and stakes |
| `/mine` | Mines raw material for the change under pressure | notes, transcripts, a day, a memory | candidate moments with their tension, cost and meaning |
| `/interview` | Interviews without contaminating memory | a chosen subject or moment | recalled material in the user's words, one question at a time |
| `/moment` | Captures one moment as a story object seed | a single moment | a moment card with what changed and what it cost |
| `/deepen` | Reveals desire, obstacle, stakes, belief, choice, consequence | a thin story object | the deepened object with its unresolved edge named |
| `/shape` | Selects the smallest useful architecture and scene order | deepened material, audience, channel | the structure with the rationale for choosing it |
| `/hook` | Builds openings that earn the next sentence | a shaped story | hook options with what each one promises |
| `/scene` | Builds a scene from observable detail only | a beat of the story | the scene, with every invented detail refused |
| `/arc` | Maps the arc across a series or a body of work | several story objects | the arc, its throughline and its gaps |
| `/cowrite` | Co-creates: structures, beats, questions, neutral fragments | an authorised CO-CREATE request | fragments and options, never a finished story |
| `/write` | Drafts the deliverable, only under an explicit WRITE contract | an explicit request to write | the draft in the user's register |
| `/rewrite` | Rewrites an existing draft under an authorised contract | a draft and the change wanted | the rewrite, with what was preserved stated |
| `/voice` | Protects and diagnoses voice fidelity | a draft, plus samples of the user's own writing | a voice report and the lines that drifted |
| `/adapt` | Moves a story to another channel preserving story DNA | a shaped story and a target channel | the channel version and what was preserved |
| `/content` | Produces the content shaped version of a story | a story object and a content brief | the content piece, handed to Content {OS} for publishing |
| `/keynote` | Shapes a story for a stage | a story object, the room, the time | the talk structure with timing and beats |
| `/pitch` | Shapes a story for an investor or partner pitch | a story object and the ask | the pitch narrative with the ask in its place |
| `/brandstory` | Shapes the origin and purpose story of the company | founding material, the claim | the brand story, consistent with the live claim |
| `/customerstory` | Shapes a customer story from verified outcome evidence | outcome evidence, consent record | the customer story, or a refusal if consent is unresolved |
| `/datastory` | Connects a data pattern to human consequence | a dataset or a finding | the data story with every number traced to its source |
| `/truthcheck` | Separates fact, memory, inference, interpretation, invention | a story object or a draft | per element truth classes and the load bearing risks |
| `/score` | Scores structural completeness, never truth or virality | a story object | the score with what is missing, explicitly not a truth signal |
| `/rehearse` | Tests delivery: clarity, movement, timing, breath, ending | a shaped story or a talk | the rehearsal notes and the beats that lost the room |
| `/feedback` | Reads audience response against intent | performance data or reactions | what the audience actually received versus what was intended |
| `/storybank` | Operates the durable bank of story objects | a query, or a story object | bank records with claims, consent and versions |
| `/repurpose` | Finds the stories worth telling again, and where | the bank and a channel need | ranked candidates with the adaptation each needs |
| `/story-review` | Issues the release verdict | a story proposed for release | READY, READY WITH CUTS, NEEDS TRUTH CHECK, NEEDS DEEPENING, WRONG STORY FOR THIS JOB, or DO NOT PUBLISH |

### `/story`

```
/story
/story "I want to talk about the year we nearly closed"
```

The entry point. Identifies the intended outcome, selects the agency contract,
names the story class and truth class, and routes. It states the active
contract in one line whenever ambiguity could cause overreach.

**When to reach for it:** whenever you do not know which mode you need.
**Returns:** the routing decision and one decisive question or action.

### `/story-setup`

```
/story-setup
```

Establishes the operator's working context: register, boundaries, what is
private, what may never be told, and which people are off limits by default.

**When to reach for it:** once, at the start, and again when your situation or
your boundaries change.
**Returns:** the defaults every later mode inherits.

### `/mine`

```
/mine ./notes-this-week.md
/mine "the last six months"
```

Mines raw material for the specific change under meaningful pressure. It looks
for change, tension, surprise, cost, contradiction, choice, proof and meaning,
and it discards chronology that carries none of them.

**When to reach for it:** when you know something happened but not what the
story is.
**Returns:** candidate moments, each with its change, its tension and its cost.

### `/interview`

```
/interview --about "the day the contract fell through"
```

Interviews you without contaminating recall: one open question at a time, open
recall before interpretation, observable prompts, and any hypothesis offered as
a hypothesis rather than as a memory.

**When to reach for it:** when the material only exists in your head.
**Returns:** the recalled material in your own words, with exact wording marked
as a quote only where you supplied or confirmed it.

### `/moment`

```
/moment "the phone call in the car park"
```

Captures a single moment as the seed of a story object: what happened, what
changed, what it cost, and what is still unresolved.

**When to reach for it:** immediately after something happens, before the
memory smooths itself.
**Returns:** a moment card ready for `/deepen`, and a bank record if you ask
for one.

### `/deepen`

```
/deepen --story s-042
```

Takes a thin story and reveals desire, obstacle, stakes, identity, belief,
choice, consequence and the unresolved edge. It refuses to resolve the edge for
you, because the resolved version is the one nobody remembers.

**When to reach for it:** when a story is true and complete and still lands
flat.
**Returns:** the deepened object with the unresolved edge named explicitly.

### `/shape`

```
/shape --story s-042 --audience "operators" --channel keynote
```

Selects the smallest useful narrative architecture and the scene order, after
the material, meaning, audience and channel are known. It does not force a
hero's journey onto material that does not fit one.

**When to reach for it:** once the material is deep enough to hold a shape.
**Returns:** the structure, the scene order, and why this architecture rather
than another.

### `/hook`

```
/hook --story s-042
```

Builds openings that earn the next sentence, each stating what it promises the
audience, because a hook that promises something the story does not deliver
costs more than a dull opening.

**When to reach for it:** after `/shape`, never before.
**Returns:** hook options with the promise each one makes.

### `/scene`

```
/scene --beat "the call"
```

Builds a scene from observable detail that you supplied or confirmed. Sensory
detail you did not report is not added, because invented detail is the most
common way a true story quietly becomes false.

**When to reach for it:** when a beat needs to be seen rather than summarised.
**Returns:** the scene, and a list of the details it declined to invent.

### `/arc`

```
/arc --bank --theme "building alone"
```

Maps the arc across several stories: the throughline, the order, the repetition
that is working, and the gap where a story is missing.

**When to reach for it:** for a talk, a book, a series, or a year of content.
**Returns:** the arc with its gaps named.

### `/cowrite`

```
/cowrite --story s-042
```

The CO-CREATE contract. Offers structures, beat options, questions and short
neutral fragments. It does not complete the story, and it does not write in
your voice.

**When to reach for it:** when you want to write it yourself but are stuck on
the architecture.
**Returns:** options and fragments, with the next decision left to you.

### `/write`

```
/write --story s-042 --as linkedin-post
```

The WRITE contract. Runs only after you explicitly ask for a draft, a script, a
post or the deliverable. A direct request for the artifact is that
authorisation; anything less is not.

**When to reach for it:** when you want the draft produced.
**Returns:** the draft in your register, with every element whose truth class
is not documented marked as such.

### `/rewrite`

```
/rewrite --draft ./post.md --change "cut the middle, keep the ending"
```

Rewrites an existing draft under the EDIT contract, preserving your language
and intent unless you authorise a stronger transformation.

**When to reach for it:** when the draft exists and the shape is wrong.
**Returns:** the rewrite plus an explicit list of what was preserved and what
was changed.

### `/voice`

```
/voice --draft ./post.md --samples ./my-writing/
```

Diagnoses voice fidelity against your own writing and against the Brand {OS}
voice rules. Manufactured raw authenticity is reported as a failure, not as a
style.

**When to reach for it:** on any draft that will go out under your name.
**Returns:** the voice report and the exact lines that drifted.

### `/adapt`

```
/adapt --story s-042 --to reel
```

Moves a shaped story into another channel while preserving story DNA: the
change, the pressure, the cost and the meaning. It never simply crops to a word
count.

**When to reach for it:** when one story has to live on several surfaces.
**Returns:** the channel version and a statement of what was preserved and what
was deliberately lost.

### `/content`

```
/content --story s-042 --format carousel
```

Produces the content shaped version of a story and hands it to Content {OS} for
packaging and publishing. Storyteller does not decide when or where it runs.

**When to reach for it:** when a verified story is ready to become a piece.
**Returns:** the piece, plus the story object reference Content {OS} will
publish against.

### `/keynote`

```
/keynote --story s-042 --minutes 18
```

Shapes a story for a stage: beats, timing, breath, the visual proof and the
ending. Structure follows the room and the clock.

**When to reach for it:** for a talk, a panel or an internal all hands.
**Returns:** the talk structure with timings and the beats most at risk.

### `/pitch`

```
/pitch --story s-042 --ask "seed round"
```

Shapes a story for a pitch, with the ask placed where it belongs rather than
bolted on at the end.

**When to reach for it:** for investors, partners or a large deal.
**Returns:** the pitch narrative, the ask, and the claims it leans on with
their ledger status.

### `/brandstory`

```
/brandstory
```

Shapes the origin and purpose story of the company, consistent with the live
claim from Positioning {OS} and the voice rules from Brand {OS}.

**When to reach for it:** for an about page, a founding story, a first company
narrative.
**Returns:** the brand story, and any conflict it found with the live claim,
reported rather than resolved here.

### `/customerstory`

```
/customerstory --customer acme
```

Shapes a customer story from verified outcome evidence supplied by Delivery &
Customer Success {OS}, with the consent record attached. Without consent, it
returns a refusal, not a draft.

**When to reach for it:** for case studies, proof stories and references.
**Returns:** the customer story with its evidence and consent scope, or DO NOT
PUBLISH with the missing piece named.

### `/datastory`

```
/datastory --finding ./churn-analysis.md
```

Connects a data pattern to a human consequence. Every number is traced to its
source, and a pattern with no traceable source is dropped rather than rounded.

**When to reach for it:** when a chart needs to mean something to a person.
**Returns:** the data story with each number's source beside it.

### `/truthcheck`

```
/truthcheck --story s-042
```

Separates known fact, remembered detail, inference, interpretation and
invention, and labels composites, hypotheticals and reconstructed dialogue as
exactly what they are. This is a gate, not a score.

**When to reach for it:** before any release, and any time a detail feels too
convenient.
**Returns:** the truth class per element, the load bearing risks, and a hold at
VERIFY when a load bearing fact is uncertain.

### `/score`

```
/score --story s-042
```

Scores structural completeness only. It never predicts virality and never
proves truth, and a story never passes release on score alone.

**When to reach for it:** to find what is structurally missing.
**Returns:** the score with the missing elements listed and an explicit
statement of what the score does not measure.

### `/rehearse`

```
/rehearse --story s-042 --out-loud
```

Tests delivery: clarity, emotional movement, timing, breath, pause, visual
proof and the ending.

**When to reach for it:** before you say it in front of anyone.
**Returns:** the rehearsal notes and the beats where the room is lost.

### `/feedback`

```
/feedback --story s-042 --response ./comments.md
```

Compares what the audience received with what you intended, and updates the
story object. It updates the story, not the publishing plan.

**When to reach for it:** after a story has been told to a real audience.
**Returns:** the gap between intent and reception, and the story object update
it justifies.

### `/storybank`

```
/storybank
/storybank --search "first customer"
```

Operates the durable bank of story objects with their claims, consent records
and versions. It reports only what the CLI actually wrote.

**When to reach for it:** whenever you need a story you already have.
**Returns:** the matching records with truth classes, consent scope and version
history.

### `/repurpose`

```
/repurpose --for newsletter
```

Finds stories in the bank worth telling again, ranked by fit for the channel
and by how long since they were last told, each with the adaptation required.

**When to reach for it:** when a channel needs material and inventing something
new would be worse than telling a true thing again.
**Returns:** ranked candidates, each with its required adaptation and its
current consent scope.

### `/story-review`

```
/story-review --story s-042 --surface linkedin
```

Issues the release verdict for a specific story on a specific surface. VERIFY
and consent are gates here: a structurally excellent story with an unresolved
fact or an unresolved consent does not pass.

**When to reach for it:** last, before anything is handed on or published.
**Returns:** READY, READY WITH CUTS, NEEDS TRUTH CHECK, NEEDS DEEPENING, WRONG
STORY FOR THIS JOB, or DO NOT PUBLISH, with the reason.

## Story bank CLI

The `omega-story` CLI is deterministic: standard library Python and SQLite, no
network, no model. It owns the durable story objects and their evidence, and it
never claims a save, a permission or a version that did not actually happen.
Default bank: `~/.omega/os/storytelling-os/ledger/story-bank.db`, overridable
with `--db`.

| Command | What it does |
|---|---|
| `omega-story init` | Creates a bank |
| `omega-story capture --title ... --raw-file ... --story-class ...` | Creates a story object from raw material |
| `omega-story list` | Lists story objects |
| `omega-story show <id>` | Shows one story object in full |
| `omega-story update <id> --set path=value` | Edits a field of a story object |
| `omega-story add-claim <id> ...` | Adds a claim with its truth class to the claim ledger |
| `omega-story add-consent <id> ...` | Adds a consent record with its scope |
| `omega-story validate <id>` | Checks structural completeness of one object |
| `omega-story score <id>` | Scores structural completeness only |
| `omega-story doctor` | Reports bank health |
| `omega-story export --format jsonl\|json\|markdown --output ...` | Exports the bank portably |

```bash
omega-story init
omega-story capture --title "the call in the car park" --raw-file ./moment.md --story-class lived-moment
omega-story add-claim s-042 --text "we lost the contract in March" --truth-class documented
omega-story add-consent s-042 --person "Acme" --scope "public case study" --expires 2027-01-01
omega-story score s-042
omega-story export --format markdown --output ./bank.md
```

The CLI score checks structural completeness, never literary quality, audience
response or truth. A story never passes release on the CLI score alone.

## Command summary

| Command | Does |
|---|---|
| `/story` | orient and route |
| `/story-setup` | establish context, register and boundaries |
| `/mine` | find the change under pressure in raw material |
| `/interview` | recall without contaminating memory |
| `/moment` | capture one moment as a story seed |
| `/deepen` | desire, obstacle, stakes, choice, unresolved edge |
| `/shape` | select the architecture and the scene order |
| `/hook` | openings that earn the next sentence |
| `/scene` | build a scene from observable detail only |
| `/arc` | the throughline across many stories |
| `/cowrite` | structures and fragments, never the finished story |
| `/write` | draft, under an explicit WRITE contract |
| `/rewrite` | rewrite while preserving language and intent |
| `/voice` | diagnose voice fidelity and drift |
| `/adapt` | change channel, preserve story DNA |
| `/content` | the content shaped version, handed to Content {OS} |
| `/keynote` | shape it for a stage, with timing |
| `/pitch` | shape it for a pitch, with the ask in place |
| `/brandstory` | the origin and purpose story |
| `/customerstory` | a customer story with evidence and consent |
| `/datastory` | a data pattern with human consequence |
| `/truthcheck` | fact, memory, inference, interpretation, invention |
| `/score` | structural completeness, never truth or virality |
| `/rehearse` | test delivery out loud |
| `/feedback` | reception against intent |
| `/storybank` | operate the durable story bank |
| `/repurpose` | what is worth telling again, and where |
| `/story-review` | the release verdict |
