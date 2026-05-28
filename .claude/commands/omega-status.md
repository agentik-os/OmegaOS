---
description: Show OmegaOS runtime status — active oracles, workers, queued missions, recent done.json events
---

Run `omega session list --json | jq -r '.[] | "\(.role) \(.name) \(.project // \"-\")"'` and summarize:

- Active oracles (and which project each owns)
- Active workers and their parent oracle
- AISB master session status (alive / dead / restart count)
- Last 5 `.done.json` events from `~/.omega/state/oracle-*.done.json` (mtime DESC)
- Any worker-blocked files in `~/.omega/state/worker-blocked-*.json`

Output a compact table. No prose.
