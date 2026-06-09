# R-VISUAL-ID — When to use the Higgsfield visual-identity pair

**Kind:** Rule
**Category:** Orchestration
**Added:** 2026-06-09

## Rule

For **coherent visual identity** and **consistent-character** image/video generation, use the vendored Higgsfield pair, in order:

1. **higgsfield-soul-id** — train the identity anchor **once**. A "Soul" is a face-faithful model of a person/character; use it when a brand needs a consistent human face across all visuals (founder avatar, brand spokesperson, recurring character/mascot). Returns a `reference_id`.
2. **higgsfield-generate** — the generation engine. Produces brand images, videos, ads, UGC, product demos (Higgsfield Marketing Studio: branded ads, avatars, products, hooks). Pass the Soul via `--soul-id <ref_id>` for identity-faithful output.

**Where it triggers in OmegaOS.** This pair is the **visual-asset arm of the brand pipeline**:
- Inside `/omg-brand-identity` — **after** the brand book sets visual direction (palette, typography, logo, character direction), `higgsfield-soul-id` creates the consistent character and `higgsfield-generate` produces on-brand visual assets.
- Alongside **R-MARKETING** `ad-creative`: `ad-creative` writes the *copy*, `higgsfield-generate` produces the *visual* — together they make a complete paid-ad creative (copy + image/video).
- A bare "make our brand video", "generate a product demo", "create my Soul / digital twin" routes here via the `/omg-*` aliases.

**External-dependency boundary (R-SEC).** Both skills orchestrate the external `higgsfield` CLI, which they install at runtime via `curl … | sh`, then authenticate with `higgsfield auth login`, and which requires a **paid plan**. OmegaOS ships **only the skill markdown** — the CLI is a **runtime opt-in**, **never** auto-installed by `install.sh`. Consequently live generation is **not runtime-verifiable** inside OmegaOS without the operator's Higgsfield credentials; the skill resolving via its alias is the verifiable contract, the generated asset is not.

## Origin

OmegaOS could design a brand book but had no path from "visual direction" to **actually generated, identity-consistent visual assets**. `higgsfield-soul-id` + `higgsfield-generate` close that gap and plug visual identity into the brand pipeline (and into paid creative beside `ad-creative`). The external-CLI dependency is written down so the boundary is explicit: ship the markdown, keep the `curl|sh` CLI install a user-invoked runtime opt-in, and never claim a generated asset as runtime-verified without credentials.
