# R-SKILLPUB — Every new skill ships to the library + OmegaOS

**Kind:** Rule
**Category:** Reporting
**Added:** 2026-06-07

## Rule

A NEW skill is NOT done until it is published to BOTH sources of truth: (1) the operator's skill library `github.com/agentik-os/Agentik-Skills` (one folder per skill), and (2) OmegaOS itself — `skills/<name>/` in the repo + its `install.sh` copy block + `~/.omega/skills/<name>/` — committed AND pushed. A skill that lives only locally does not exist (lost on reset, never shipped via `npx`). OmegaOS is the SSOT; the library is the shareable mirror. Wire any Telegram / menu entry that triggers it in the same change.

## Origin

Skills were built and used locally but never pushed to the library nor wired into OmegaOS, so they were lost on reset and never shipped to other installs. Publishing every new skill to both SSOTs makes them durable and shareable.
