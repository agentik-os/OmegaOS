# Workflow: Compile a context pack

Give a requesting OS exactly enough context for a stated purpose, with
provenance, and nothing it is not permitted to see.

## Trigger

- Any OS begins a mode and requests context.
- A session resumes after an interruption and needs to recover where it was.
- A user asks what is known about a subject before making a decision.

## Steps

1. **Require a stated purpose.** "Everything you have" is not a purpose and is
   refused. The purpose is what makes the pack checkable for sufficiency, and it
   is what bounds the permission check.
2. **Resolve the requester's permission scope:** which projects, which tiers,
   which record types it may read. A requester with no declared scope gets the
   narrowest one, not the widest.
3. **Select candidate records** by relevance to the purpose, not by recency
   alone. A decision from a year ago that still governs outranks last week's
   note that does not.
4. **Apply project isolation.** Records from other projects are excluded by
   default. If the purpose genuinely needs a crossing, it is requested
   explicitly, approved, and logged.
5. **Attach provenance to every record in the pack.** Source, timestamp,
   confidence, record type and tier travel with the content. A fact without its
   provenance is an assertion, and downstream OSes are entitled to weigh it.
6. **Mark unresolved contradictions rather than picking a side.** The requester
   is told that the subject is contested and gets both records. Compilation is
   not the place adjudication happens.
7. **Flag stale and expiring records.** Anything past its review date is
   included with a stale marker, or excluded with a note, never included
   silently as current.
8. **Trim to minimum sufficient.** Remove anything the purpose does not need.
   Volume is a cost paid by the requester, and an oversized pack degrades every
   downstream judgment.
9. **State what was withheld and why:** permission, project isolation,
   irrelevance, or staleness. The requester must be able to tell the difference
   between "nothing exists" and "you may not see it".
10. **Emit the pack** as `memory.context.compiled`, and log the compilation so
    an audit can later ask who received which records.

## Completion test

- The request carried a stated purpose and was refused if it did not.
- Every record in the pack is within the requester's permission scope.
- Every record carries source, timestamp, confidence, type and tier.
- Contested subjects appear as contradictions, not as a single chosen side.
- Stale records are marked or excluded with a note, never passed as current.
- The pack names what was withheld and under which of the four reasons.
- The compilation is logged and the pack is reproducible from that log.

The test of sufficiency is behavioural: the requesting OS completed its mode
without asking for more context. The test of restraint is that nothing in the
pack went unused.
