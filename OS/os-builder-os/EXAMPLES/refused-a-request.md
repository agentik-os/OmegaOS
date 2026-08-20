# Example: refusing to build an OS

The pipeline stopping at phase 1 and delivering the right smaller thing. This
is a walk-through, and it is the more common outcome: most requests that reach
OS Builder should not become an OS.

---

## The request

> "Every time we onboard a client we forget something. Build me a Client
> Onboarding {OS}."

The request arrives pre-solved. Someone has already decided the answer is an
OS, has picked a name, and has skipped the question this OS exists to ask.

---

## Phase 0. Intake

`00-intake.md` runs unchanged. Refusing later does not license a shallower
intake now: the refusal has to be defensible, and a defensible refusal needs
the same record a build does.

```
Name:                  Client Onboarding
Capability:            Make sure nothing is missed when a new client
                       starts.                                       [stated]
Primary operator:      Whoever runs the onboarding, currently two
                       different people depending on the week.        [stated]
Target environment:    Claude, and a shared document the team reads.  [stated]
Business problem:      Steps get forgotten. The forgotten step is
                       usually a different one each time.             [stated]
Desired outcome:       Nothing is forgotten.                          [stated]
Primary artifact:      A completed onboarding, and a record of it.    [inferred]
Upstream systems:      The signed contract.                           [stated]
Downstream systems:    Delivery.                                      [stated]
Shared systems:        None.
Constraints:           Two people must be able to run it identically. [stated]
Research depth:        None required. The steps are already known and
                       written down in two places.                    [stated]
Security sensitivity:  Internal. Client contact details, no regulated
                       data.                                          [stated]
Required modes:        One.                                           [inferred]
Packaging target:      Whatever is smallest.                          [stated]
```

The three fields that decide this are already visible. Research depth is none.
Required modes is one. The steps are already written down.

**Blocking questions: zero.** Nothing here would change the answer, and asking
anyway would be billing the operator for the builder's uncertainty.

---

## Phase 1. Viability

The tree, node by node, answered rather than jumped.

```
Repeatable professional capability?
  yes. It happens every time a client signs, and it is the same work.

Recurring decisions, workflow, or artifacts?
  Workflow: yes.
  Artifacts: yes, one record per client.
  Decisions: NO. Nothing is being decided. The steps are known, their
  order is known, and no step's answer depends on judgement. The
  reported failure is not a wrong decision, it is a missed step.
```

This is the node that decides it, and it is the node people skip because the
first two answers were yes.

```
    -> USE A LIGHTER ARTIFACT
```

**Verdict: `USE A LIGHTER ARTIFACT`. The artifact is a checklist.**

---

## Why an OS would be the wrong answer here

Stated for the requester, because a refusal without a reason reads as a
refusal to work.

1. **There are no recurring decisions.** An OS encodes judgement under
   uncertainty. This capability has no uncertainty: it has memory failure.
   Those need different tools, and a checklist is the tool for the second.
2. **There is no evidence problem.** Nothing needs to be researched, sourced,
   weighed or reconciled. Six of the sixteen rubric dimensions would score
   zero, not because the unit was built badly but because they do not apply.
3. **It would cost more to maintain than to run.** Twenty-three contract files
   would need updating every time a step changes. A checklist needs one line
   edited. The team changes its steps more often than it onboards.
4. **It would be worse at the actual job.** The failure is at 4pm on a Friday
   when someone is tired. What fixes that is a list in front of them, not a
   conversation with a model.
5. **The suite has a home for this already.** Recurring behaviour contracts and
   the evidence they happened is `habit-tracker-os`. Repeatable operational
   procedure is `process-sop-os`. Building a third would create two more
   answers to one question.

---

## What was delivered instead

Not "no". A refusal that ends at "no" has produced nothing, and the requester's
problem is still there at 4pm on Friday.

**An onboarding checklist**, drafted from the two places the steps were already
written down, reconciled where those two disagreed (they disagreed on three
steps, which is itself part of why things were being forgotten), ordered by
dependency rather than by habit, with the owner named per step and the evidence
that closes each one.

**Two structural notes**, because the reconciliation surfaced them:

- Three steps existed in one document and not the other. Whichever document a
  person opened decided what they did. That is the missed-step mechanism, and
  it was never a tooling problem.
- Two steps had no owner in either document. An unowned step is missed
  systematically rather than occasionally.

**One handoff.** If the checklist is still missed after a month, the failure is
not the artifact, and the request that comes back is a different one:
`process-sop-os` for enforcement, or `execution-os` if the real problem is that
nobody is tracking the work at all. Named now, so the requester knows where to
go without re-entering here.

---

## What this run proves

The pipeline stopped at phase 1 of 15, produced a usable artifact, and cost
about twenty minutes. That is a successful run.

An OS Builder that returns `BUILD` on every request is not deciding anything,
and the suite pays for that in units nobody uses and files everybody has to
maintain. The four non-BUILD leaves of the viability tree, `NOT A CAPABILITY`,
`USE A LIGHTER ARTIFACT`, `SPLIT` and `ALREADY COVERED`, are outputs, not
failures.

The one thing this run must never do is build the OS anyway because it was
asked to by name.
