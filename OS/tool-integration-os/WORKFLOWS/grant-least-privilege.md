# Workflow: Grant least privilege

Give a named consumer exactly the operations it needs, for a bounded time, with
approval where the consequence demands it.

## Trigger

- Agent {OS} computed a tool grant from a brief.
- Automation {OS} needs an external system for a blueprint step.
- An existing grant is expiring.
- A review found grants nobody can justify.

## Steps

1. **Require a named consumer.** Not a team, not a system, not "the pipeline". A
   grant with no named consumer cannot be reviewed or revoked meaningfully.
2. **Require the justification per operation:** the brief step or blueprint step
   that needs it. An operation with no step behind it is not granted, however
   convenient it would be.
3. **Start from nothing and add.** Never start from an existing grant and trim;
   trimming preserves whatever the previous grant got wrong.
4. **Check the consequence class of each requested operation.** Read operations
   proceed. Anything that sends, publishes, pays, deletes or signs stops here for
   an explicit human approval, and the approval names the operation, not the
   system.
5. **Check for customer or personal data** in the scope. If present, route to
   Review & Governance {OS} before issuing.
6. **Set an expiry.** Every grant has one. Permanent grants accumulate until
   nobody can say who may do what, and that state is discovered during an
   incident.
7. **Scope the credential** to the operations granted. A credential broader than
   the contract requires is refused, and the refusal names the narrower scope
   that would be accepted.
8. **Record where the credential is configured,** and confirm that the grant
   record contains no value.
9. **State the shared quota position:** what this consumer's expected volume does
   to the limits other consumers depend on.
10. **Issue the grant,** notify the consumer of the failure semantics it is
    accepting, and stage the grant to Context & Memory {OS}.
11. **Schedule the review** at expiry, with the question: was every granted
    operation actually used?

## Completion test

- The grant names one consumer.
- Every granted operation has a justifying step.
- The grant was built additively, not by trimming a previous one.
- Every send, publish, pay, delete or sign operation carries a recorded human
  approval naming that operation.
- Scope touching customer or personal data carries a Review & Governance {OS}
  approval.
- The grant has an expiry and a scheduled review.
- The credential scope is no wider than the granted operations.
- No credential value appears in the grant record.
- The consumer has been told the failure semantics it must handle.
- The grant is staged to Context & Memory {OS}.

At review, any operation that was never used is removed. Grants only ever grow
unless something removes them, and this is that something.
