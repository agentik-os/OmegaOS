# Security {OS}: Commands

Every command this OS exposes, what it does, when to reach for it, and what it
gives back. A command that is not documented here does not exist.

Every command below runs inside a written scope naming the assets, the
environments and the authorised techniques. A command with no scope does not
run, whatever the urgency.

## Runtime commands

| Command | What it does | When to use it |
|---|---|---|
| `agentik install security-os` | Installs this OS into your environment | Once, first |
| `agentik configure security-os` | Collects the minimum context it needs | After install |
| `agentik run security-os` | Starts the OS | Every session |
| `agentik doctor security-os` | Checks config, adapters and dependencies | When something is off |
| `agentik update security-os` | Updates to the latest version | When a release lands |
| `agentik eval security-os` | Runs its evaluation suite | Before trusting it |

## Scoping and modelling

### `/security`

Open the security authority. It reads the pinned build, the quality verdict and
the Blueprint security requirements, and asks for the written scope.

**When to use it:** the moment Quality & Evaluation {OS} issues its verdict.
**Returns:** what is in scope, what is explicitly out, and the assessment plan.

### `/threat-model`

Enumerate assets, actors, entry points, trust boundaries and abuse cases from
the built system.

**When to use it:** first, and again whenever the architecture changes.
**Returns:** the model, including the threats considered and dismissed with
reasons. A dismissed threat with no reason is treated as unassessed.

## Assessment

### `/security-review [--paths <globs>]`

Static review of the paths that matter: authentication, authorisation, session
handling, input handling, cryptography, secret storage, permission logic, tool
invocation.

**Returns:** findings with file and line, plus the critical paths that were not
reviewed, named explicitly.

### `/pentest [--classes <list>]`

Attempt the planned attack classes against an in-scope environment: injection,
exfiltration, privilege escalation, tenancy isolation, session abuse, business
logic abuse.

**When to use it:** after the threat model, never before.
**Returns:** per class, what worked with its reproduction, what did not, and
what could not be attempted with the blocker named.

### `/ai-attack`

Attack the AI surfaces specifically: prompt injection through untrusted
content, tool misuse, data exfiltration through model output, permission
bypass through delegated actions, and context leakage between tenants.

**When to use it:** any product with a model that can read untrusted input or
call a tool. That is nearly all of them.
**Returns:** the attempted attacks, the successful ones with reproductions, and
the mitigations that held.

### `/supply-chain`

Inventory dependencies and build provenance: SBOM, origins, pinning,
advisories, build integrity.

**When to use it:** every release, and immediately when a dependency changes.
**Returns:** the SBOM attached to the build fingerprint, plus unpinned
dependencies, unknown origins and known advisories.

### `/privacy`

Map personal data: what is collected, why, where it goes, how long it is kept,
who can read it.

**When to use it:** any product touching personal data, and any change to data
flows.
**Returns:** the flow map against the Blueprint data governance requirements,
with unmapped flows as findings.

### `/secrets-scan`

Hunt credentials in the repository, the build artifact, the configuration, the
logs and the history.

**When to use it:** every assessment. This is the cheapest finding class and
the most damaging one.
**Returns:** locations only, through the restricted path. Values are never
reproduced in a report, a ticket or a chat, and a live secret triggers rotation
immediately.

## Ruling

### `/findings [--severity <level>]`

The vulnerability ledger.

**Returns:** each finding with severity, affected component, preconditions,
reproduction, impact and remediation. Distribution is restricted while a
finding is unfixed.

### `/harden`

Prioritised remediation.

**Returns:** the fixes ordered by exploitability multiplied by impact, each
concrete enough to become a Stepper step, and each with the retest that will
prove it.

### `/retest <finding-id>`

Re-run the original reproduction after a fix.

**When to use it:** after Builder {OS} closes the remediation step. A fix is
not a fix until the original reproduction fails.
**Returns:** the reproduction attempt and its result.

### `/clearance`

Issue the security clearance.

**Returns:** `CLEARED`, `CLEARED WITH CONDITIONS` (each with a named owner) or
`BLOCKED`, always naming the residual risk and the untested surface. Release
{OS} reads this.

## Command summary

| Command | Does |
|---|---|
| `/security` | open the authority, establish the written scope |
| `/threat-model` | assets, actors, entry points, trust boundaries, abuse cases |
| `/security-review` | static review of the paths that matter |
| `/pentest` | attempt the planned attack classes in scope |
| `/ai-attack` | prompt injection, tool misuse, exfiltration, tenancy leakage |
| `/supply-chain` | SBOM, origins, pinning, advisories, provenance |
| `/privacy` | personal data flows against the governance requirements |
| `/secrets-scan` | credential hunt, restricted reporting, rotation on a live hit |
| `/findings` | the vulnerability ledger |
| `/harden` | remediation ordered by exploitability times impact |
| `/retest <id>` | prove the fix by failing the original reproduction |
| `/clearance` | the security clearance Release {OS} requires |
