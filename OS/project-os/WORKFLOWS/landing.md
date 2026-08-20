# Workflow: landing the project

Produces acceptance by the requester, a closeout record, and a correction to
how the next project is estimated.

## Trigger

The done test appears to be met. The same workflow runs, with the abort branch,
when the project must stop before the done test is met.

## Inputs

- The scope statement with its done test and out-of-scope list.
- The final milestone plan and the archived superseded plans.
- Every change record.
- The deliverable itself.

## Steps

1. **Apply the done test literally.** Not approximately, not in spirit. If it
   fails, the project is not landed, and the recovery options open instead.
2. **Get acceptance from the requester in writing.** The team does not accept
   its own work.
3. **List what shipped and what was cut.** The cut list matters more than the
   shipped list, because it is what people misremember.
4. **Compute actual versus planned.** Duration, cost, and the number of change
   records. Do not soften any of the three.
5. **Hand over.** Whoever operates or maintains the result gets the artifacts,
   the access and the known defects, through Documentation {OS} and, where the
   result becomes recurring work, Process & SOP {OS}.
6. **Close the open ends.** Open commitments in Execution {OS}, open work
   packages in Team & Delegation {OS}, and the client's expectations in Client
   {OS} are all explicitly closed.
7. **Write the estimate correction.** One sentence: what we now know we
   underestimated, and by how much. It goes to Context & Memory {OS}.
8. **Send the retro input to Review & Governance {OS}.** Evidence, not opinion.
9. **Archive the project.** Superseded plans stay archived, never deleted.

### Abort branch

If the project stops before the done test: record the stop decision and the
decider, name what is salvaged and where it goes, name what is written off, and
tell everyone who was holding an expectation. An abort is closed with the same
rigour as a landing.

## Completion test

- The done test was applied literally and the result is recorded.
- The requester has accepted in writing, or the abort decision is recorded with
  its decider.
- The closeout record lists what shipped, what was cut, and actual versus
  planned.
- Handover happened: artifacts, access and known defects are with their new
  owner.
- No commitment, work package or client expectation from this project is still
  open anywhere.
- One estimate correction is written and stored.

## Failure paths

| Situation | Response |
|---|---|
| the requester will not sign off but keeps using the deliverable | record use as de facto acceptance, state it plainly, and escalate through Client {OS} |
| the done test was quietly changed during the project | land against the original test, and record the change of test as the main finding |
| known defects remain | hand them over as a named list with owners; never let them dissolve into the closeout narrative |
| nobody wants to own the result after handover | do not close; escalate to Review & Governance {OS}, because an unowned deliverable becomes an operations problem later |
