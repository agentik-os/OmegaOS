# Open Design — the self-hosted design engine

Open Design (vendored from [nexu-io/open-design](https://github.com/nexu-io/open-design), Apache-2.0,
pinned in [`tools/open-design/README.md`](../tools/open-design/README.md)) turns a coding agent into
a design engine: prototypes, dashboards, landing pages, decks, images and video from a brand
contract (`DESIGN.md`), 150+ built-in design systems. It runs as a Docker daemon and is served
tailnet-only so you SEE the build in a browser.

## Quick access

```
omega-design open        # a card: the view link + how-to + detected agents
omega-design url         # just the link
omega-design status      # container + serve + health
omega-design projects    # OmegaOS projects you can open as a working directory
omega-design agents      # which local CLIs Open Design detected
```

**View link:** Tailscale up → `https://<tailnet-host>:7456`; no Tailscale → `http://localhost:7456`.
(This box: `https://station.tail64d114.ts.net:7456` — needs Tailscale on your device.)

## Install (opt-in — heavy Docker dependency, not auto-run by install.sh)

```
bash tools/open-design/install-open-design.sh
```

It clones (pinned), builds a CLI-baked image, wires auth + git + projects, starts the daemon,
serves it tailnet-only, and installs `omega-design`. Opt-outs: `OMEGA_SKIP_OD_LOCALCLI=1` (use the
UI's BYOK instead of the local CLIs), `OMEGA_SKIP_TS_SERVE=1`, `OMEGA_PROJECTS_ROOT=<dir>`.

## Local CLI mode = your subscription (not BYOK)

Open Design delegates the agent loop to a coding-agent CLI on the **daemon's** PATH. The daemon is a
Docker container, so it can't see the host's `claude`/`codex` — out of the box it finds none and errors
`vela binary not found`. OmegaOS fixes this by:

- baking `claude`+`codex` into an extension image `od-omega` (`deploy/Dockerfile.omega-agents`),
- running the container as the **host uid** so it reads the operator's auth and writes files with
  correct ownership,
- mounting a resolved copy of the operator's auth at `~/.omega/open-design-agent-home`
  (never the host `~/.claude` symlink), via `deploy/docker-compose.omega.yml`.

Result: the "Local CLI" picker detects **claude** and **codex** and runs on the operator's
subscription. `omega-design agents` confirms it.

## Redesign an existing project (refonte)

The OmegaOS projects root (`~/Station`) is mounted into the container at the **same path**. In Open
Design, **Select working directory** → a project path from `omega-design projects`
(e.g. `/home/vibe/Station/Partners/rm/rm-site`). The agent reads and writes the real project files,
and git is wired (`~/.omega/secrets/agentik-os.git-credentials`) so it can push.

## Marketing machine

Build the visual system and viewable creatives (landing pages, decks, ad mocks) here; the operator
reviews them at the link. Publishing still goes through Zernio (R-ZERNIO). Open Design BUILDS + shows;
Zernio DISTRIBUTES.

## Security posture

Container bound to `127.0.0.1:7456`; exposure is `tailscale serve` (tailnet-only, no Funnel). The
tailnet is the trusted auth layer, so daemon token-auth is disabled behind it. The API token still
lives in `~/.omega/secrets/open-design.env`. Running as the host uid with `~/Station` mounted
read-write is deliberate (so a refonte can write) — keep it tailnet-only; never add Funnel without
re-enabling `OD_API_TOKEN` and dropping the projects mount.
