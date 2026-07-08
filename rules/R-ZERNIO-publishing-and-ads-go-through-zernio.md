# R-ZERNIO — Publishing & ADS go through Zernio

**Kind:** Rule
**Category:** Orchestration
**Added:** 2026-07-08

## Rule

Every social PUBLISH (organic post / reel / story / thread / carousel) AND every paid ADS action for an OmegaOS project account goes through Zernio — the single publishing funnel — via the `omega-zernio` CLI. NEVER hand-roll the Instagram/Facebook Graph API, a Composio poster, or a bespoke uploader for these accounts. One zernio profile = one project (map `~/.omega/zernio-profiles.json`); the key is `ZERNIO_API_KEY` in `~/.omega/secrets/integrations.env` (NOT the empty `zernio.env` — that footgun caused a bespoke Composio poster to be started while Zernio was already connected). List connected accounts with `omega-zernio accounts [project]`; publish with `omega-zernio post <project-slug> --text "…" --platforms instagram,tiktok,… --media <file|url> [--dry-run|--schedule ISO]` (auto-uploads local media to media.zernio.com). ALWAYS `--dry-run` to validate first, then confirm the post went LIVE (R-PROD/L1: `posted:true` is ACCEPTED, not published — Instagram finalizes reels async via `awaiting-finalize`; verify on the real profile). Platforms: facebook, instagram, linkedin, twitter, tiktok, youtube, threads, reddit, pinterest, bluesky, googlebusiness, telegram, snapchat, discord, whatsapp — plus the paid ADS accounts (metaads, googleads, linkedinads, pinterestads, xads, tiktokads). Pitfalls: YouTube/TikTok REQUIRE a video (an image → HTTP 400 that fails the WHOLE batch), Reddit requires a target subreddit, ads accounts are paid (not organic), and validation is all-or-nothing at creation. Sole documented legacy exception: Nova's own Instagram runs on a pre-wired Composio path; every other account defaults to Zernio.

## Origin

The assistant wrongly concluded publishing was not wired — it checked the empty `zernio.env` instead of the real `ZERNIO_API_KEY` in `integrations.env` — and started building a bespoke Composio/Graph-API poster for @agentik_os when Zernio already had the account connected and active. The operator mandated that ALL posting AND ads route through Zernio henceforth, so the publishing funnel is never re-derived or hand-rolled again. Complements R-MARKETING (what to produce) and R-VISUAL-ID (the visual half): R-ZERNIO owns the distribution step.
