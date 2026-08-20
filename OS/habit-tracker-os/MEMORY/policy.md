# Habit Tracker {OS}: Memory Policy

<!-- agentik:scaffold -->

Not everything said deserves to become permanent context. This file declares
what this OS may remember, for how long, and what the user may remove.

| Tier | Example | Lifetime | User can delete |
|---|---|---|---|
| Temporary | a value used once in this answer | the turn | n/a |
| Session | what we are working on right now | the session | yes |
| Project | decisions scoped to one project | the project | yes |
| Preference | how the user wants things done | until changed | yes |
| Confirmed | a fact the user explicitly confirmed | durable | yes |
| Outcome | what happened and what it taught | durable | yes |

## Never stored

Credentials, secrets, and anything the user marks private. Canonical durable
state routes through Context & Memory {OS} rather than living here.

## Retrieval

Only what is relevant to the current mission is loaded. Loading everything is
the failure mode this policy exists to prevent.
