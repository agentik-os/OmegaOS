# Workflow: Frozen handoff packet

**Produces:** an immutable versioned concept plus the packet the named
downstream OS needs, containing only the fields that unit consumes, and the
emitted event.

## Trigger

A session reached `BRAINSTORM CONVERGED: HANDOFF READY` and the concept is about
to leave this OS. Also triggered on `/freeze` and `/handoff`.

## Steps

1. **Run the audit first.** Run `AUDIT` mode (or `brainstorm_os.py audit
   --require-pass`) over the session: diversity, dissent, evidence discipline,
   traceability, decision quality and handoff readiness. A red gate stops the
   freeze. Overriding a red gate is a human decision, it is recorded with its
   author and reason, and the packet states that it was overridden.
2. **Check for false convergence.** Confirm that the selected concept beat a
   genuine alternative and not a strawman, that at least one cell dissented at
   some point, and that the tension map is not empty. An empty tension map after
   a real council is evidence of averaging, not of agreement.
3. **Close the open questions.** Every `BS-QUE` must be resolved, deferred with
   a reason, or explicitly listed in the packet as an open dependency. A
   critical open question blocks the freeze.
4. **Verify the lineage.** Every artifact in the packet traces to its parents:
   idea to genome to frame. No recycled ID, no orphan, no superseded item
   presented as current.
5. **Freeze.** Version the selected concept (`brainstorm_os.py freeze --level
   patch|minor|major --converged`). A freeze creates a new version; it never
   overwrites a prior one. Both stay readable, with the reason for the change.
   This step requires human approval.
6. **Choose the target and cut the packet to it.** Include only what that unit
   consumes:
   - **research**: the concept statement, the hypotheses (`BS-HYP`) that depend
     on external facts, each with its falsifier and why it matters, and the
     decision each one unblocks. No internal debate transcript.
   - **blueprint**: the concept statement, the locked decisions (`BS-DEC`), the
     surface decision (`BS-SRF`) with its rationale, the constraints, the
     non-goals, and the confirmed-versus-hypothesis split stated in full. No
     screens, no schemas, no endpoints: defining those is Blueprint's job.
   - **decision**: the survivors, the criteria and weights, the sensitivity of
     the ranking, and the tensions that a human must adjudicate.
   - **creative brief**: the concept, the Founder DNA that shapes its voice, the
     signature tension, and the anti-patterns.
7. **Label everything in the packet.** `EVIDENCE`, `DECISION`, `HYPOTHESIS`,
   `ASSUMPTION`, `CONSTRAINT`, `UNKNOWN`, `CONFLICT`. A downstream unit that
   cannot tell a decision from an assumption will treat both as fact.
8. **State what was skipped.** When the target is `blueprint` and Market
   Research {OS} or Validation {OS} is being bypassed, say so in the packet, in
   plain words, and name who authorized it. This step requires human approval
   and is never implicit.
9. **Emit.** `brainstorm.concept.selected` to Market Research {OS}, Validation
   {OS} or Blueprint {OS} as chosen, and `brainstorm.session.completed` to
   Context & Memory {OS} with the full ledger and lineage.
10. **Leave the resume point.** Record the current round, the completed
    artifacts, the next exact action, the remaining challenges, the founder
    decisions still needed, and the state checksum, so a later session resumes
    rather than restarts.

## Completion test

- The audit ran and every gate passed, or a red gate was overridden by a named
  human with a recorded reason that appears in the packet.
- No critical `BS-QUE` is open.
- The concept has a version number and a lineage that resolves to a frame. No
  prior version was overwritten.
- The packet contains only the fields the named target consumes, and a reader
  from that unit can start without asking a clarifying question.
- Every statement in the packet carries a label, and the confirmed-versus-
  hypothesis split is explicit.
- No external fact is asserted anywhere in the packet.
- Where a `blueprint` target skips Market Research {OS} or Validation {OS}, the
  packet says so and names the authorizing human.
- Both events were emitted, and the canonical ledger in Context & Memory {OS}
  matches what was handed over.
- The session ends on exactly one of `BRAINSTORM CONVERGED: HANDOFF READY`,
  `BRAINSTORM IN PROGRESS`, `BRAINSTORM BLOCKED` or `BRAINSTORM PARKED`.
