# Builder {OS} v4.1.0

## Research-first, evidence-complete OS construction

Builder {OS} converts a domain name into a complete, executable operating system.

The critical rule in v3 is simple:

> Never architect an OS directly from its name.
>
> First build the strongest possible domain knowledge base. Then convert that knowledge into executable system logic.

A bare command such as:

```text
/os-build Mindset {OS}
```

must autonomously trigger the complete pipeline:

```text
DOMAIN FRAMING
→ OUTCOME MAP
→ RESEARCH PROTOCOL
→ BESTSELLER DISCOVERY
→ CORPUS CURATION
→ /book --deep FOR EVERY RETAINED BOOK
→ CURRENT AND PRIMARY EVIDENCE
→ CLAIM AND EVIDENCE LEDGER
→ CONTRADICTION MAP
→ KNOWLEDGE SYNTHESIS
→ DOMAIN ONTOLOGY
→ DECISION RULES
→ CAPABILITIES
→ WORKFLOWS, GRAPHS AND LOOPS
→ COMMANDS, AGENTS, SKILLS, TOOLS AND MEMORY
→ IMPLEMENTATION
→ TESTS AND EVALS
→ GAUNTLET AND OMEGA AUDIT
→ REPAIR
→ DOCUMENTATION AND PRESENTATION
→ VERSIONED ZIP
→ REGISTRY UPDATE
```

## Books are mandatory, but books are not the whole evidence base

Builder {OS} routes book intelligence through Librarian {OS}:

1. `/bestseller <domain>` maps influential, practical and canonical books.
2. Corpus curation removes redundancy and fills missing perspectives.
3. `/book --deep <title>` runs for every book admitted to the retained corpus.
4. Each analysis is normalized into the same extraction schema.
5. The corpus is compared, contradicted, synthesized and translated into system components.

The build then expands beyond books through Research {OS}:

- current primary research and systematic evidence;
- official standards, regulations and documentation;
- expert consensus and specialist frameworks;
- practitioner playbooks and field cases;
- failure reports and counter-evidence;
- datasets, benchmarks and current developments where relevant.

This prevents popularity from being confused with truth and prevents research from remaining theoretical.

## What the package contains

- `AGENTS.md`: durable Codex and Work instructions.
- `MASTER_PROMPT.md`: canonical autonomous Builder prompt.
- `OS_DEFINITION.md`: purpose, boundaries and core model.
- `HOW_TO_USE.md`: full user guide.
- `commands/COMMAND_REFERENCE.md`: exhaustive commands, syntax and examples.
- `docs/`: research, synthesis, architecture, quality and update protocols.
- `workflows/`: machine-readable orchestration definitions.
- `schemas/`: source, claim, book analysis, build state and OS manifest schemas.
- `agents/`: specialized agent roster and handoff rules.
- `templates/`: source-of-truth files generated for every new OS build.
- `evals/`: quality rubric, release gates and test scenarios.
- `scripts/`: workspace scaffolding and package validation.
- `registry/`: command, capability and inter-OS handoff registries.
- `codex/`: installation and long-horizon execution instructions.

## Quick start

Create a build workspace:

```bash
python scripts/create_build_workspace.py "Mindset" --output ../builds
```

Then run the canonical command in ChatGPT Work or Codex:

```text
/os-build Mindset {OS}
```

Builder {OS} must not ask basic intake questions. It infers sensible defaults, records assumptions in `BUILD_CONTRACT.yaml`, and proceeds unless a missing fact makes safe execution impossible.

## Release standard

An OS is not releasable because its article looks complete. It is releasable only when:

- the retained corpus has adequate coverage;
- all retained books have passed deep-analysis validation;
- material claims have provenance and confidence;
- contradictions are represented rather than hidden;
- knowledge has been converted into executable rules and workflows;
- commands map to real capabilities;
- critical scenarios pass evals;
- red-team findings have been repaired or explicitly accepted;
- documentation, command reference, examples and package manifest agree;
- the OS remains standalone while optional inter-OS handoffs stay typed, traceable, disableable and user-controlled.

## Build Factory v4

See `docs/BUILD_LIFECYCLE_ROADMAP.md` and `docs/GENERATED_BUILD_FILE_TREE.md`. A new build now creates the complete 26-phase repository skeleton at initialization.


## Public release security

All builds follow `docs/PUBLIC_NO_SECRETS_POLICY.md`. No API keys, tokens, credentials, or secret-key setup may be included in public OS packages.
