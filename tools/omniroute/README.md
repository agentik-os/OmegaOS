# OmniRoute — vendored tool (BYOK/API gateway)

Upstream: **github.com/diegosouzapw/OmniRoute** (MIT). Docker image
`diegosouzapw/omniroute:latest`.

ONE OpenAI-compatible `/v1` endpoint in front of 290+ providers (40+ free), with smart routing
(`auto/best-coding|reasoning|fast`), automatic fallback on quota/failure, and 15-95% token
compression. OmegaOS runs it as a Docker container and serves the dashboard tailnet-only.

## What it is FOR (and NOT for)

- **For:** the API-key edges — embeddings (skill-RAG), a tool's BYOK proxy, cheap/background/high-
  volume LLM calls, provider fallback. It complements the subscription core.
- **NOT for:** routing the core Claude Code / Codex sessions. Those run on the operator's
  SUBSCRIPTION; sending them through an API gateway bills per token and defeats OmegaOS's budget
  model (R-MODEL / the CLI+rotation layer stays untouched).

## Boundary (opt-in, like open-design / ZernFlow / higgsfield)

OmegaOS ships only this markdown + `install-omniroute.sh` + the `omega-omniroute` CLI + the
`omniroute` skill. The Docker pull + run are a **runtime opt-in** — `install.sh` does NOT start it.

## Install (opt-in)

```bash
bash tools/omniroute/install-omniroute.sh
```

Pulls + runs the container (`127.0.0.1:20128`), serves the dashboard tailnet-only, installs the CLI
and skill, verifies `/v1/models`. Opt out of the serve step with `OMEGA_SKIP_TS_SERVE=1`.

## Use

- `omega-omniroute status | url | v1 | models | up | down | serve | api <path>`
- Agent doctrine: the `omniroute` skill (`~/.claude/skills/omniroute`).
- Dashboard: `https://<tailnet-host>:20128` (add API keys here — encrypted locally, never the repo).
- `/v1` for tools: `http://127.0.0.1:20128/v1`.

## Security posture

Bound to `127.0.0.1:20128`; dashboard exposed via `tailscale serve` (tailnet-only, no Funnel). Keys
are AES-256 encrypted in the `omniroute-data` volume; no telemetry, local-first. Never add Funnel.
