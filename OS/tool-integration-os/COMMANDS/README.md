# Tool & Integration {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. A command that is not documented here does not exist.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install tool-integration-os` | Installs this OS into your environment | Once, first |
| `agentik configure tool-integration-os` | Collects secret store locations, consumers and probe cadence | After install |
| `agentik run tool-integration-os` | Starts the OS | Every session |
| `agentik doctor tool-integration-os` | Probes every registered contract and reports reachable or not, per surface | When something is off |
| `agentik update tool-integration-os` | Updates to the latest version | When a release lands |
| `agentik eval tool-integration-os` | Runs its evaluation suite | Before trusting it |

## OS commands

The OS answers to `/tool`.

### `/tool contract <system>`

Write or update the typed contract for a system.

**When to use it:** before the first call, and after any drift report.
**Returns:** inputs, outputs, error classes, idempotency per operation, rate
limits, latency and cost, plus whether the contract has been proved by a probe or
is still a draft from documentation.

### `/tool probe <system>`

Call the live system and record what it actually does.

**When to use it:** before trusting any contract, on the review cadence, and
whenever behaviour looks different from the contract.
**Returns:** the real responses with their date, the error classes observed
including undocumented ones, and every point where reality contradicts the
contract. Unreachable is reported as unreachable, never as an empty result.

### `/tool auth <system>`

Record the credential requirement and its configured location.

**When to use it:** when a contract needs authentication.
**Returns:** which secret, which scope, and where it is configured. It never
stores, prints, logs or echoes a credential value, in any mode.

### `/tool grant <consumer> <system>`

Issue a least privilege grant.

**When to use it:** when a named agent or automation needs to call a system.
**Returns:** the operations granted, the scope, the expiry, and the justification
per operation. Anything that sends, publishes, pays, deletes or signs is held for
explicit human approval before issue.

### `/tool failures <system>`

Define the response to every error class.

**When to use it:** as part of writing the contract, and after any new error
class appears in production.
**Returns:** per error class: retry with a ceiling, do not retry, escalate, or
compensate. Operations that are not idempotent are marked as never safe to retry
blindly.

### `/tool drift [system]`

Compare the contract against observed reality.

**When to use it:** on the cadence, and after any upstream deprecation notice.
**Returns:** what changed, when it was first observed, which consumers are
affected, and the deprecation dates that apply. It never adapts a parser silently
to hide the change.

### `/tool inventory`

List every integration, its consumers, its grants and its health.

**When to use it:** periodically, and before granting anything new.
**Returns:** contracts with their last probe date, grants with their expiry,
consumers per system, and integrations that no longer have a caller.

### `/tool retire <system>`

Close a connection down.

**When to use it:** when nothing should be calling it any more.
**Returns:** the callers found, the paths to close, the credentials to revoke, and
a refusal while any consumer still holds a live grant.

## Command summary

| Command | Mode | Does |
|---|---|---|
| `/tool contract` | contract | the typed contract for a system |
| `/tool probe` | probe | what the live system actually does |
| `/tool auth` | auth | which secret is required, and where it is configured |
| `/tool grant` | grant | least privilege, named consumer, expiry |
| `/tool failures` | failures | the response to every error class |
| `/tool drift` | drift | contract against reality, and who is affected |
| `/tool inventory` | inventory | who calls what, with which grant |
| `/tool retire` | retire | revoke, close, record |

No command in this OS ever stores or displays a credential value.
