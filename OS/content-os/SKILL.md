---
name: content-os
description: Operate the full content lifecycle from positioning and daily capture through story mining, research, writing, visual/audio/video production, platform-native packaging, publishing, community engagement and performance learning. Contains 38 specialist agents, 44 skills, 12 protocols and 10 schemas. Use for content strategy, daily capture, story mining, pillar assets, multi-platform cascades (Instagram, TikTok, YouTube, LinkedIn, X, newsletter, article), visual/video/sound briefs, editorial calendars, or performance review. Trigger words: content, capture, story mine, pillar, cascade, platform package, visual brief, video brief, content calendar; FR: contenu, capture du jour, mine d'histoires, cascade multi-plateforme, brief visuel, brief video, calendrier editorial.
---

# Content {OS}

Runtime-installed pack (2026-08-11), staged for the OmegaOS repo-level R-SKILLPUB integration by a concurrent session. This SKILL.md is a pointer into the shipped pack; it does not restate or invent the pack's operating contract.

## Load before operating

- [README.md](README.md) for purpose, operating loop, commands and main handoffs.
- [system/SYSTEM_PROMPT.md](system/SYSTEM_PROMPT.md) for the full operating contract.
- [system/PRINCIPLES.md](system/PRINCIPLES.md) and [system/BOUNDARIES.md](system/BOUNDARIES.md) for scope and limits.
- [system/ROUTER.md](system/ROUTER.md) for command/intent routing.
- [MANIFEST.json](MANIFEST.json) for the full inventory (agents, skills, protocols, schemas).
- [OMEGA_INTEGRATION.md](OMEGA_INTEGRATION.md) for registration ID, event types and cross-OS handoffs.
- `agents/*.md` for specialist agent definitions, `skills/*.md` for reusable skill procedures, `protocols/*.md` for multi-step operating protocols, `schemas/*.json` for the data model.

## Commands

| Command | Mode | Purpose |
| --- | --- | --- |
| `/content` | strategy | Open Content OS |
| `/content-gps` | strategy | Define positioning and content system |
| `/capture-day` | capture | Ingest the day as source material |
| `/story-mine` | mine | Find stories and insights |
| `/pillar` | create | Create a pillar asset |
| `/cascade` | cascade | Build a multi-platform waterfall |
| `/instagram` | platform | Create Instagram-native package |
| `/tiktok` | platform | Create TikTok-native package |
| `/youtube` | platform | Create YouTube package |
| `/linkedin` | platform | Create LinkedIn package |
| `/x` | platform | Create X package |
| `/newsletter` | platform | Create newsletter |

## Boundary

This pack is a runtime skill install only. It does not modify the OmegaOS repository, its install.sh, os_products.rs or OS-SUITE.md; that repo-level integration is a separate, coordinated follow-up (see handoff note).

## Suite contract

Content {OS} is unit 33 of the AGENTIK {OS} suite, in 04 GROW. The full
operating specification is [OS.md](OS.md). This section states how the pack
behaves as a member of the suite.

### When to use this

Use Content {OS} when the question is editorial: what to say, in what order, on
which surface, at what cadence, and whether the last cycle worked. It is the
right unit for the calendar, the cascade, the platform package, the production
brief and the performance review.

Near neighbours it is confused with:

- **Storyteller {OS}** owns narrative craft: mining a lived moment, structure,
  scene, voice fidelity, truth class and consent. If the question is whether the
  story is true, deep enough, or the storyteller's own, it is Storyteller.
  Content receives the finished story object and packages it.
- **Brand {OS}** owns voice rules and the visual system. Content operates
  inside them and can fail against them; it does not set them.
- **Positioning {OS}** owns the claim. A pillar that contradicts the claim is a
  Positioning escalation, not an editorial rewrite.
- **Growth {OS}** owns loops and experiments. Growth hands Content a hypothesis;
  Content still applies its own publishing gate to the resulting asset.
- **Affiliate {OS}** owns partner selection for a promotion. Content owns the
  slot the promotion runs in.

The discriminating question: **is this about the story, or about the slot?**
Story is Storyteller. Slot, surface, cadence and measurement are Content.

### Capabilities

- Define a content GPS: audience, pillars, cadence, and what will not be covered.
- Capture a day of real work as sourced, timestamped material.
- Mine captured material for candidate insights and proof, each epistemically labelled.
- Build a pillar asset and cascade it into native packages per surface.
- Produce visual, video and sound briefs carrying rights, licences and accessibility.
- Build and maintain the editorial calendar, every slot with a stated job.
- Publish through a human approval gate on the exact text and assets.
- Run the performance council and report what would change the recommendation.
- Block a release on rights, consent or accessibility (`content.rights.blocked`).

### Procedure

1. **Pull the constraints first**: the claim from Positioning {OS}, the voice
   rules and visual system from Brand {OS}. State which versions are in use.
2. **Capture or receive the material.** Every record carries a source and a
   timestamp. Low-confidence extractions stay staged.
3. **Mine for candidates.** Anything that is a story goes to Storyteller {OS}
   and comes back as a story object with a truth verdict and a consent record.
   Nothing is packaged without one.
4. **Label every material claim** on the E1 to E5 scale before it enters a draft.
5. **Build the pillar**, then cascade natively. Each surface's package obeys
   that surface's grammar; a crop is not an adaptation.
6. **Clear rights, licences, likeness, privacy and accessibility.** An unresolved
   item emits `content.rights.blocked` and stops the release.
7. **Get explicit human approval of the exact text and assets.**
8. **Publish, then measure against the job the asset was given**, not against
   whichever metric it happened to move.
9. **Route the learning**: `content.performance.feedback` to Storyteller {OS},
   performance to Growth {OS} and KPI & Analytics {OS},
   `content.intent.qualified` to Sales {OS}.

### Handoffs

| Receiver | What it gets | What it expects |
|---|---|---|
| Growth {OS} | the editorial calendar and per-asset performance | the asset's stated job and the metric definition used |
| Sales {OS} | `content.intent.qualified`, and the calendar | a reader signal with its source asset attached |
| KPI & Analytics {OS} | published asset performance | figures against canonical metric definitions |
| Storyteller {OS} | `content.performance.feedback` | performance tied to a story object, for story learning only, never as narrative direction |
| Positioning {OS} | a contradiction between a planned pillar and the claim | the pillar, the claim, and the exact contradiction |
| Review & Governance {OS} | any change to boundaries, schemas or quality gates | the change and its blast radius, in production |

Content {OS} hands nothing to Delivery & Customer Success {OS} and nothing to
Revenue {OS} directly: commercial conversion runs through Sales {OS}, and cash
through Revenue {OS}. In this suite the legacy note that Revenue {OS} owns
offers and pipeline is superseded: Offer {OS} owns what is sold, Pricing {OS}
owns the price, Sales {OS} owns the pipeline, and Revenue {OS} owns the cash.
