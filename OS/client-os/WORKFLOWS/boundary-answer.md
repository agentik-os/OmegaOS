# Workflow: the boundary answer

Produces one of exactly three answers to an out-of-scope request, plus the
record that keeps the boundary from moving silently.

## Trigger

The client asks for something the expectation record does not contain,
including a request that sounds small and a request from someone senior.

## Inputs

- The expectation record and its exclusion list.
- The current plan and remaining capacity, from Project {OS}.
- The price model, from Pricing {OS}.
- Every exception previously granted to this client.

## Steps

1. **Check it against the record.** If it is inside the agreed scope, it is not
   a boundary question. Say yes and continue.
2. **Price it if it is outside.** Time, money, and the effect on the landing
   date, from Project {OS}. Never price from instinct in a live conversation.
3. **Check the precedent.** Look at exceptions already granted to this client.
   Three free extras in a row have already redefined the agreement, whatever the
   contract says.
4. **Choose one of the three legitimate answers:**
   - Included: confirm, and point to where the record says so.
   - Extra: give the price and the date effect, and open a change record in
     Project {OS}.
   - Refused: give the reason and an alternative that serves the underlying
     need. A refusal with no alternative reads as unwillingness.
5. **Do not choose the fourth option.** Doing it quietly for free without saying
   so is what dissolves the boundary, and it is invisible until renewal.
6. **If it is granted as a goodwill exception, say that it is one.** Name what
   it would normally cost, and record it dated.
7. **Get human approval and send the draft.**
8. **Record the outcome** in the client ledger, and in Project {OS} if the plan
   moved.

## Completion test

- The request was checked against the expectation record before being answered.
- The answer is one of the three: included, extra with a price, refused with an
  alternative.
- Anything granted outside scope is recorded as a dated exception with its
  normal price.
- Any accepted extra has a change record in Project {OS} before work starts.
- A human approved the message before it was sent.

## Failure paths

| Situation | Response |
|---|---|
| the request arrives verbally mid-call | acknowledge, promise an answer by a named time, and price it out of the room |
| the team has already started doing it | stop, price it, and tell the client it was started in good faith and now needs a decision |
| the client escalates over a refusal | hold the boundary, bring the decider, and offer the paid path and the alternative again |
| exceptions to this client are piling up | run the health read: repeated exceptions are usually a mispriced or misscoped engagement, not a generous relationship |
