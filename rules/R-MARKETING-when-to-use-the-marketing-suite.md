# R-MARKETING — When to use the marketing / go-to-market suite

**Kind:** Rule
**Category:** Orchestration
**Added:** 2026-06-09

## Rule

When a mission is **marketing / go-to-market / growth / launch / outbound / content** — for a client project or for an OmegaOS product — reach for the vendored marketing suite, and invoke the skills in **dependency order**, never ad-hoc:

1. **market-research** *(research, upstream-most)* — size the market, map competitors and decision-makers before any positioning. Requires gooseworks credentials (`~/.gooseworks/credentials.json` via `npx gooseworks login`); it calls an external paid data API, so it is not runtime-verifiable without those creds.
2. **product-marketing-context** — frame positioning, ICP, persona, messaging. Run this FIRST among the prompt skills: it writes `.agents/product-marketing.md`, the context file every other marketing skill reads, so foundational info is stated once.
3. **marketing-strategist** — the strategist lens: GTM strategy, demand-gen architecture, category creation. Frames the campaign before tactics.
4. **launch-strategy** — plan a specific launch moment (Product Hunt, waitlist, announcement, beta/early-access).
5. **content-strategy** — decide *what* content to produce (pillars, topic clusters, editorial calendar). Upstream of the content-production skills.
6. **social-content** — produce organic social posts, threads, carousels, short-form video scripts (downstream of content-strategy).
7. **ad-creative** — produce paid-ad creative at scale (headlines, primary text, RSA variants). Pairs with **R-VISUAL-ID** `higgsfield-generate` for the *visual* half of paid creative.
8. **cold-email** — write outbound sequences (subject lines, openers, follow-ups).

**How to invoke.** These are prompt-driven strategy/copy skills (the eight ship no executable code except `market-research`, which only `curl`s the gooseworks API with the user's own bearer token). Invoke the real `/omg-<skill>` (or `/<skill>`) — never paraphrase the skill into prose (R-AUDIT discipline). Disjoint marketing sub-tasks fan out in parallel; the dependency chain above serializes only what truly feeds the next stage.

**Where it triggers in OmegaOS.** The suite is the **go-to-market layer** of the new-project pipeline: it slots in **after** `/omg-brand-identity` (vision → PRD → brand-identity → **marketing GTM**). Any oracle dispatched a marketing/GTM/growth mission drives this chain (research → context → strategy → launch/content → social/ad/email). The `/omg-*` aliases and keyword triggers route a bare "write me cold emails" / "plan our launch" / "what should we post" to the right skill.

## Origin

OmegaOS could research, design a brand, write a PRD, and build a product — but had no canonical **go-to-market layer**. A marketing mission had no real skill to invoke and no ordering, so agents improvised GTM from scratch each time, inconsistently. Vendoring the marketing suite (8 skills) and pinning the dependency order makes go-to-market reproducible and slots it into the new-project pipeline after brand-identity. `market-research` adds an upstream research primitive (external paid API), folded into this rule rather than a separate research rule to keep the registry lean.
