# Workflow: Clear a release

**Modes:** `MODEL`, `REVIEW`, `TEST`, `SUPPLY`, `PRIVACY`, `HARDEN`,
`CLEARANCE`
**Produces:** the security clearance handed to Release {OS}, with its
conditions, its residual risk and its untested surface named.

## Trigger

Quality & Evaluation {OS} issued a verdict on a pinned build and the release is
in view. Also triggered on any release that changes authentication, data
handling, permissions, dependencies or an AI tool surface, even a small one.

## Preconditions

- The written scope exists: assets, environments, authorised techniques, and
  the person who authorised them.
- The build is pinned, with its dependency manifest and build provenance.
- A non-production environment representative enough to test against, or
  written authorisation for production.

## Steps

1. **Confirm scope before anything else.** No scope, no assessment. Record who
   authorised it and what is explicitly out.
2. **Model the threat against the build as it is.** Entry points, trust
   boundaries, assets, actors, abuse cases. Record threats considered and
   dismissed, with reasons.
3. **Review the critical paths.** Auth, permissions, secrets, input handling,
   crypto, data access, tool invocation. Name the critical paths not reviewed.
4. **Run the secrets scan first.** It is the cheapest and most damaging class.
   A live hit switches to the live-secret workflow immediately.
5. **Test each planned attack class.** Record successes with reproductions,
   failures as tried and held, and blocked attempts with the blocker named.
6. **Attack the AI surfaces** where the product has a model reachable by
   untrusted input: injection, tool misuse, exfiltration, tenancy leakage.
7. **Inventory the supply chain.** SBOM for this fingerprint, origins, pinning,
   advisories, build integrity.
8. **Map the personal data flows** to purpose and retention, against the
   Blueprint data governance requirements.
9. **Refute your own findings.** Rule out mocks, fixtures, local-only
   configuration and existing mitigations before asserting anything.
10. **Prioritise and route.** Exploitability times impact. Each finding becomes
    a restricted Stepper step for Builder {OS} with the fix named.
11. **Retest after remediation.** The original reproduction must fail.
12. **Issue the clearance.** `CLEARED`, `CLEARED WITH CONDITIONS` (owner per
    condition) or `BLOCKED`, always naming the untested surface. Hand it to
    Release {OS}.

## Completion test

By inspection of the security record:

- the written scope is recorded with its authoriser;
- every entry point and trust boundary in the threat model has an assessment
  verdict, including untested;
- every planned attack class is recorded as succeeded, held or blocked;
- every asserted finding has a reproduction and a refutation attempt;
- an SBOM exists for the pinned build fingerprint;
- every finding is fixed and retested, or accepted by a named owner;
- the clearance names its conditions, its residual risk and its untested
  surface.

A `CLEARED` verdict with no untested surface listed fails this test, because no
assessment covers everything and claiming otherwise is the most misleading
output this OS can produce.

## Failure paths

| What happens | What the workflow does |
|---|---|
| the scope will not be written down | stop, escalate, do not proceed on implied permission |
| an environment cannot be reached | record that surface untested, name the blocker and its owner |
| a finding cannot be reproduced | downgrade to a suspicion with its conditions, do not report it as a vulnerability |
| remediation needs a product change | raise a decision request to Blueprint {OS} rather than accepting silently |
| the release is imminent and a finding is exploitable | `BLOCKED`, escalate to the named risk owner, never soften severity for a schedule |
| someone wants the finding published before the fix | refuse, keep distribution restricted, and route disclosure through the approval boundary |
