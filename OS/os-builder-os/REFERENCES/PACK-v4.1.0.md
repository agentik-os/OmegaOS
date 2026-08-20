# The vendored pack, v4.1.0

`pack/v4.1.0/` holds the operator pack `agentik-builder-os-v4.1.0-public.zip`
verbatim, 75 files. **It is never edited.** Divergences between the pack and this
OmegaOS unit are recorded here instead, so the pack stays a clean upstream.

- Received 2026-08-18 via the Telegram DEPOSIT bot
- Source archive `~/.omega/inbox/2026-08-18_183850_agentik-builder-os-v4.1.0-public.zip`
- SHA-256 `be74e0bf107ed934609db503b7fd22510f7bf4d13ff0a6892ecfacaf807d0a98`
- Safety glance done before extraction: no network call, no curl-pipe-sh, no
  credential access. The four Python scripts are workspace validators and a
  packager.

## What the pack brings

| Area | Files |
|------|-------|
| Method | `docs/CANONICAL_PIPELINE.md`, `METHODOLOGICAL_FOUNDATIONS.md`, `OS_ARCHITECTURE_STANDARD.md` |
| Corpus and books | `docs/RESEARCH_AND_CORPUS_PROTOCOL.md`, `docs/BOOK_DEEP_EXTRACTION_STANDARD.md` |
| Synthesis | `docs/KNOWLEDGE_SYNTHESIS_PROTOCOL.md` |
| Inter-OS | `docs/INTER_OS_ORCHESTRATION.md` |
| Gates | `docs/QUALITY_GATES.md`, `evals/RELEASE_GATES.md`, `evals/ULTIMATE_OS_RUBRIC.yaml` |
| Schemas | `book-analysis`, `source`, `claim`, `decision-rule`, `os-manifest`, `build-state`, `artifact-index` |
| Templates | build contract, source ledger, claim ledger, corpus matrix, synthesis map, research protocol, eval report |
| Agents | `agents/AGENT_ROSTER.md`, 15 specialist roles |
| Workflows | `os-build-ultimate`, `book-deep-fanout`, `evidence-synthesis`, `eval-repair-loop` |
| Scripts | `create_build_workspace`, `validate_build_workspace`, `package_os_release`, `validate_package` |

## Declared divergences

1. **The suite contract wins on file layout.** The pack describes a 24-directory
   build workspace; an OmegaOS unit ships the 23-file contract that
   `OS/_tools/verify.py` enforces. The workspace is where a build HAPPENS, the
   23 files are what it DELIVERS. Both are kept, neither is rewritten.
2. **Registry edits stay human-approved.** The pack's release stage proposes a
   registry line; on this box `OS/_tools/suite.py` is the SSOT and only a human
   authorizes the edit.
3. **NODASH is stricter here.** `verify.py` fails a unit on any em or en dash.
   The pack tolerates them in prose.
4. **No auto-publish.** The pack's Release {OS} role can publish to a registry.
   On this box publishing off the machine is an operator decision.

## Upstream drift

When a v4.2.0 lands, vendor it beside this one as `pack/v4.2.0/` rather than
overwriting. The unit then names which pack version it implements, and the
divergence list is re-checked against the new pipeline.
