---
name: builder-os
description: The implementation runtime: steps executed into release-ready code. Builder {OS}, unit 24 of the AGENTIK {OS} suite (03 · BUILD). Use when the user asks about builder or invokes /builder-os.
---

# Builder {OS}

The implementation runtime: steps executed into release-ready code.

## When to use this

Use Builder {OS} when:

- a Stepper graph is `BUILD READY` and code has to be written;
- an implementation session has to survive interruption, compaction and
  handover without losing what was actually done;
- several workers implement in parallel and their evidence must land in one
  ledger;
- a build has to be defensible later: which step produced this line, and what
  proved it worked.

Do not use it when:

- there is no step contract. Building without one produces a diff nobody can
  review against anything. Go to Stepper {OS}, and behind it Blueprint {OS}.
- the artifact is meant to be thrown away. That is Prototype {OS}, which is
  deliberately unreviewed and unsafe to ship.
- the question is whether the built thing is correct, safe or shippable. Those
  are Quality & Evaluation {OS}, Security {OS} and Release {OS}, and their
  independence is the point.

The near neighbour people confuse it with is Stepper {OS}. Stepper decides what
is next and rules on whether it is done. Builder implements. A Builder session
that starts keeping its own list of remaining work has forked the plan.

## Capabilities

- Verifies Blueprint and Stepper fingerprints before touching anything.
- Hydrates a claimed step: contract, Blueprint references, design references,
  dependency artifacts, prior failure evidence.
- Implements exactly one step contract, respecting its do not touch block.
- Records deterministic evidence: the command run and the output it produced.
- Runs the build gates BG01 to BG20 and reports each with evidence.
- Repairs against printed evidence under a bounded ceiling, then escalates.
- Reconciles a dirty working tree and resumes an interrupted attempt without
  destroying uncommitted work.
- Integrates verified work across parallel workers without discarding either
  side of a conflict.
- Produces the frozen final engineering and operations handoff.
- Routes definition conflicts upstream as decision requests.

## Procedure

1. **Open the session with state, not memory.** Load the manifest and Builder
   state, verify Blueprint and Stepper fingerprints, inspect git status,
   worktrees and locks, then `omega-stepper resume && omega-stepper status &&
   omega-stepper plan`.
2. **Finish what is open before claiming anything new.** An interrupted attempt
   is reconciled first.
3. **Claim one step.** Through Stepper, so the graph stays authoritative.
   Mirror it into Builder state with `sync-step` and `claim`.
4. **Hydrate.** Read the contract, the Blueprint and design references it
   cites, the artifacts it depends on, and every prior failed attempt.
5. **Preflight.** Does the contract still hold against the repository as it is
   now. A contradiction here is cheaper than one found halfway through.
6. **Micro-plan.** The files, the signatures, the order. Inside the contract
   only.
7. **Implement.** Exactly the contract. Discovered work becomes a new step.
8. **Verify with real commands.** Record each check with its real output
   through `record-check`. A green build in a narrative is not evidence.
9. **Repair against evidence, bounded.** At the ceiling, escalate with every
   attempt attached.
10. **Review, integrate, evidence, close.** Then let Stepper's verifier close
    the step, and mirror the verdict with `mark-step`.
11. **Gate and finalise.** `gate` for BG01 to BG20, and when Stepper's release
    check passes as well, `finalize` the frozen handoff and hand it to Quality
    & Evaluation {OS}.

## Handoffs

| Receives from | What arrives |
|---|---|
| Blueprint {OS} (20) | the frozen pack, verified by version and checksum |
| Design {OS} (21) | the surface, state and component contracts a UI step must satisfy |
| Prototype {OS} (22) | verdicts a step's approach rests on |
| Stepper {OS} (23) | the `BUILD READY` graph, the step contracts, the agent briefs, and the verifier that closes them |

| Hands to | What it expects |
|---|---|
| Quality & Evaluation {OS} (25) | the build artifact, the evidence ledger, BG01 to BG20 results, and step to requirement traceability |
| Blueprint {OS} / Design {OS} / Stepper {OS} | decision requests, whenever the definition rather than the code is wrong |

Defects return here from Quality & Evaluation {OS} and from Security {OS}. They
arrive as new steps in the Stepper graph, never as informal fixes, because a
fix with no step has no contract and no evidence.
