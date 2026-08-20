# Workflow: Threat model a system

**Mode:** `MODEL`
**Produces:** the enumerated assets, actors, entry points, trust boundaries and
abuse cases, with the threats considered and dismissed and why.

## Trigger

A new system is being built, or an existing one changed its architecture,
authentication, data flows, tenancy model or tool surface. Also triggered
before a first assessment of an inherited product.

## Preconditions

- The built system is inspectable: code, configuration, infrastructure
  definitions, or a running instance in scope.
- Blueprint security, privacy and abuse requirements are pinned.

## Steps

1. **Model the system as it is, not as designed.** Read the deployed
   configuration and the code. Architecture diagrams describe intentions;
   attackers meet reality.
2. **Enumerate assets.** What has value: credentials, personal data, money
   movement, intellectual property, availability, reputation, and the ability
   to act on a user's behalf.
3. **Enumerate actors.** Anonymous user, authenticated user, other tenant,
   staff, third-party integration, the model itself, a compromised dependency,
   an insider. Each with what it can legitimately do.
4. **Enumerate entry points.** Every route, endpoint, webhook, queue consumer,
   file upload, scheduled job, admin surface, tool the model can call, and any
   place untrusted content reaches a prompt.
5. **Draw the trust boundaries.** Where data crosses from less trusted to more
   trusted. Every crossing is where validation and authorisation must exist,
   and where they usually do not.
6. **Write abuse cases, not just misuse.** Not "a user might type the wrong
   thing" but "an actor with this access wants this asset and will try this".
7. **Score by exploitability times impact.** Not by how interesting the attack
   is.
8. **Record what you dismissed and why.** A threat model that lists only
   threats to be tested hides its own blind spots. A dismissal with no reason is
   treated as unassessed.
9. **Turn the model into a test plan.** Each entry point and boundary gets a
   planned attack class, which the assessment then executes.
10. **Raise missing requirements.** Where the model exposes a control the
    Blueprint never required, propose it upstream as a decision request.

## Completion test

By inspection of the threat model:

- every route, endpoint, queue, job, upload, admin surface and model tool
  appears as an entry point;
- every entry point is attached to at least one trust boundary crossing or is
  explicitly marked as internal with the reason;
- every asset has at least one actor who would want it;
- every listed threat carries a score and a planned test or a recorded
  dismissal with a reason;
- every gap between a needed control and a stated requirement is raised
  upstream.

An entry point discovered later during testing that was not in this model is a
defect in the model, and it is recorded as one.

## Failure paths

| What happens | What the workflow does |
|---|---|
| the system cannot be fully inspected | model what is inspectable, list what is opaque, and mark the opaque part unassessed |
| an entry point has no owner | record it and escalate; an unowned surface is usually the oldest one |
| the tenancy model cannot be explained by anyone | treat isolation as unproven and plan a tenancy test first |
| the model surface can reach a tool with real effects | flag it as a primary attack path, not a secondary one |
| the architecture changes mid assessment | re-model the changed part before continuing, and record the version the model belongs to |
