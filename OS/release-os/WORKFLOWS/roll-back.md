# Workflow: Roll back

**Mode:** `ROLLBACK`
**Produces:** production restored to the last known good version, with the data
implications stated rather than assumed.

## Trigger

An abort criterion fired mid rollout, production verification failed, or a
defect appeared after a release. Rolling back is the default response; staying
broken while investigating is the choice that requires an explicit owner.

## Preconditions

- The last known good version is identified and is still deployable.
- The rollback plan written with the candidate is at hand.
- Where the release included a data migration, the approval boundary is
  respected before executing.

## Steps

1. **Stop the spread first.** Halt the rollout before diagnosing anything.
   Every minute of diagnosis while the change is still advancing costs more
   users.
2. **Name the last known good version.** By fingerprint, not by "the previous
   one". Two releases in a day make that ambiguous exactly when it matters.
3. **Check what the rollback does not restore.** Data written by the new
   version, migrations already applied, messages already sent, webhooks already
   delivered, caches already poisoned. This is the part of a rollback that
   surprises people.
4. **Decide the shape.** Straight revert where nothing moved. Revert plus a
   forward-fix migration where data moved and the old code cannot read the new
   shape. Feature-flag off where the change was flagged, which is usually the
   fastest and safest path.
5. **Get approval where data is involved.** A rollback that touches a
   migration is an approval boundary, and this is true under time pressure,
   which is the only time it gets skipped.
6. **Execute, and record as you go.** Every step, its time, its result. The
   record written afterwards is always shorter and always wrong.
7. **Verify the restored state.** Exercise the golden path on the restored
   version and read the signals. A rollback is not complete because a
   deployment finished.
8. **State the residue.** What is still inconsistent: records written in the
   new shape, partial workflows, notifications that cannot be unsent. Name it
   for Operations & Automation {OS} and Delivery & Customer Success {OS}.
9. **Open the defect.** The reason for the rollback becomes a Stepper step for
   Builder {OS}, never an informal hotfix.
10. **Route the postmortem** to Review & Governance {OS} when the incident was
    customer-visible.

## Completion test

By inspection of the rollback record:

- the restored version is identified by fingerprint;
- the golden path has been exercised on the restored version with its real
  responses recorded;
- signals are back within their thresholds, or the remaining deviation is named
  with an owner;
- the data residue is stated explicitly, including the case where there is none;
- the cause is open as a defect against Builder {OS};
- the timeline records each step as it happened.

A rollback recorded as complete with no statement about data fails this test.

## Failure paths

| What happens | What the workflow does |
|---|---|
| the old version can no longer read the new data shape | do not force it; use a forward fix or a compatibility shim, and treat it as an incident |
| the rollback itself fails | escalate immediately, declare an incident, and stop attempting variations alone |
| nobody knows which version was good | check the deployment record; where it is absent, that gap is itself a finding for the next release |
| the change was not flagged and cannot be partially undone | roll back the whole release; that is the cost of shipping unflagged, and it is recorded |
| someone wants to fix forward instead | allowed, with a named owner and a time box; when the box expires, roll back |
| customers were affected | notify through Delivery & Customer Success {OS}, with what happened and what was restored |
