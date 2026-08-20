# Workflow: Publish a tool contract

Turn an external system into something a consumer can call with a shared
understanding of what happens when it goes wrong.

## Trigger

- An agent brief or an automation blueprint names an external system.
- An existing integration has no written contract.
- A drift report showed the contract no longer matches reality.

## Steps

1. **List the operations actually needed,** with the consumer step that
   justifies each. A contract covering a whole system rather than named
   operations is a grant nobody has thought about.
2. **Draft the contract from documentation** and mark it explicitly unproved.
3. **Probe the live system** for every operation in the draft. Real calls, real
   responses, recorded with the date. This step is what makes the contract more
   than a description of somebody's hopes.
4. **Record every error class observed,** including the ones documentation does
   not mention. Undocumented errors are the ones that reach production
   unhandled.
5. **Define the response per error class:** retry with a bounded ceiling, do not
   retry, escalate, or compensate. Leave none undefined; an undefined class
   becomes a blind retry, and a blind retry is how one failure becomes many.
6. **State idempotency per operation.** Where the system offers a key, record how
   it is formed and what window it covers. Where it does not, mark the operation
   as never safe to retry blindly and say so in the contract's most visible
   place.
7. **Measure rate limits, quotas, latency and cost** under realistic load rather
   than copying published figures, and state them per consumer so a shared quota
   is visible before it is exhausted.
8. **Record the credential requirement:** which secret, which scope, configured
   where. Never the value. Confirm that no probe output, log or example in this
   contract contains a token.
9. **Classify each operation by consequence:** read, write, send, pay, delete,
   sign. This classification decides which grants need human approval later.
10. **Register the consumers** so the drift workflow knows whom to notify.
11. **Publish the contract** and stage it to Context & Memory {OS} with its probe
    date.

## Completion test

- Every operation in the contract exists because a consumer step needs it.
- Every operation has been probed against the live system, with a recorded date.
- Every observed error class, documented or not, has a defined response.
- Idempotency is stated per operation, and non idempotent operations are marked
  prominently.
- Limits, latency and cost are measured, not copied.
- The credential requirement records the secret and its location, and no value
  appears anywhere in the contract or its probe evidence.
- Each operation carries a consequence class.
- Consumers are registered.
- The contract is staged to Context & Memory {OS} and is marked proved rather
  than draft.

A contract that has never been probed stays marked unproved, and consumers are
entitled to refuse it.
