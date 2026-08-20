# Specialized agent roster

## 1. Build Orchestrator

Owns build state, milestones, handoffs, validation and completion.

Must not perform all specialist work itself when parallelization improves independence or coverage.

## 2. Domain Framer

Maps scope, jobs, outcomes, boundaries, actors, maturity and terminology.

## 3. Bestseller Scout

Discovers influential, foundational, current and practical books across languages and schools.

## 4. Corpus Curator

Scores candidates, deduplicates, builds coverage matrix and selects the retained corpus.

## 5. Book Deep Analyst

One isolated worker per book or small batch. Produces schema-valid extraction without seeing other first-pass conclusions.

## 6. Evidence Researcher

Finds primary, official, systematic, technical, field and counter-evidence beyond books.

## 7. Claim Auditor

Normalizes claims and checks provenance, access, confidence, scope and unsupported inference.

## 8. Contradiction Analyst

Maps schools, assumptions and disagreements. Converts resolvable conflicts into conditional logic and preserves unresolved ones.

## 9. Knowledge Synthesizer

Builds mechanisms, principles, conditions, rules and open questions from validated claims.

## 10. Ontology Architect

Defines canonical entities, states, events, relations and glossary.

## 11. OS Architect

Designs capabilities, command families, workflows, graphs, loops, memory, permissions and handoffs.

## 12. Workflow and Command Engineer

Implements contracts, state transitions, errors, examples and registry entries.

## 13. Eval Engineer

Builds deterministic tests, scenario datasets, graders, thresholds and regressions.

## 14. Red Team Auditor

Attacks assumptions, sources, logic, safety, dependencies and misuse resistance.

## 15. Documentation and Release Engineer

Generates coherent documentation, presentation, changelog, package, checksums where requested, and registry update.

## Merge rules

- Specialists write structured artifacts.
- The Orchestrator validates schema before merge.
- Synthesis cannot start while retained-book analyses are incomplete.
- Architecture cannot start before synthesis and ontology pass.
- Documentation cannot invent unimplemented commands.
- Release cannot bypass critical gates.
