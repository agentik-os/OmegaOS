# Workflow: the account health read

Produces an evidence-backed read on each live account, and a named repair for
anything that is not green.

## Trigger

The review cadence fires, or a strain signal appears: silence through a full
cadence period, a change of tone, an escalation, a late payment, a sudden
increase in scope pressure.

## Inputs

- The client ledger: promises, exceptions, communication log.
- Project position and slip history, from Project {OS}.
- Support and delivery signals, from Delivery & Customer Success {OS}.
- Payment status, from Revenue {OS}.
- Who on the client side has stopped replying, and who has started copying
  other people in.

## Steps

1. **Gather the signals before forming an opinion.** Communication gaps, slip
   history, exception count, support load, payment behaviour, and changes in who
   is in the thread.
2. **Read each signal against the agreement**, not against a feeling. Silence
   from a client who bought a quarterly engagement is not the same as silence
   from a weekly one.
3. **Classify: green, watch or at risk.** Green means the agreement is being met
   and the client behaves as expected. Watch means one signal is off. At risk
   means the relationship or the revenue is genuinely in question.
4. **Name the cause for anything not green.** Delivery, expectation,
   communication, price, or relationship. These need different repairs and are
   routinely confused.
5. **Choose one repair action** with an owner and a date, observable by the
   client. A repair the client cannot observe does not repair anything.
6. **State the risk of loss** in plain terms, including the revenue at stake,
   and send it to Revenue {OS} and Sales {OS}.
7. **Check the exception count.** An account carrying many exceptions is
   mispriced or misscoped; that finding goes to Pricing {OS} and Offer {OS}.
8. **Escalate the systemic ones.** If three accounts show the same cause, it is
   not a client problem, and it goes to Review & Governance {OS}.
9. **Record the read** with its evidence, and set the next review date.

## Completion test

- Every live account has a current read with a date.
- Every read cites its signals rather than an impression.
- Every non-green account has a named cause, one repair action, an owner and a
  date.
- The revenue at risk is stated for anything at risk.
- Causes appearing across three or more accounts are escalated as systemic.

## Failure paths

| Situation | Response |
|---|---|
| the signals are ambiguous | ask the client a specific question rather than a general one; "are you happy" produces no information |
| the account is green but the user feels uneasy | record the read on evidence, note the unease as a watch item, and set an earlier review date |
| the cause is price | do not repair it with free work; route to Pricing {OS} and Offer {OS} and have the real conversation |
| the relationship is unrecoverable | say so with the evidence, and run the close workflow deliberately rather than letting it fade |
