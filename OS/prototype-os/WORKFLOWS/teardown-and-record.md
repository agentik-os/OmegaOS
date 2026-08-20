# Workflow: Teardown and record

**Mode:** `TEARDOWN`
**Produces:** a written verdict and an artifact that no longer exists.

## Trigger

Any of: a prototype's question has been answered, a prototype's expiry date has
passed, `ledger` reports an artifact still alive, or a prototype is abandoned
before producing an answer.

## Preconditions

- The prototype is identified and its directory or environment is known.
- Whatever evidence it produced has been collected out of it.

## Steps

1. **Collect the evidence first.** Logs, measurements, recordings, transcripts,
   screenshots. Once the artifact is gone, anything left inside it is gone.
2. **Write the verdict, including for an abandoned prototype.** An abandoned
   prototype has a verdict too: `INCONCLUSIVE`, with why it was abandoned. A
   question that quietly disappears will be asked again in three months.
3. **Name the upstream records.** Which Blueprint ASSUMPTION or UNKNOWN, which
   Design decision, which planned step this changes. A verdict pointing at
   nothing changes nothing.
4. **Revoke everything the artifact held.** API keys, service accounts,
   webhooks, third-party app registrations, DNS entries, test accounts,
   scheduled jobs.
5. **Delete the artifact.** Directory, container, deployment, database,
   bucket. Where a stakeholder wants it kept, archive it read only, mark it
   ARCHIVED PROTOTYPE, and record that it must never be imported.
6. **Check for leaks.** Anything the prototype wrote outside its own directory:
   a row in a shared table, a queue subscription, a cron entry, a Vercel
   project, a repository branch.
7. **Record the teardown.** What existed, where, what was revoked, what was
   deleted, and by whom.
8. **Update the ledger.** The prototype moves to torn down, and disappears from
   the live list.

## Completion test

```bash
test ! -e prototypes/<id>/artifact        # the artifact path is gone
```

And, by inspection: `prototypes/<id>/verdict.json` exists with a verdict and at
least one named upstream record; the teardown record lists every credential
revoked; the ledger shows no live artifact for this id; and no scheduled job,
webhook or deployment referencing it remains.

## Failure paths

| What happens | What the workflow does |
|---|---|
| the artifact is already in use by someone | stop, escalate immediately, record it as an unplanned dependency; this is an incident, not a cleanup |
| a credential cannot be revoked by this OS | name it, name its owner, and leave the teardown open rather than reporting it closed |
| the stakeholder wants it kept alive | require an owner, a new expiry and an approval, and record it as a live artifact in the ledger, never as torn down |
| evidence was never collected and the artifact is already deleted | record `INCONCLUSIVE` with the evidence lost, and say so plainly |
