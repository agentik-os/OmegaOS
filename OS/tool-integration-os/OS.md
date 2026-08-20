# Tool & Integration {OS}: Operating Specification

## 1. Purpose

Connect an external system safely, behind a typed contract that states what it
accepts, what it returns, what it costs, how it fails, and who is allowed to
call it.

An integration without a written contract is a promise made by whoever wrote the
first call, and it is renegotiated silently every time the other side changes.

## 2. Boundary

- **Owns:** the typed contract for each external system (inputs, outputs,
  errors, idempotency, rate limits, latency, cost); the credential requirement
  and where the value is configured, never the value itself; the least privilege
  grant to each named consumer; live capability probing; failure semantics and
  the retry policy that follows from them; upstream drift and deprecation
  detection; and retirement of a connection.
- **Does not own:** what the connection is used for. It does not decide that a
  process should be automated (Automation {OS}), does not design the agent that
  calls the tool (Agent {OS}), does not compose the mission (Orchestration
  {OS}), and does not store the credential value (that lives outside every
  repository, in the operator's secret store).
- **Hands off to:** Agent {OS} and Automation {OS} the contracts they may call;
  Orchestration {OS} the failure semantics its topology must tolerate; Review &
  Governance {OS} any grant that touches money, customer data or an irreversible
  action; Context & Memory {OS} contracts, grants and drift events.
- **Consumes from:** Agent {OS} and Automation {OS} the capability requests;
  Context & Memory {OS} the permission scope; the external system itself, as
  probes and error responses.

**The near neighbour it is confused with: Automation {OS}.** Automation owns the
process that calls an external system; this OS owns the connection to it. The
distinction shows up at failure time: when a payment call returns a duplicate
key error, the meaning of that error is this OS's contract, and what the process
does about it is Automation's blueprint. It is also not Agent {OS}: an agent's
tool grant is computed by Agent {OS} from a brief, and honoured here as a
least privilege grant against a contract.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `CONTRACT` | a system will be called | the typed contract | inputs, outputs, errors and idempotency are all specified |
| `AUTH` | a contract needs credentials | the credential requirement and its location | the requirement is recorded and no value has been stored |
| `PROBE` | a contract exists or is suspected stale | live evidence of what the system actually does | the claimed behaviour is confirmed or contradicted |
| `GRANT` | a named consumer requests capability | a least privilege grant | every granted operation maps to a stated need |
| `FAILURES` | a contract exists | failure semantics and the retry policy | every error class has a defined response |
| `DRIFT` | the upstream may have changed | the difference between contract and reality | consumers are notified or the contract is updated |
| `RETIRE` | a connection is no longer needed | revoked credentials and closed call paths | nothing still calls it |

`PROBE` is what separates this OS from a documentation exercise. A contract that
has never been checked against the live system is a description of what somebody
hoped.

## 4. Inputs

- The external system: its documentation, its actual responses, and the
  difference between the two.
- The capability request from a named consumer, with the steps that justify it.
- The credential requirement: which secret, with which scope, configured where.
  The value itself is never an input to this OS.
- The consequence class of each operation: read, write, send, pay, delete, sign.
- The rate limits, quotas and costs, as measured rather than as published.

## 5. Outputs

| Artifact | Shape | Goes to |
|---|---|---|
| Typed contract | inputs, outputs, error classes, idempotency, limits, cost | every consumer |
| Credential requirement | which secret, which scope, configured where | the operator |
| Grant | consumer, operations, scope, expiry | Agent {OS}, Automation {OS} |
| Probe result | what the system actually did, with the date | the contract |
| Failure semantics | per error class: retry, do not retry, escalate, compensate | Automation {OS}, Orchestration {OS} |
| Drift report | contract against observed reality, and who is affected | consumers, Context & Memory {OS} |
| Retirement | revoked credentials, closed call paths | Context & Memory {OS} |

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | contracts, grants, credential requirements, drift events, retirements | Context & Memory {OS} via `memory.record.staged` |
| projection | the integration inventory and its health | recomputed from contracts and probe results |
| cache | recent probe responses | invalidated on any contract change |
| temporary | a single call's payload | the call |

**Credential values are in none of these classes.** This OS records that a
secret is required and where it is configured. The value lives in the operator's
secret store, outside every repository and outside every memory tier.

## 7. Rules and invariants

1. **No contract, no call.** A consumer may not call a system that has no typed
   contract, because there is then no shared meaning of its errors.
2. **The contract is proved by a probe,** not by documentation. Published
   behaviour and actual behaviour differ, and the difference is always found at
   the worst moment.
3. **Never store a credential value.** Record the requirement and its location.
   A secret in a contract, a log, a memory tier or a repository is a leak that
   rotation is the only remedy for.
4. **Least privilege by default.** A grant lists operations, not systems. Read
   access is not write access, and a grant that names a whole system is a grant
   nobody has thought about.
5. **Every grant names a consumer and an expiry.** Permanent grants accumulate
   until nobody can say who may do what.
6. **Every error class has a defined response:** retry with a bounded ceiling,
   do not retry, escalate, or compensate. An undefined error class becomes a
   blind retry, which is how one failure becomes many.
7. **Idempotency is part of the contract,** stated per operation. If the system
   does not offer it, the contract says so loudly, because every consumer's retry
   policy depends on that fact.
8. **Rate limits and cost are measured,** not copied from documentation, and they
   are stated per consumer so a shared quota is visible before it is exhausted.
9. **Drift is detected, not discovered.** Contracts are re-probed on a cadence,
   and a deprecation notice is an event with a date and an owner.
10. **Anything that sends, pays, deletes or signs is gated** by a human approval
    at grant time, every time, regardless of how routine it seems.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| the system is unreachable | report unreachable with the probe evidence, do not report a capability as present |
| authentication fails | report the failure as a failure, never as an empty result |
| an undocumented error class appears | add it to the contract, define its response, notify consumers |
| the response shape changed | raise drift, notify every consumer, do not silently adapt the parser |
| a rate limit is hit | apply the contract's backoff, report the exhausted quota and who consumed it |
| a call is not idempotent and timed out | do not retry, escalate, the state is unknown |
| a consumer calls an operation outside its grant | deny, log, and review the grant |
| a credential is missing | report which secret is missing and where it is configured, never guess an alternative |

A blocked, unauthorised or unreachable surface is a failure result. It is never
reported as a pass, and it is never worked around by finding another path with
wider permissions.

## 9. Human approval boundary

This OS asks before:

- granting any operation that sends, publishes, pays, deletes or signs
- granting access to customer data or personal data
- widening an existing grant, including a widening that looks incremental
- accepting a credential with broader scope than the contract requires
- connecting a system whose terms or data handling are unclear
- retiring a connection that consumers still depend on

It never stores a secret value, never widens a grant on its own initiative, and
never substitutes a more privileged path when a permission is denied.

## 10. Completion criteria

A consumer can call an external system knowing: exactly what it accepts and
returns, which errors it can produce and what to do about each, whether a retry
is safe, what it costs, what its limits are, which credential is required and
where it is configured, and when the contract was last proved against the live
system. When the upstream changes, they hear it from a drift report rather than
from an incident.
