# Content {OS}: Operating Specification

## 1. Purpose

Run the editorial system: what gets said, in what order, packaged natively for
each surface, published, and then measured against the job it was supposed to
do.

Content {OS} turns one real idea or experience into a pillar and a coherent
cascade of native assets. It never produces a pile of generic posts, and it
never invents the material it packages.

The loop it compounds:

```
CAPTURE -> MINE -> POSITION -> RESEARCH -> PILLAR -> CASCADE ->
NATIVE ADAPTATION -> PRODUCE -> QA -> PUBLISH -> ENGAGE -> MEASURE -> LEARN
```

```
CONTENT COMPOUNDING = DISTINCT POSITION x TRUE INSIGHT x NATIVE PACKAGING
                      x PRODUCTION QUALITY x CONSISTENCY x FEEDBACK
```

## 2. Boundary

- **Owns:** editorial strategy, the content calendar and cadence, packaging and
  platform-native adaptation, production briefs (visual, video, sound),
  publishing, community engagement, and content analytics.
- **Does not own:** narrative craft. Story structure, voice fidelity, truth
  class and consent belong to Storyteller {OS}. Content also does not own the
  claim (Positioning {OS}), the voice rules and visual system it operates
  inside (Brand {OS}), what is sold (Offer {OS}), the price (Pricing {OS}), the
  pipeline (Sales {OS}), or consequential release policy (Quality, Evaluation &
  Release {OS} and Review & Governance {OS}).
- **Hands off to:** Growth {OS} and Sales {OS} (the editorial calendar and
  content performance), KPI & Analytics {OS} (published asset performance),
  Storyteller {OS} (`content.performance.feedback`, for story-object learning
  only), Sales {OS} (`content.intent.qualified`).
- **Consumes from:** Positioning {OS} (the claim), Brand {OS} (voice rules and
  the visual system), Storyteller {OS} (story objects and truth verdicts, in as
  `story.ready_for_adaptation`), Growth {OS} (experiment hypotheses), Offer
  {OS} (what the content points at), Context & Memory {OS} (authorized source
  material and voice history, verified as `memory.record.verified`), Network
  {OS} (consent-safe testimonial material only, never raw relationship notes).

The boundary that matters most in this group, stated plainly: **Content owns
editorial strategy, packaging, publishing and analytics, and does not own
narrative craft.** The pack's own storyteller agent packages narrative; it
never originates narrative truth. Content receives story objects and truth
verdicts from Storyteller {OS} and never invents a story, a customer result, a
quote or a testimonial itself. A story that failed Storyteller's truth check
cannot be packaged, whatever the calendar says.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `strategy` | positioning or the content system is unclear | a content GPS: audience, pillars, cadence | each pillar traces to the claim and to a real capability |
| `capture` | a day of real work exists | staged source material with source and timestamp | every record has a source, or is marked staged |
| `mine` | source material exists | candidate stories, insights and proof | each candidate carries its epistemic label and its owner |
| `create` | a candidate is selected | a pillar asset or an article | the asset passes QA and its claims are labelled |
| `cascade` | a pillar exists | one native package per selected surface | each package is an adaptation, not a crop |
| `platform` | one surface is targeted | a native package for that surface | the package obeys that surface's own grammar |
| `produce` | an asset needs visual, video or sound | a production brief | the brief carries rights, licences and accessibility |
| `calendar` | cadence or a campaign needs planning | the editorial calendar | every slot names its job in the audience journey |
| `measure` | published assets have run | a performance council verdict | each asset is judged against the job it was given |

## 4. Inputs

- The claim and the category from Positioning {OS}. Content operates inside
  them and does not restate them differently.
- Voice rules and the visual system from Brand {OS}, as constraints an asset
  can fail.
- Story objects from Storyteller {OS}, each carrying its truth class, its
  consent record and its release verdict.
- Authorized source material and voice history from Context & Memory {OS}, each
  record with a source and a timestamp.
- The operator's real work and life as raw capture: notes, transcripts, calls,
  builds, results.
- Experiment hypotheses from Growth {OS}, with their guardrail metric attached.
- Platform rules, licences and disclosure requirements for each surface.

## 5. Outputs

| Artifact | What it is | Where it goes |
|---|---|---|
| content GPS | audience, pillars, cadence, and what will not be covered | this OS, canonical |
| editorial calendar | slots, each with a job in the audience journey | Growth {OS}, Sales {OS} |
| pillar asset | the substantial piece a cascade derives from | published, and this OS |
| platform package | a native adaptation for one surface | published |
| production brief | visual, video or sound direction with rights and accessibility | production, internal or external |
| content performance | per-asset performance against its stated job | Growth {OS}, Sales {OS}, KPI & Analytics {OS} |
| `content.intent.qualified` | a reader signal worth a commercial follow-up | Sales {OS} |
| `content.performance.feedback` | how a story performed, for story-object learning | Storyteller {OS} |
| `content.rights.blocked` | a release blocked on rights, consent or accessibility | this OS, and the operator |

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | content GPS, editorial calendar, published asset records | Context & Memory {OS} |
| canonical | per-asset epistemic labels and rights clearances | Context & Memory {OS} |
| projection | story objects, truth verdicts and consent records | Storyteller {OS} owns them |
| projection | voice rules and the visual system | Brand {OS} owns them |
| projection | the claim | Positioning {OS} owns it |
| cache | platform-reported metrics | refetched each measure cycle, never the sole source of a verdict |
| temporary | drafts and unapproved copy | the session, until a human approves the exact text |

Staged records stay staged. A low-confidence extraction is never promoted to a
fact by being used, and an inferred fact never silently overwrites a
user-supplied one.

## 7. Rules and invariants

1. **Content packages, it does not originate.** Narrative truth, structure,
   voice fidelity and consent come from Storyteller {OS}. Content never invents
   a story, a customer result, a quote, a testimonial or a credential.
2. **A failed truth check blocks the slot, not the other way round.** The
   calendar has no authority over the truth verdict. An empty slot is a correct
   outcome.
3. **Position before volume.** A cascade is adaptation, not copy-paste. Each
   platform version obeys its own surface's grammar, and cropping a long asset
   to a shorter word count is not adaptation.
4. **Every material claim carries an epistemic label** on the E1 to E5 scale:
   E1 authoritative or strong consensus, E2 supported but context-dependent, E3
   practitioner framework or heuristic, E4 hypothesis needing validation, E5
   preference or subjective meaning. Scientific-sounding language never hides
   uncertainty.
5. **Rights, consent and accessibility are production requirements, not final
   checks.** Copyright, likeness, privacy, platform rules, music and image
   licences and advertising disclosure each can block a release, and the block
   is recorded as `content.rights.blocked`.
6. **Publishing is irreversible in practice**, so it is gated on a human
   approving the exact text and assets that will ship. Nothing leaves this OS
   on a generated approval.
7. **Every asset has one job in the audience journey**, and is measured against
   that job. Performing well at a job it was not given is a finding, not a win.
8. **No material record without a source and a timestamp.** Deletion,
   correction and export stay possible for every record this OS holds.
9. **Measurement improves judgment, it does not replace it.** The performance
   council reports what would change the recommendation, not a vanity score.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| a story arrives without a Storyteller truth verdict | refuse to package it, name the missing verdict, leave the slot empty |
| a claim cannot be given an epistemic label | drop the claim, keep the asset, report the drop |
| rights, licence or consent unresolved | emit `content.rights.blocked`, do not publish, name what would clear it |
| the claim from Positioning contradicts a planned pillar | stop, state the contradiction, escalate to Positioning {OS}, do not reword around it |
| an asset would name a third party without consent | refuse, offer anonymisation or omission, do not publish pending resolution |
| platform metrics unavailable for a measure cycle | report the gap, judge on what exists, never infer performance |
| a Growth hypothesis requires copy that breaks a Brand voice rule | report the conflict to both, do not silently pick one |
| no candidate clears the bar for a scheduled slot | leave the slot empty and say so |

Abstention is a valid output. A confident guess about a result, a quote or a
consent state is not.

## 9. Human approval boundary

Content {OS} asks before:

- publishing to any external channel, in the exact wording and with the exact
  assets that will ship
- any paid distribution spend
- publishing content that names a customer, a partner or any third party
- publishing any material claim labelled E4 or E5 as though it were settled
- responding publicly on behalf of the operator in community engagement
- overriding a `content.rights.blocked` release block
- changing the boundaries, schemas or quality gates of the pack itself, which
  additionally requires Review & Governance {OS} approval in production

Nothing customer-facing is sent or published without an explicit human approval
of the exact text. A generated asset is a draft until then.

## 10. Completion criteria

The operator can point at a calendar where every slot has a job, an audience
and a claim it supports; can trace every published asset back to a real piece
of captured material and a Storyteller truth verdict; can see the epistemic
label on every claim that went out; and can say, for the last cycle, which
assets did the job they were given and which merely performed.
