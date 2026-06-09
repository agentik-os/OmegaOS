# R-BROWSER — When to use browser-use (agentic) vs Playwright (scripted)

**Kind:** Rule
**Category:** Orchestration
**Added:** 2026-06-09

## Rule

Two browser-automation paths, split by **whether the steps are known in advance**:

1. **Playwright (Bun, via the Bash CLI)** — **deterministic / scripted** automation where the steps are **known up front**: OmegaOS's acceptance gate, golden-path route sweeps, and end-to-end testing of **our own apps**. This is the **default** for our apps' E2E (see `/omg-acceptance` and **R-TEST**: drive the prod URL with Playwright via Bash — **never** an MCP browser tool).
2. **browser-use (the `browser-use-sdk` cloud SDK)** — **LLM-agentic**, natural-language browser tasks where the steps are **unknown up front**: navigate/extract on an arbitrary or unfamiliar site, fill an unknown form, do agentic web research/automation across UIs **we don't control**. The agent runs on the **Browser Use cloud**, not the local box.

**The decision rule (plainly).** Known steps + our app → **Playwright**. Unknown UI / open-ended / "figure it out" agentic browsing → **browser-use**. Never reach for browser-use for our own apps' deterministic E2E (that is Playwright/acceptance); never hand-script an unknown third-party UI step-by-step in Playwright when an agentic task would do.

**Where it triggers in OmegaOS.** `/omg-browser-use` + `/browser-use`; a bare "go to X and extract / do Y on a site we don't control" routes here. It is the **agentic complement** to the deterministic Playwright/acceptance path — the two cover disjoint halves of "drive a browser".

**External-dependency boundary (R-SEC).** `browser-use-sdk` is the **paid Browser Use cloud API** (Python, current major **v3**, base `https://api.browser-use.com`; clients `BrowserUse` / `AsyncBrowserUse`), authenticated by **`BROWSER_USE_API_KEY`** (key prefix `bu_`, sent as header `X-Browser-Use-API-Key`) and requiring a **paid plan**. OmegaOS ships **only the skill markdown plus a thin wrapper** — the `pip install browser-use-sdk` (venv at `~/.omega/skills/browser-use/.venv`, created lazily on first skill run) and the key are a **runtime opt-in**, **never** auto-installed by `install.sh`. The key lives in `~/.omega/secrets/integrations.env` (mode 600, gitignored) — **never** in the repo, **never** echoed or logged; the wrapper sources it at runtime when the env var is unset. Consequently a live agentic run is **not runtime-verifiable** inside OmegaOS without the operator's key; the skill resolving via its alias is the verifiable contract, the agentic run is not. This stays at the **skill layer** and does **not** enter `omega-core` (**R-STACK**: Rust/Bun first, Python only when a dependency demands it — here it does).

## Origin

OmegaOS had a deterministic browser path (Playwright/acceptance) but **no agentic-browsing primitive** for UIs it doesn't control; `browser-use-sdk` fills that gap. Without a written boundary agents would misuse the paid agentic cloud for routine E2E (or try to hand-script an unknown third-party UI in Playwright), so the decision rule and the paid-API / runtime-opt-in boundary — the same boundary as higgsfield (R-VISUAL-ID) and gooseworks (R-SEC) — are pinned explicitly: ship the markdown + wrapper, keep the pip install and key a user-invoked runtime opt-in, and never claim an agentic run as runtime-verified without the operator's key.
