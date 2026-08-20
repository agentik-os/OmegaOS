# Agent instructions for Builder {OS}

This file is the durable runbook for Codex, ChatGPT Work and compatible agentic runtimes.

## Mission

Given only a domain name, build the strongest practical operating system for achieving the domain's intended outcomes.

Do not create an OS directly from model memory or from a short brainstorm. Research first, synthesize second, architect third, implement fourth, evaluate fifth, package last.

## Non-negotiable behavior

1. Never stop at a table of contents, outline, prompt or presentation.
2. Never ask basic scoping questions when sensible defaults can be inferred.
3. Record inferred scope, assumptions and exclusions in `BUILD_CONTRACT.yaml`.
4. Route book discovery and analysis through Librarian {OS} contracts.
5. Run `/book --deep` for every title admitted to the retained corpus.
6. Route current, primary, official and adversarial evidence through Research {OS} contracts.
7. Do not fabricate access to a book, paper, dataset, standard or source.
8. Preserve provenance from source to claim to principle to decision rule to workflow to command.
9. Keep empirical claims separate from opinion, values, heuristics and design decisions.
10. Represent meaningful disagreement. Do not force false consensus.
11. Translate knowledge into executable behavior: diagnostics, rules, states, workflows, loops, outputs and escalation paths.
12. Validate after every milestone and repair failures immediately.
13. Keep the OS standalone by default.
14. Make every external handoff typed, traceable, disableable and user-controlled.
15. Do not include API keys or credential boilerplate unless explicitly requested.
16. Complete HOW_TO_USE, the full `/presentation-os` command reference, examples, tests, audit reports and a versioned ZIP before release.

## Source-of-truth hierarchy

Use these files in this order:

1. `BUILD_CONTRACT.yaml`
2. `RESEARCH_PROTOCOL.yaml`
3. `BUILD_STATE.json`
4. `CORPUS_MATRIX.csv`
5. `SOURCE_LEDGER.jsonl`
6. `CLAIM_LEDGER.jsonl`
7. `SYNTHESIS_MAP.yaml`
8. `OS_MANIFEST.yaml`
9. `DECISION_LOG.md`
10. `EVAL_REPORT.md`
11. `RELEASE_REPORT.md`

When documents disagree, update the lower-priority artifact or record the intentional exception in `DECISION_LOG.md`.

## Long-horizon execution rules

- Work milestone by milestone.
- Keep changes scoped to the active milestone.
- Write status after every milestone to `BUILD_STATE.json` and `DECISION_LOG.md`.
- Fan out independent book analyses and evidence searches to parallel specialists.
- Do not let one book analyst see another analyst's conclusions before first-pass extraction.
- Merge only validated outputs.
- Run deterministic validators before model-based graders.
- Run model-based graders with explicit rubrics and evidence links.
- Stop and repair on failed critical gates.
- Preserve incomplete but valid progress if execution is interrupted.

## Completion definition

A build is complete only when every required artifact exists, all critical gates pass, unresolved risks are disclosed, commands are implemented or explicitly marked as interface-only, the versioned package validates, and the registry is updated.


## Public release security

All builds follow `docs/PUBLIC_NO_SECRETS_POLICY.md`. No API keys, tokens, credentials, or secret-key setup may be included in public OS packages.
