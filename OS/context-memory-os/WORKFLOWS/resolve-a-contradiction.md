# Workflow: Resolve a contradiction

Adjudicate two records that disagree, or two identities that might be one, and
keep the loser.

## Trigger

- A capture finds an existing record on the same subject with a different value.
- An OS reports that the context it received conflicts with what it observed.
- An entity resolution is ambiguous: two names, accounts or projects that may or
  may not be the same thing.
- A user says that something the system believes is wrong.

## Steps

1. **Open the contradiction as an object.** It gets an identifier, both records,
   and a state. It is not a cleanup task and it is never resolved by deleting one
   side quietly.
2. **Compare provenance, not recency.** Record type first: a user statement
   outranks an inference regardless of which arrived later. Then source
   authority, then confidence, then time.
3. **Check whether both can be true.** Many contradictions are scope errors: two
   projects, two time periods, two definitions of the same word. If both hold
   under different scopes, the resolution is to scope them, not to choose.
4. **Check whether it is a legitimate change over time.** A fact that was true
   and is now different is not a contradiction; it is a supersession with a date.
   Record the transition rather than the conflict.
5. **For entity ambiguity, do not merge on similarity.** Two similar names are
   evidence, not proof. Ask the user. Until answered, both entities persist and
   neither absorbs the other's records.
6. **Adjudicate, and write the reason.** Which record governs, on what grounds,
   and what would reverse the decision.
7. **Supersede rather than delete.** The losing record is retained, marked
   superseded, and remains visible in audit and history. A store that hides its
   corrections cannot be trusted about anything else.
8. **Propagate.** Any OS that received the losing record in a context pack is
   notified, because a decision may already have been taken on it.
9. **Record the outcome** so that a repeated contradiction on the same subject is
   recognisable as a pattern rather than adjudicated again from scratch.
10. **Escalate if unresolvable.** If provenance is symmetric and no scope splits
    them, the contradiction stays open and is reported as open. An open
    contradiction is a truthful state; a forced resolution is not.

## Completion test

- The contradiction exists as an addressable object with both records attached.
- The adjudication states its grounds and what would reverse it.
- The losing record is superseded and still visible, never deleted.
- Entity merges happened only on explicit user confirmation.
- Every OS that consumed the losing record has been notified.
- An unresolvable contradiction is reported as open rather than closed by
  preference.

A contradiction closed without a written reason is not resolved. It is hidden,
and it will resurface in a context pack.
