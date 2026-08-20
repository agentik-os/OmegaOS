# Workflow: Handle an integration failure

Contain a failing external call, decide honestly whether a retry is safe, and
turn the failure into a contract that is one class better.

## Trigger

- A call failed, timed out, or returned an error class the contract does not
  cover.
- Authentication started failing.
- A response shape changed under a working consumer.
- A rate limit or quota was exhausted.
- An upstream deprecation notice arrived.

## Steps

1. **Classify the failure before acting.** Unreachable, unauthorised, rejected
   input, rate limited, upstream error, timeout with unknown state. These need
   different responses, and treating them all as "it failed" produces a blind
   retry.
2. **Report an unreachable or unauthorised surface as a failure.** Never as an
   empty result, never as a pass, and never by finding another path with wider
   permissions.
3. **On a timeout, do not retry a non idempotent operation.** The state is
   unknown, and a retry is a decision to risk a second effect. Escalate instead,
   and say that the state is unknown rather than guessing.
4. **On a retryable class, apply the contract's ceiling,** then escalate. An
   unbounded retry is a slow outage that also generates a bill.
5. **On a rate limit, report which consumers exhausted the quota,** not only that
   it was exhausted. A shared quota failure is usually somebody else's volume.
6. **On an unknown error class, add it to the contract** with a defined response
   before anything resumes. Resuming without defining it guarantees the same
   incident again.
7. **On a response shape change, raise drift.** Never silently adapt the parser:
   the adaptation hides the change from every other consumer, and the next change
   is discovered by an incident rather than a report.
8. **Notify every registered consumer** of the affected operations, with the date
   and the deprecation timeline where one exists.
9. **Re-probe the affected operations** and update the contract, including
   idempotency and limits, which frequently change together with a shape change.
10. **Check the grants.** A failure sometimes reveals a consumer calling an
    operation it was never granted; deny it, log it, and review the grant.
11. **Stage the drift event and the updated contract** to Context & Memory {OS},
    and hand the consumer facing consequences to Automation {OS} or Orchestration
    {OS}, which own what the process does about it.

## Completion test

- The failure carries exactly one classification.
- No unreachable or unauthorised result was reported as a pass or an empty
  result.
- No non idempotent operation was retried after a timeout.
- Retries respected the contract's ceiling and ended in an escalation.
- Any new error class is in the contract with a defined response.
- A response shape change raised drift and was not absorbed silently by a parser.
- Every registered consumer of the affected operations was notified, with dates.
- The affected operations were re-probed and the contract updated.
- Grants were checked against what was actually called.
- The drift event and updated contract are staged to Context & Memory {OS}.

The output of this workflow is a contract that now covers the case that broke it.
Restoring the call without that is not a fix, it is a pause.
