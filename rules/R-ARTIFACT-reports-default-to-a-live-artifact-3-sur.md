# R-ARTIFACT — Reports default to a live Artifact (3-surface router)

**Kind:** Rule
**Category:** Reporting
**Added:** 2026-07-03

## Rule

Deliverable reports (audit, research memo, strategy doc, mission recap, brief) route across THREE surfaces. (1) DEFAULT — a report asked for with no format specified ships as a LIVE ARTIFACT: load the artifact-design skill, write ONE self-contained HTML under the project deliverable folder (`agentic/reports/` where the convention exists), publish it with the native Artifact tool, and hand back the live URL plus the file path. Artifacts are private to the author on claude.ai (org-shareable, versioned republish at a stable URL; Team/Enterprise beta). The page follows the artifact contract: content-only HTML (no doctype/html/head/body wrapper), inline CSS/JS with zero external hosts (strict CSP), both themes via token-level `prefers-color-scheme` plus `:root[data-theme]` overrides, premium design standard, zero em/en dashes in visible copy (R-NODASH). Complex interactive artifacts (state, routing, shadcn/ui) go through the web-artifacts-builder skill IF installed (local-only today, not shipped by install.sh; absent = keep the artifact a hand-written single page) and publish its bundle.html. (2) HTML (R-HTML) — when the operator wants a FILE (attachment, email, repo doc, offline reading) or the session has NO Artifact tool (headless `claude -p` and cron runs lack it — runtime-verified 2026-07-03), ship the self-contained HTML and say which surface fired. (3) PDF — ONLY on explicit ask, via `omega pdf` (R-PDF). Never claim a live URL without having published it (L1); a session without the Artifact tool falls back to surface 2 and says so, never fabricates.

## Origin

Operator directive (2026-07-03): a report asked for with no format should land as a live claude.ai artifact — instantly viewable, versioned, shareable — with HTML the offline twin and PDF explicit-only. Runtime research proved the native Artifact tool and bundled artifact-design skill exist in entitled interactive sessions and are absent headless, so the router encodes the real capability boundary instead of assuming one surface fits every session.
