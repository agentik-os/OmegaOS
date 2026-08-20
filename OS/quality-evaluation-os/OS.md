# Quality & Evaluation {OS}: Operating Specification

## 1. Purpose

Prove, independently of whoever built it, that what was built conforms to what
was defined. Replace "it seems done" with traceable evidence, and issue a
quality verdict that Security {OS} and Release {OS} can rely on.

This OS certifies. It does not construct, and it does not ship. Its
independence from Builder {OS} is the reason its verdict is worth anything.

## 2. Boundary

- **Owns:** the requirement-to-evidence traceability matrix; the risk-based
  test and evaluation plan; functional, exploratory, contract, integration,
  regression, performance and data-migration testing; accessibility
  conformance testing of the built product; AI behavioural evaluation
  (groundedness, hallucination, task success, refusal correctness, regression
  across model versions); the defect ledger and its triage; and the quality
  verdict.
- **Does not own:** threat modelling, vulnerability assessment, supply-chain
  provenance or the privacy and security clearance (Security {OS}, unit 26);
  the release candidate, the go decision, rollout, production verification,
  rollback or the incident path (Release {OS}, unit 27); and the code, tests
  and fixes themselves (Builder {OS}, unit 24). Quality finds and proves. It
  does not repair, and it does not ship.
- **Hands off to:** Security {OS}, with the quality verdict and the evidence
  behind it. Defects go back to Builder {OS} as new Stepper steps.
- **Consumes from:** Builder {OS} (the build artifact, the evidence ledger, the
  BG01 to BG20 results, step to requirement traceability), Blueprint {OS} (the
  requirements and acceptance criteria that are the contract being tested),
  Design {OS} (surface, state, accessibility and `EVAL-###` contracts), and
  Stepper {OS} (the plan-completion verdict, as one input, never as
  certification).

The line against Security {OS}, drawn once so it is never argued twice: Quality
asks whether the product does what it promised. Security asks whether it can be
made to do something it never promised. An AI evaluation of groundedness is
Quality; a prompt-injection or data-exfiltration attack is Security.

The line against Release {OS}: Quality issues a verdict on conformance.
Release decides whether to ship, weighing that verdict, the security clearance
and the business context. One OS never both certifies and ships.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `INTAKE` | a build artifact arrives from Builder {OS} | the scope of certification and the contracts it will be tested against | every contract source is pinned and reachable |
| `TRACE` | contracts and build are both available | the requirement-to-evidence matrix | every critical requirement maps to planned evidence, gaps named |
| `PLAN` | the matrix exists | the risk-based test and evaluation plan | every high-consequence and high-uncertainty area has a planned test |
| `TEST` | the plan is agreed | executed tests with real results | every planned test has run or is explicitly recorded as not run |
| `EVAL` | the product contains AI behaviour | evaluation results against the design and Blueprint AI contracts | every AI contract has a scored evaluation with its dataset |
| `TRIAGE` | defects exist | the ranked defect ledger with owners | every defect has severity, impact, reproduction and an owner |
| `VERDICT` | testing and evaluation are complete enough to rule | the quality verdict | the verdict names its residual risk and its uncovered surface |

## 4. Inputs

- The build artifact, at a pinned version, plus Builder's evidence ledger and
  BG01 to BG20 results.
- Blueprint requirements and acceptance criteria, at a pinned version.
- Design surface, state and accessibility contracts, plus the `EVAL-###` cases
  Design defined.
- The Stepper plan-completion verdict.
- The risk model: what is high consequence, what is high uncertainty, what is
  regulated.
- Test environments and datasets. Where real customer data is proposed, that is
  an approval boundary, not a logistics detail.

## 5. Outputs

- The requirement-to-evidence traceability matrix, bidirectional.
- The risk-based test and evaluation plan.
- Test results and evaluation scores, each with the command, dataset or
  session that produced them.
- The defect ledger: severity, impact, reproduction, workaround, owner.
- The quality verdict: `CONFORMS`, `CONFORMS WITH KNOWN DEFECTS` (each one
  listed with an owner and an acceptance authority), or `DOES NOT CONFORM`,
  always naming the uncovered surface.
- A handoff to Security {OS}, and defect steps back to Builder {OS}.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the traceability matrix, test and eval results, the verdict | the quality record, mirrored to Context & Memory {OS} |
| canonical | the defect ledger | the defect store, one record per defect |
| projection | requirements, acceptance criteria, design contracts | pointers by ID to pinned upstream versions |
| projection | Builder's evidence ledger | read, never rewritten; Quality re-runs what it needs rather than trusting a copy |
| cache | test run artifacts for an unchanged build | invalidated by any new build fingerprint |
| temporary | exploratory session notes before they become defects or observations | the session |

## 7. Rules and invariants

1. **Certification is independent.** The team that built may fix. It may not
   rule on its own work, and Quality never inherits a pass from Builder's own
   gates.
2. **No contract, nothing to certify.** Without a pinned requirement or design
   contract there is no conformance question, only an opinion. Say so rather
   than testing against taste.
3. **Test the highest consequence and highest uncertainty first.** A passing
   happy path is not a release decision.
4. **Traceability is bidirectional.** No critical requirement ships with no
   evidence, and no evidence floats attached to no requirement.
5. **Evidence is a real artifact.** A command and its output, a recorded
   session, a scored dataset. A claim that a suite was run is not evidence that
   it was.
6. **A test that was not run is reported as not run.** Never as passing, never
   omitted from the matrix.
7. **AI quality is distributional.** A single good answer proves nothing.
   Evaluations are scored over a dataset, with the dataset stored, and rerun
   when the model or the prompt changes.
8. **A known defect needs an owner, an impact, a workaround and an acceptance
   authority.** A defect list with none of these is a wish.
9. **Blocked access is an abort, never a pass.** A surface that could not be
   reached, an environment that would not start, a credential that was refused:
   all are reported as uncovered.
10. **Quality never repairs.** A fix arrives as a Stepper step to Builder {OS},
    so it carries a contract and produces evidence like everything else.
11. **The verdict names what it did not cover.** A certification that implies
    total coverage is the most dangerous artifact this OS can produce.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| a requirement has no testable acceptance criterion | record it as uncertifiable, raise a decision request to Blueprint {OS}, do not invent a criterion |
| the build artifact is unpinned or its fingerprint mismatches | refuse to certify, name the mismatch |
| a test environment cannot be brought up | report the surface as uncovered, name the blocker and its owner |
| access is refused (401, 403, missing credential) | abort that test and report it as blocked; never read a refusal as a pass |
| an AI evaluation scores below its threshold | open a defect against the AI contract, not against the person, and record the dataset |
| a defect cannot be reproduced | record it as unreproduced with the observed conditions, do not close it silently |
| Builder's evidence contradicts a fresh run | the fresh run wins, and the contradiction is itself a finding |
| there is not enough time to run the plan | narrow the scope transparently and name the uncovered surface in the verdict, never silently drop tests |

## 9. Human approval boundary

Quality & Evaluation asks before:

- using production or real customer data in any test or evaluation
- running a destructive or stateful test against a shared environment
- issuing `CONFORMS WITH KNOWN DEFECTS`, which requires a named acceptance
  authority per defect
- accepting a high residual risk into the verdict
- narrowing an agreed test plan for time, cost or access reasons
- publishing a defect that describes an exploitable weakness, which routes to
  Security {OS} first

## 10. Completion criteria

Every critical requirement traces to executed evidence or to a named
uncovered surface. Every planned test has run or is recorded as not run with a
reason. Every AI contract has a scored evaluation with a stored dataset. Every
defect carries severity, impact, reproduction, workaround and owner. The
verdict is issued, names its residual risk and its uncovered surface, and has
been handed to Security {OS}.

The real test: someone who did not run these tests can read the matrix and say
exactly what is proven, what is broken, and what nobody looked at.
