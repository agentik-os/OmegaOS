# Inter-OS orchestration

## Canonical construction chain

```text
Builder {OS}
  → Research {OS}
  → Librarian {OS}
  → Builder {OS} synthesis compiler
  → Quality & Evaluation {OS}
  → Review & Governance {OS}
  → Documentation {OS}
  → Release {OS}
```

## Role boundaries

### Research {OS}

Owns discovery, retrieval, verification, extraction, triangulation, citation and evidence freshness.

### Librarian {OS}

Owns book discovery, retained-corpus analysis, comparison, tutoring and book-derived knowledge artifacts.

### Builder {OS}

Owns the build contract, synthesis, domain ontology, executable logic, architecture and implementation coordination.

### Quality & Evaluation {OS}

Owns eval design, test execution, graders, regression and quality reports.

### Review & Governance {OS}

Owns audit, safety, permissions, user control, boundary compliance and accepted-risk decisions.

### Documentation {OS}

Owns user documentation and full command reference, but cannot invent capabilities not present in the manifest.

### Release {OS}

Owns semantic versioning, packaging, immutable release artifacts and registry publishing.

## Handoff envelope

Every handoff contains:

```yaml
handoff_id: unique-id
from_os: builder-os
to_os: librarian-os
capability: corpus.deep_analysis
input_schema: book_analysis_request.v1
output_schema: book_analysis.v1
reason: retained corpus analysis
user_visible: true
disableable: true
trace_id: build-trace-id
on_failure: queue_and_continue_other_independent_work
```

## Standalone rule

The final OS may use optional neighboring OSs, but its core value must remain usable without them.

## Failure behavior

When a neighboring OS is unavailable:

1. report the missing capability;
2. use a local fallback only when it meets the same contract;
3. lower confidence where appropriate;
4. preserve queued work;
5. never silently omit the stage.
