# R-ARTIFACT — Reports default to a LOCAL self-hosted artifact (Tailscale), 3-surface router

**Kind:** Rule
**Category:** Reporting
**Added:** 2026-07-03

## Rule

Deliverable reports (audit, research memo, strategy doc, mission recap, brief) route across THREE surfaces. **"Artifact" in operator language means a LOCAL, self-hosted page on the machine, reachable over Tailscale (like kairos), NEVER a claude.ai-account artifact by default.**

(1) DEFAULT, a report asked for with no format specified ships as a **LOCAL SELF-HOSTED ARTIFACT**: load the artifact-design skill, write ONE self-contained HTML under the project deliverable folder (`agentic/reports/` where the convention exists) AND drop a standalone copy into the local artifact host `~/.omega/artifacts/` (wrap the content-only HTML in a full `<!doctype html>` document). That directory is served tailnet-only by `tailscale serve --bg --https=8443 ~/.omega/artifacts` (no Funnel, private to the tailnet), so every artifact is live at `https://station.tail64d114.ts.net:8443/<file>.html` and listed on the `/` index. Update `~/.omega/artifacts/index.html` with the new entry. Hand back the **Tailscale URL** plus the repo file path. This is the DEFAULT because it keeps deliverables on the operator's own box, private, no external account. The page still follows the artifact contract: self-contained HTML, inline CSS/JS zero external hosts, both themes via token-level `prefers-color-scheme` plus `:root[data-theme]` overrides, premium design standard, zero em/en dashes in visible copy (R-NODASH).

**claude.ai native Artifact tool, account-gated (HARD):** publishing to a claude.ai account is permitted **ONLY when the active Claude account is `x@agentik-os.com`** (the operator consented to that ONE account). On ANY other account (e.g. `city.dentistrygpt@gmail.com`, the default session), NEVER publish a claude.ai artifact, fall back to the local surface above and say which surface fired. To check the active account before any Artifact-tool call: `python3 -c "import json,os;print(json.load(open(os.path.expanduser('~/.claude.json'))).get('oauthAccount',{}).get('emailAddress'))"`. If a report has already been published to the wrong account, redact it immediately (republish the same URL via the Artifact tool `url` param with a tombstone page, since there is no delete tool) and tell the operator to delete the shell from the claude.ai UI.

(2) HTML (R-HTML), when the operator wants a FILE (attachment, email, repo doc, offline reading), ship the self-contained HTML file and say so. The local-artifact copy is the same self-contained file, so surface 1 always produces surface 2 as a side effect.

(3) PDF (R-PDF), ONLY on explicit ask, via `omega pdf` (never hand-rolled).

Never claim a live URL without having served it (L1): verify the Tailscale URL returns HTTP 200 before reporting it. A headless/cron session that cannot serve falls back to surface 2 (the file) and says so, never fabricates a URL.

## Origin

Operator directive (2026-07-03): the earlier version of this rule defaulted reports to a live **claude.ai** artifact. The operator corrected it: "artifact" means **auto-hebergé en local sur la machine, accès via Tailscale comme kairos**, NOT on the claude account, and claude.ai publishing is acceptable ONLY on `x@agentik-os.com`, never the other accounts. A strategy report (Verba 90-day plan) had been published to the default `city.dentistrygpt@gmail.com` account; it was redacted and re-served locally at `https://station.tail64d114.ts.net:8443/`. The router now encodes the real capability boundary: local Tailscale-served HTML is the private, operator-owned default (mirroring the kairos serving pattern: Caddy + `tailscale serve`), the claude.ai native tool is a single-account exception, and PDF/HTML remain surfaces 2 and 3.
