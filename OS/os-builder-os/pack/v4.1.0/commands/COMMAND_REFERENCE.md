# Builder {OS} command reference

## Command family: build lifecycle

### `/os-build`

**Purpose:** Research, synthesize, architect, implement, test, audit, document and package a complete OS.

**Syntax:**

```text
/os-build <Name> {OS} [--mode ultimate|systematic|current|field|technical|regulated]
                       [--language <lang>]
                       [--audience <audience>]
                       [--jurisdiction <jurisdiction>]
                       [--from <existing-package>]
```

**When to use:** Whenever a new OS must be created or an incomplete OS must be rebuilt to Forge standard.

**Example:**

```text
/os-build Search Funds {OS} --mode field --language en
```

### `/os-build-status`

**Purpose:** Read the build state and summarize milestones, blockers and quality gates.

**Syntax:**

```text
/os-build-status <os-slug> [--verbose]
```

**When to use:** During a long-horizon build or before resuming interrupted work.

**Example:**

```text
/os-build-status search-funds-os --verbose
```

### `/os-build-resume`

**Purpose:** Resume from the last validated milestone without repeating completed work.

**Syntax:**

```text
/os-build-resume <os-slug> [--from <milestone-id>]
```

**When to use:** After interruption, failed execution or a manual review pause.

**Example:**

```text
/os-build-resume search-funds-os --from synthesis
```

### `/os-build-explain`

**Purpose:** Trace a principle, rule, command or workflow back to its sources and design decisions.

**Syntax:**

```text
/os-build-explain <os-slug> --rule <id>
/os-build-explain <os-slug> --command <command>
/os-build-explain <os-slug> --workflow <workflow-id>
```

**When to use:** For auditability, review, debugging or teaching.

**Example:**

```text
/os-build-explain mindset-os --rule RULE-RECOVERY-004
```

## Command family: research and books

### `/os-research-plan`

**Purpose:** Generate or audit the research protocol for the OS domain.

**Syntax:**

```text
/os-research-plan <os-slug> [--rebuild]
```

**When to use:** Before discovery or when the domain scope changes.

**Example:**

```text
/os-research-plan mindset-os --rebuild
```

### `/os-corpus-discover`

**Purpose:** Run bestseller, canonical, specialist, current and contrarian book discovery.

**Syntax:**

```text
/os-corpus-discover <domain> [--global] [--languages <langs>] [--refresh]
```

**When to use:** At the beginning of a build or during a major update.

**Example:**

```text
/os-corpus-discover "search funds" --global --languages en,fr,es
```

### `/os-corpus-curate`

**Purpose:** Score, deduplicate and select the retained book corpus using the coverage matrix.

**Syntax:**

```text
/os-corpus-curate <os-slug> [--target-saturation <0-1>]
```

**When to use:** After discovery and before per-book deep analysis.

**Example:**

```text
/os-corpus-curate search-funds-os --target-saturation 0.9
```

### `/os-book-deep`

**Purpose:** Run OS-oriented deep extraction for one or all retained books.

**Syntax:**

```text
/os-book-deep <os-slug> --book "<title>"
/os-book-deep <os-slug> --all-retained [--parallel]
```

**When to use:** After corpus curation or to repair an incomplete analysis.

**Example:**

```text
/os-book-deep search-funds-os --all-retained --parallel
```

### `/os-corpus-audit`

**Purpose:** Check coverage, redundancy, school diversity, missing analyses and saturation.

**Syntax:**

```text
/os-corpus-audit <os-slug>
```

**When to use:** Before synthesis and before release.

**Example:**

```text
/os-corpus-audit search-funds-os
```

### `/os-evidence-research`

**Purpose:** Expand beyond books into current, primary, official, technical and adversarial evidence.

**Syntax:**

```text
/os-evidence-research <os-slug> [--mode systematic|current|technical|field|regulated]
```

**When to use:** After book extraction or whenever current claims need verification.

**Example:**

```text
/os-evidence-research search-funds-os --mode field
```

### `/os-evidence-map`

**Purpose:** Display source classes, claims, support, contradiction, confidence and gaps.

**Syntax:**

```text
/os-evidence-map <os-slug> [--claim <id>] [--subdomain <name>]
```

**When to use:** During synthesis, audit or update.

**Example:**

```text
/os-evidence-map mindset-os --subdomain "belief change"
```

## Command family: synthesis and architecture

### `/os-contradictions`

**Purpose:** Build or inspect the school-of-thought and contradiction map.

**Syntax:**

```text
/os-contradictions <os-slug> [--unresolved-only]
```

**When to use:** Before compiling universal principles or decision rules.

**Example:**

```text
/os-contradictions mindset-os --unresolved-only
```

### `/os-synthesize`

**Purpose:** Convert validated claims into mechanisms, principles, conditions and decision rules.

**Syntax:**

```text
/os-synthesize <os-slug> [--rebuild] [--subdomain <name>]
```

**When to use:** After corpus and evidence gates pass.

**Example:**

```text
/os-synthesize mindset-os --rebuild
```

### `/os-ontology`

**Purpose:** Build the canonical domain entities, states, events and relationships.

**Syntax:**

```text
/os-ontology <os-slug> [--validate]
```

**When to use:** Before workflow and state-machine design.

**Example:**

```text
/os-ontology mindset-os --validate
```

### `/os-architecture`

**Purpose:** Compile principles and rules into capabilities, commands, workflows, state and handoffs.

**Syntax:**

```text
/os-architecture <os-slug> [--rebuild] [--target standalone|suite]
```

**When to use:** After synthesis and ontology validation.

**Example:**

```text
/os-architecture mindset-os --target standalone
```

## Command family: implementation and quality

### `/os-implement`

**Purpose:** Generate or update runtime artifacts, prompts, skills, tools, schemas, workflows and registries.

**Syntax:**

```text
/os-implement <os-slug> [--milestone <id>] [--provider-agnostic]
```

**When to use:** After architecture approval or during repair.

**Example:**

```text
/os-implement mindset-os --provider-agnostic
```

### `/os-eval`

**Purpose:** Run deterministic and rubric-based evaluation suites.

**Syntax:**

```text
/os-eval <os-slug> --suite all|knowledge|logic|commands|safety|ux|regression
```

**When to use:** After implementation and after every update.

**Example:**

```text
/os-eval mindset-os --suite all
```

### `/os-gauntlet`

**Purpose:** Run adversarial, ambiguity, misuse and degraded-dependency scenarios.

**Syntax:**

```text
/os-gauntlet <os-slug> [--severity critical|high|all]
```

**When to use:** Before release and after major logic changes.

**Example:**

```text
/os-gauntlet mindset-os --severity all
```

### `/os-repair`

**Purpose:** Patch failed gates, rerun targeted tests and prevent regressions.

**Syntax:**

```text
/os-repair <os-slug> --from <eval-report|audit-report|finding-id>
```

**When to use:** Whenever evals or audits fail.

**Example:**

```text
/os-repair mindset-os --from FINDING-GAUNTLET-017
```

### `/os-audit`

**Purpose:** Run the complete Forge, provenance, boundary and release audit.

**Syntax:**

```text
/os-audit <os-slug> [--omega] [--strict]
```

**When to use:** Immediately before packaging.

**Example:**

```text
/os-audit mindset-os --omega --strict
```

## Command family: release and lifecycle

### `/os-document`

**Purpose:** Generate HOW_TO_USE, architecture docs, examples and complete presentation command reference.

**Syntax:**

```text
/os-document <os-slug> [--presentation-os] [--language <lang>]
```

**When to use:** After logic stabilizes and before release.

**Example:**

```text
/os-document mindset-os --presentation-os --language en
```

### `/os-package`

**Purpose:** Validate, version and create the distributable ZIP.

**Syntax:**

```text
/os-package <os-slug> [--version <semver>] [--registry-update]
```

**When to use:** After all release gates pass.

**Example:**

```text
/os-package mindset-os --version 3.0.0 --registry-update
```

### `/os-update`

**Purpose:** Discover new evidence, diff claims and logic, migrate the OS and run regression tests.

**Syntax:**

```text
/os-update <os-slug> [--since <YYYY-MM-DD>] [--sources books,research,standards,cases]
```

**When to use:** On a schedule, after a major domain change or when a new edition appears.

**Example:**

```text
/os-update ai-regulation-os --since 2026-01-01 --sources research,standards
```

### `/os-diff`

**Purpose:** Compare two OS versions across sources, claims, rules, commands and workflows.

**Syntax:**

```text
/os-diff <os-slug> <version-a> <version-b>
```

**When to use:** During review, migration and release notes.

**Example:**

```text
/os-diff mindset-os 2.1.0 3.0.0
```
