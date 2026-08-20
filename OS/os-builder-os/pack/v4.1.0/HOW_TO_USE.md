# How to use Builder {OS}

## 1. The default command

```text
/os-build <Name> {OS}
```

Example:

```text
/os-build Search Funds {OS}
```

With no extra options, Builder uses `ultimate` mode and executes the full research-first pipeline.

## 2. What happens automatically

A bare name triggers:

1. domain and outcome inference;
2. research protocol;
3. bestseller and canonical corpus discovery;
4. retained-corpus selection;
5. `/book --deep` for every retained title;
6. current and primary evidence expansion;
7. claim and contradiction mapping;
8. synthesis into mechanisms and decision rules;
9. OS architecture and implementation;
10. tests, red team, repair, documentation and packaging.

The user does not need to supply a reading list or architecture.

## 3. Recommended execution surface

Use normal ChatGPT for:

- discussing an OS concept;
- reviewing a build;
- running a command from an already installed OS;
- making targeted changes;
- generating a compact prototype.

Use ChatGPT Work or Codex for:

- a complete new OS build;
- dozens of source analyses;
- parallel book-deep workers;
- repository implementation;
- long-running tests and repair loops;
- ZIP and registry delivery.

## 4. Build modes

### Ultimate

```text
/os-build Mindset {OS} --mode ultimate
```

Default. Maximum coverage, synthesis, implementation and evaluation.

### Systematic

```text
/os-build Health & Energy {OS} --mode systematic
```

Prioritizes explicit evidence protocols, source quality and uncertainty.

### Current

```text
/os-build AI Regulation {OS} --mode current
```

Prioritizes recency, official sources, change detection and dated claims.

### Field

```text
/os-build Sales {OS} --mode field
```

Adds practitioner playbooks, cases, scripts, postmortems and performance metrics.

### Technical

```text
/os-build Context & Memory {OS} --mode technical
```

Prioritizes specifications, official documentation, papers, implementation patterns and benchmarks.

### Regulated

```text
/os-build Financial Advice {OS} --mode regulated
```

Adds jurisdiction, compliance, escalation and high-risk release gates.

Modes alter the evidence mix and tests. They do not remove the mandatory book and synthesis stages unless books are genuinely irrelevant and the exception is recorded.

## 5. Inspecting progress

```text
/os-build-status <os-slug>
```

Shows completed milestones, active workers, failed gates, source counts, corpus coverage and next actions.

```text
/os-build-explain <os-slug> --rule <rule-id>
```

Explains which sources, claims and design decisions produced a rule or workflow.

## 6. Reviewing the corpus

```text
/os-corpus-discover <domain>
/os-corpus-audit <os-slug>
/os-evidence-map <os-slug>
```

Use these commands to inspect source coverage, redundancy, school diversity and unresolved gaps.

## 7. Running evaluation and repair

```text
/os-eval <os-slug> --suite all
/os-gauntlet <os-slug>
/os-repair <os-slug> --from eval-report
```

Critical failures block release.

## 8. Updating an existing OS

```text
/os-update <os-slug> --since 2026-01-01
```

The updater finds new evidence, compares claims, recompiles affected logic, runs regression tests and produces migration notes.

## 9. Final deliverables

Every ultimate build must contain:

- complete OS package;
- versioned manifest;
- source and claim ledgers;
- synthesis map;
- commands and workflows;
- prompts, skills, tools and agents;
- schemas and templates;
- tests and eval reports;
- audit and repair report;
- HOW_TO_USE;
- complete presentation article and command reference;
- versioned ZIP;
- registry entries.

## 10. Minimal user input, maximal builder responsibility

The user owns the objective and final judgment.

Builder {OS} owns research depth, knowledge coverage, architecture quality, implementation completeness, validation and traceability.


## 11. Full build repository initialization

Create the complete 26-phase build repository:

```bash
python scripts/create_build_workspace.py "Search Funds" --output builds
```

Validate it:

```bash
python scripts/validate_build_workspace.py builds/search-funds-os
```

The full roadmap and every generated file are documented in:

- `docs/BUILD_LIFECYCLE_ROADMAP.md`
- `docs/GENERATED_BUILD_FILE_TREE.md`
- `docs/BUILD_ARTIFACT_CONTRACT.md`

After release gates pass, package the built OS:

```bash
python scripts/package_os_release.py builds/search-funds-os --version 1.0.0
```


## Public release security

All builds follow `docs/PUBLIC_NO_SECRETS_POLICY.md`. No API keys, tokens, credentials, or secret-key setup may be included in public OS packages.
