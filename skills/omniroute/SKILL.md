---
name: omniroute
description: >
  OmegaOS BYOK/API gateway (self-hosted OmniRoute): ONE OpenAI-compatible /v1 endpoint fronting 290+
  providers (40+ free) with smart routing (auto/best-coding|reasoning|fast), automatic fallback on
  quota/failure, and 15-95% token compression. Use it for API-based LLM calls: embeddings, a tool's
  BYOK proxy, cheap/background classification or fan-out, or any place OmegaOS already calls an LLM
  API by key. Point the tool at `http://127.0.0.1:20128/v1` (OPENAI_BASE_URL) and pick a model
  (e.g. `auto/best-fast`, `deepseek-v4-flash`, or a specific provider model). Triggers (EN): "api
  gateway", "route LLM calls", "cheap/free model", "fallback provider", "byok proxy", "compress
  tokens", "omniroute". Triggers (FR): "gateway API", "router les appels LLM", "modèle gratuit/pas
  cher", "provider de secours", "omniroute". NEVER route the subscription CLIs (claude/codex) through
  it — that turns the operator's subscription into per-token API cost, which OmegaOS deliberately avoids.
allowed-tools: ["Bash", "Read"]
metadata:
  source: omegaos
  version: "1.0"
---

# OmniRoute — the OmegaOS API gateway

A self-hosted Docker gateway ([diegosouzapw/OmniRoute](https://github.com/diegosouzapw/OmniRoute),
MIT) that exposes ONE OpenAI-compatible `/v1` in front of 290+ providers. It complements — never
replaces — the subscription/CLI core (R-MODEL, /duo, rotation): those stay on the operator's
subscription. OmniRoute is for the **API-key edges**.

## Quick use

```bash
omega-omniroute status     # container + dashboard + /v1 endpoint
omega-omniroute v1         # the /v1 base URL to hand a tool (OPENAI_BASE_URL)
omega-omniroute models     # routed models (auto/best-*, free providers, 290+ real)
omega-omniroute url        # tailnet dashboard link
```

Drive any OpenAI-compatible client at it:
```bash
curl -s http://127.0.0.1:20128/v1/chat/completions -H 'content-type: application/json' \
  -d '{"model":"auto/best-fast","messages":[{"role":"user","content":"hi"}]}'
```

## When to use it (API-based calls ONLY)

- **Embeddings** (e.g. the skill-RAG) and other API-key LLM work → fallback + free-tier + one key store.
- **A tool's BYOK proxy** (e.g. open-design's `/api/proxy/*`) → point it at OmniRoute `/v1`.
- **Cheap / background / high-volume fan-out** (classification, extraction, veille) → free providers.
- Resilience: one provider hits a quota → OmniRoute falls back automatically.

## The hard boundary

- **NEVER** route the core Claude Code / Codex sessions through OmniRoute. Those run on the operator's
  SUBSCRIPTION (Claude Max, Codex) — sending them through an API gateway bills per token and defeats
  OmegaOS's whole budget model (R-MODEL is subscription/CLI doctrine, a different layer).
- Keys live encrypted in the container's data volume; add them in the dashboard (`omega-omniroute url`),
  never in the repo. Served tailnet-only (no Funnel).

## Boundary (external dependency)

Docker image `diegosouzapw/omniroute:latest`, opt-in install
(`tools/omniroute/install-omniroute.sh`) — not auto-run by OmegaOS install.sh (same boundary as
open-design / ZernFlow / higgsfield). A live gateway is not runtime-verifiable without running it.
