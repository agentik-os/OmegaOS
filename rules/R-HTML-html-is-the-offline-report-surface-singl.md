# R-HTML — HTML is the offline report surface (single self-contained file)

**Kind:** Rule
**Category:** Reporting
**Added:** 2026-07-02

## Rule

A report delivered as a FILE ships as ONE self-contained HTML (inline CSS, no external assets/CDN, opens offline anywhere), styled to be genuinely pleasant to read: clear typography, a sticky/linked table of contents for long docs, readable tables, scorebars/badges, print-friendly `@media print`. Write it under the project's deliverable folder (`agentic/reports/` where the convention exists) and tell the operator the path. Within the report router (R-ARTIFACT), HTML is surface 2: the generic 'give me a report' ask goes to a live artifact FIRST when the session has the Artifact tool; HTML is the default whenever a file is wanted (attachment, email, repo-committed doc, offline reading) and the universal fallback when the artifact surface is unavailable (headless/cron sessions). The artifact path keeps an HTML twin anyway — the same self-contained file is what gets published live. Markdown may be the intermediate; the THING HANDED OVER is HTML. PDF (and docx/pptx) only on explicit ask — PDF via `omega pdf`, never hand-rolled (R-PDF). When unsure whether a doc counts as a report, default to the router's surface 1.

## Origin

The operator saw a large market-research deliverable rendered as HTML and asked that reports default to it over PDF (2026-07-02) — self-contained, instantly viewable, cheaper than a PDF pipeline. Compiled into the registry on 2026-07-03 (it lived only as a hand-written md, failing the registry-markdown parity gate) and reworded as surface 2 of the R-ARTIFACT router when the live-artifact surface shipped.
