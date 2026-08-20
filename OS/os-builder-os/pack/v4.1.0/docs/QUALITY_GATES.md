# Quality gates

## Gate groups

### A. Domain intelligence

- Scope covers major subdomains.
- Target users and outcomes are explicit.
- Glossary resolves ambiguous terms.
- Risks and non-goals are present.

### B. Corpus quality

- Bestseller discovery completed.
- Foundational, evidence-led, practitioner, specialist, current and critical lenses considered.
- Retained corpus is diverse and non-redundant.
- Every retained book has a complete deep analysis.
- Rejected books have logged reasons.

### C. Evidence integrity

- Material claims have provenance.
- Empirical, procedural, normative and design claims are separated.
- Confidence and applicability are explicit.
- Current claims use current sources.
- Contradicting evidence is represented.
- Access limitations are disclosed.

### D. Synthesis quality

- Concepts are normalized.
- Schools and contradictions are mapped.
- Universal, conditional, disputed, outdated and rejected claims are separated.
- Principles include mechanism, conditions and limits.
- Rules include triggers, actions, exceptions and evidence.

### E. System quality

- Capabilities map to jobs and outcomes.
- Commands map to capabilities and workflows.
- Workflows define state, branches, recovery and completion.
- Feedback and update loops exist.
- Boundaries and escalation are implementable.
- External dependencies have graceful degradation.

### F. Evaluation quality

- Critical scenarios have tests.
- Deterministic validators pass.
- Rubric-based evals meet threshold.
- Adversarial scenarios pass.
- Regression suite passes after repairs.

### G. Release quality

- Manifest, implementation and documentation agree.
- HOW_TO_USE is complete.
- `/presentation-os` includes every command with syntax, purpose, timing and examples.
- Versioned ZIP validates.
- Registry update is complete.
- Known limitations and accepted risks are disclosed.

## Scoring rubric

Score each dimension from 0 to 5:

1. Domain coverage
2. Source quality
3. Corpus diversity
4. Book-analysis completeness
5. Claim traceability
6. Contradiction handling
7. Synthesis coherence
8. Operationalization
9. Command and workflow completeness
10. Evaluation strength
11. Safety and boundaries
12. Usability
13. Maintainability
14. Updateability

### Release thresholds

- No dimension below 3.
- Critical dimensions 4, 5, 8, 9, 10 and 11 must score at least 4.
- Overall average must be at least 4.
- No unresolved critical or high-severity finding.

Scores support review. Passing scores do not override a failed hard gate.
