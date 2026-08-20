---
name: tool-integration-os
description: Connect external tools safely, with typed contracts. Tool & Integration {OS}, unit 70 of the AGENTIK {OS} suite (08 · AI & SYSTEMS). Use when the user asks about tool & integration or invokes /tool-integration-os.
---

# Tool & Integration {OS}

Connect an external system behind a typed contract that states what it accepts,
what it returns, how it fails, what it costs, and who may call it.

## When to use this

Use it when:

- An agent or an automation needs to call something outside the system.
- A call fails and nobody can say whether retrying it is safe.
- An upstream API changed and the first sign of it was an incident.
- Credentials are being handed to a process and nobody has written down which
  scope it actually needs.
- Grants have accumulated and nobody can say who may do what.
- An integration is being retired and nobody knows what still calls it.

**Near neighbours, and why this is not them.** Automation {OS} owns the process
that calls the system; this OS owns the connection and the meaning of its errors.
Agent {OS} computes an agent's tool grant from its brief; this OS honours that
grant as least privilege against a contract. Orchestration {OS} composes the
mission and must tolerate the failure semantics defined here. None of them stores
a credential value, and neither does this OS.

## Capabilities

- Write a typed contract: inputs, outputs, error classes, idempotency, rate
  limits, latency and cost.
- Probe the live system and confirm or contradict its documented behaviour.
- Record the credential requirement and its configured location, never the value.
- Compute and issue least privilege grants with a named consumer and an expiry.
- Define failure semantics per error class: retry with a ceiling, do not retry,
  escalate, or compensate.
- State per operation whether a retry is safe, and say so loudly when it is not.
- Measure rate limits and cost rather than copying them from documentation.
- Detect upstream drift and deprecation on a cadence, and notify consumers.
- Retire a connection, revoke credentials and close call paths.

## Procedure

1. **Name the operations you actually need,** not the system. A contract is per
   operation, because a grant per system is a grant nobody has thought about.
2. **Write the contract from the documentation** as a first draft, and label it
   unproved.
3. **Probe the live system.** Send the real calls, capture the real responses,
   and record the date. Documented behaviour and actual behaviour differ, and
   the difference is always found at the worst moment.
4. **Enumerate the error classes from the probes,** including the ones the
   documentation does not mention, and define the response to each: retry with a
   bounded ceiling, do not retry, escalate, or compensate.
5. **State idempotency per operation.** If the system offers an idempotency key,
   record how it is formed. If it does not, say so prominently: every consumer's
   retry policy depends on that one fact.
6. **Measure limits and cost** under realistic load, and state them per consumer
   so a shared quota is visible before it is exhausted.
7. **Record the credential requirement:** which secret, which scope, configured
   where. Never the value, in any artifact, log or memory tier.
8. **Issue grants per consumer,** listing operations and an expiry. Anything that
   sends, publishes, pays, deletes or signs goes through a human approval first.
9. **Register the consumers** so a drift report knows who to notify.
10. **Re-probe on a cadence** and raise drift as an event with an owner and a
    date, rather than adapting the parser silently.
11. **On retirement,** find every caller first, close the paths, revoke the
    credentials, and record it.

## Handoffs

| Direction | Counterpart | Contract |
|---|---|---|
| in | Agent {OS} | a capability request with the brief steps that justify it |
| in | Automation {OS} | the external systems a blueprint will call |
| in | Context & Memory {OS} | the permission scope governing grants |
| out | Agent {OS}, Automation {OS} | typed contracts and least privilege grants |
| out | Orchestration {OS} | failure semantics the mission topology must tolerate |
| out | Review & Governance {OS} | any grant touching money, customer data or an irreversible action |
| out | Context & Memory {OS} | contracts, grants, probe results, drift events, retirements |

A denied permission is a result, not an obstacle to route around. This OS never
substitutes a more privileged path when a narrower one is refused.
