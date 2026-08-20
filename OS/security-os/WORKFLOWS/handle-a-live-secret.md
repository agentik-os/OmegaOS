# Workflow: Handle a live secret

**Mode:** `TEST` interrupted, escalating immediately
**Produces:** the credential rotated, the blast radius assessed, and a
restricted record. Never a public ticket.

## Trigger

A credential, token, private key or connection string is found in source, git
history, a build artifact, a log, a configuration file, an error message or a
client bundle. Also triggered when a third party reports one.

## Preconditions

- The finding is confirmed to be a real credential, not a placeholder or an
  example value.
- The credential's owner and its issuing system can be identified, or that
  identification is itself the first task.

## Steps

1. **Stop the assessment.** A live secret outranks the rest of the plan.
2. **Do not reproduce the value anywhere.** Not in a ticket, not in a commit
   message, not in a chat, not in this workflow's own output. Record the
   location, the type and the last four characters at most.
3. **Confirm it is live.** The least intrusive check that establishes validity.
   Never a check that writes, spends, sends or deletes.
4. **Identify the owner and the issuing system.** Which service, which account,
   who can rotate it.
5. **Request rotation immediately.** Rotation is the remediation. Removing the
   value from the file is not: anything already exposed must be assumed
   captured.
6. **Assess the blast radius.** What that credential could reach, for how long
   it was exposed, who could have seen it (public repository, shared log,
   client bundle, third-party service), and whether access logs can confirm or
   exclude use.
7. **Check for siblings.** The same commit, the same file, the same developer's
   other projects, the same pattern elsewhere. Leaked secrets are rarely alone.
8. **Fix the path that leaked it,** as a Stepper step for Builder {OS}: move
   the value to the secret store, remove it from the artifact, stop logging it.
   History rewriting is a separate, approval-gated decision, and it never
   replaces rotation.
9. **Record it restricted.** Location, type, exposure window, blast radius,
   rotation confirmation, and whether use could be excluded.
10. **Resume the assessment,** and carry the finding into the clearance.

## Completion test

By inspection of the restricted record:

- the credential has been rotated and the rotation is confirmed by the owner or
  the issuing system;
- the exposure window is recorded, with the start bounded by evidence rather
  than by assumption;
- the blast radius is stated, including what could not be excluded;
- the leak path is fixed through a Stepper step, verified;
- no shared channel, ticket or commit contains the value;
- a sibling search has been performed and its result recorded.

A record that says the value was removed from the file but not rotated fails
this test. The value is compromised from the moment it was exposed.

## Failure paths

| What happens | What the workflow does |
|---|---|
| the credential cannot be rotated without downtime | escalate to the owner with the tradeoff stated; rotation stays the default and the delay is recorded as accepted risk |
| nobody knows who owns it | escalate immediately; an unowned live credential is a higher finding, not a lower one |
| it was exposed publicly | assume captured, rotate first, then assess use through access logs, and treat it as an incident for Release {OS} and Operations |
| access logs cannot confirm or exclude use | state that plainly; unknown is not the same as unused |
| someone asks to keep it because rotating is inconvenient | refuse; record the request and the named person who would accept the risk |
| the secret is in git history | rotate first, then treat the history rewrite as a separate approval-gated operation |
