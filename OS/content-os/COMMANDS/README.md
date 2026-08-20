# Content {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. One row per command, no exceptions: a command that is not
documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install content-os` | Installs this OS into your environment | Once, first |
| `agentik configure content-os` | Collects the minimum context it needs | After install |
| `agentik run content-os` | Starts the OS | Every session |
| `agentik doctor content-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update content-os` | Updates to the latest version | When a release lands |
| `agentik eval content-os` | Runs its evaluation suite | Before trusting it |

## OS commands

Eighteen commands across nine modes. The mode column matches `system/ROUTER.md`
and `config/router.json` in the installed pack.

| Command | Mode | What it does | Input | Output |
|---|---|---|---|---|
| `/content` | strategy | Opens Content {OS} | none | the calendar, live blocks, what is awaiting approval |
| `/content-gps` | strategy | Defines positioning and the content system | claim, audience, capabilities | a content GPS with pillars and cadence |
| `/capture-day` | capture | Ingests the day as source material | notes, transcripts, artefacts | staged records, each with source and timestamp |
| `/story-mine` | mine | Finds stories, insights and proof | captured material | candidates with epistemic labels and story referrals |
| `/pillar` | create | Creates a pillar asset | a selected candidate | a pillar draft with labelled claims |
| `/article` | create | Creates a standalone article | a topic and its evidence | an article draft with labelled claims |
| `/cascade` | cascade | Builds a multi-platform waterfall | a pillar | one native package per selected surface |
| `/instagram` | platform | Creates an Instagram-native package | a pillar or candidate | an Instagram package |
| `/tiktok` | platform | Creates a TikTok-native package | a pillar or candidate | a TikTok package |
| `/youtube` | platform | Creates a YouTube package | a pillar or candidate | a YouTube package |
| `/linkedin` | platform | Creates a LinkedIn package | a pillar or candidate | a LinkedIn package |
| `/x` | platform | Creates an X package | a pillar or candidate | an X package |
| `/newsletter` | platform | Creates a newsletter edition | a pillar or candidate | a newsletter draft |
| `/visual-brief` | produce | Creates image and design direction | an asset | a visual brief with rights and accessibility |
| `/video-brief` | produce | Creates script, shots and edit plan | an asset | a video brief with rights and accessibility |
| `/sound-brief` | produce | Creates sound, music and voice direction | an asset | a sound brief with licence requirements |
| `/content-calendar` | calendar | Builds the editorial calendar | pillars, cadence, campaigns | a calendar where each slot states its job |
| `/content-review` | measure | Runs the performance council | published assets | per-asset verdicts against the job each was given |

---

## Strategy

### `/content`

The default view: the calendar, what is in production, what is blocked on
rights or consent, and what is waiting on a human approval.

```bash
/content
```

**When to use it:** at the start of any content session.
**Returns:** blocked releases first, then approvals pending, then the calendar.
A slot whose story object has no truth verdict is shown as empty, not as ready.

### `/content-gps`

Defines the content system: audience, pillars, cadence, and, explicitly, what
will not be covered.

```bash
/content-gps
```

**When to use it:** before the first calendar, and again when the claim or the
offer changes.
**Returns:** the content GPS. Each pillar traces to the Positioning {OS} claim
and to a real capability. A pillar that traces to neither is rejected.

---

## Capture and mining

### `/capture-day`

Ingests a day of real work as source material.

```bash
/capture-day
```

**When to use it:** daily, or after anything worth remembering happened.
**Returns:** staged records, each with a source and a timestamp. Low-confidence
extractions remain staged and are named as staged.

### `/story-mine`

Finds candidate stories, insights and proof in captured material.

```bash
/story-mine --since 2026-08-01
```

**When to use it:** before planning a cycle, and whenever the calendar is thin.
**Returns:** candidates with epistemic labels. Anything that is a story is
referred to Storyteller {OS} and comes back as a story object with a truth
verdict; Content does not deepen it here.

---

## Creation and cascade

### `/pillar`

Creates the substantial asset a cascade derives from.

```bash
/pillar "what we learned rebuilding the onboarding"
```

**Returns:** a draft with every material claim labelled E1 to E5, and the list
of claims dropped for lack of evidence.

### `/article`

Creates a standalone article that is not part of a cascade.

```bash
/article "how attribution windows distort affiliate reporting"
```

**Returns:** the same contract as `/pillar`.

### `/cascade`

Turns one pillar into a native waterfall across the selected surfaces.

```bash
/cascade --from pillar-114 --surfaces instagram,linkedin,newsletter
```

**When to use it:** once the pillar has passed QA and its rights are clear.
**Returns:** one package per surface. Each is an adaptation with its own hook,
structure and format. A package that is a word-count crop of the pillar is
rejected and reported as a crop.

### Platform commands

`/instagram` `/tiktok` `/youtube` `/linkedin` `/x` `/newsletter`

```bash
/instagram --from pillar-114
/newsletter --from pillar-114
```

**When to use them:** inside a cascade, or standalone when only one surface is
in play.
**Returns:** a package native to that surface's grammar, carrying its
accessibility requirements and any platform-specific disclosure.

---

## Production

### `/visual-brief`, `/video-brief`, `/sound-brief`

Production direction for an asset.

```bash
/visual-brief --asset ig-0421
/video-brief --asset yt-0088
/sound-brief --asset yt-0088
```

**When to use them:** after the asset's text is approved, before production
starts.
**Returns:** the brief, with the Brand {OS} visual system referenced rather than
restated, plus rights, licences, likeness clearances and accessibility
requirements. Any unresolved item emits `content.rights.blocked`.

---

## Calendar and measurement

### `/content-calendar`

Builds the editorial calendar from the pillars and the cadence.

```bash
/content-calendar --weeks 6
```

**When to use it:** at the start of a cycle.
**Returns:** the calendar. Every slot states its job in the audience journey.
A slot with no job is left unfilled rather than filled generically.

### `/content-review`

Runs the performance council over published assets.

```bash
/content-review --period 2026-07
```

**When to use it:** at the close of every cycle.
**Returns:** per-asset verdicts against the job each asset was given, the
assets that performed at a job they were not given (a finding, not a win), and
what would change the recommendation. Feeds `content.performance.feedback` to
Storyteller {OS} and performance to Growth {OS} and KPI & Analytics {OS}.

---

## Reference runtime

The pack ships a provider-neutral reference runtime (stdlib Python, no LLM, no
external API). It is not a production database, an LLM adapter or a security
layer.

```bash
python runtime/os_runtime.py info                 # name, version, slug, purpose
python runtime/os_runtime.py validate             # sha256 integrity against MANIFEST.json
python runtime/os_runtime.py route "/content"     # resolve a command to its mode
python runtime/os_runtime.py event <kind> <json>  # append a provenance event
```

**Returns:** `validate` reports per-file integrity, not a summary badge.

---

## Command summary

| Command | Does |
|---|---|
| `/content` | opens the OS: blocks, approvals, calendar |
| `/content-gps` | defines audience, pillars, cadence and exclusions |
| `/capture-day` | ingests the day as sourced, timestamped material |
| `/story-mine` | finds candidates, refers stories to Storyteller {OS} |
| `/pillar` | creates the pillar asset |
| `/article` | creates a standalone article |
| `/cascade` | native waterfall from one pillar |
| `/instagram` `/tiktok` `/youtube` `/linkedin` `/x` `/newsletter` | one native package per surface |
| `/visual-brief` `/video-brief` `/sound-brief` | production direction with rights and accessibility |
| `/content-calendar` | the calendar, every slot with a job |
| `/content-review` | the performance council verdict |
