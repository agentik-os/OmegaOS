# Workflow: the late promise

Produces a renegotiated date agreed before the deadline, or an explicit, dated
decision to accept the consequence of being late.

## Trigger

A promise in the ledger reaches its notice-by date and its next action is not
startable, or the owner states it will be missed. The notice-by date always
precedes the deadline; this workflow never runs after the fact.

## Inputs

- The promise ledger entry: who, what, by when, notice-by, consequence.
- The current state of the underlying commitment.
- The relationship record from Client {OS} if the promise is to a client.

## Steps

1. **Verify it will actually be missed.** Look at the remaining work and the
   remaining protected blocks. A promise that still fits is not late.
2. **Name what can still land by the original date.** Full delivery, a reduced
   scope, or a partial with the rest dated. Prefer a smaller true delivery over
   a whole late one.
3. **Set the new date from capacity, not from hope.** The proposed date is
   backed by protected blocks that already exist in the plan.
4. **Write the notice.** Three parts and no apology theatre: what will happen,
   by when, and what the other person needs to do differently because of it.
5. **Get human approval before sending.** Execution {OS} drafts. The user, or
   Client {OS} for a client relationship, sends.
6. **Record the outcome.** New date accepted, new date refused, or consequence
   accepted. All three are legitimate; none of them is silence.
7. **Update the commitment.** New deadline, new next action, and a protected
   block that makes the new date real.
8. **Feed the weekly reset.** A missed promise is friction data, and repeated
   misses to the same person are a boundary problem for Client {OS}, not a
   scheduling problem.

## Completion test

- The notice was sent on or before the notice-by date, never after the deadline.
- The new date is backed by blocks that exist in the plan.
- The ledger entry records the outcome: accepted, refused, or consequence taken.
- The underlying commitment has a new deadline and a physical next action.

## Failure paths

| Situation | Response |
|---|---|
| the notice-by date has already passed | send the notice immediately, and record the lateness of the notice itself as the finding |
| the same person has been renegotiated with twice already | escalate to Client {OS} or Team & Delegation {OS}: this is a commitment-sizing problem, not a date problem |
| the user wants to stay silent and hope | refuse to help draft a silent path, state the consequence, and record the decision if they proceed |
