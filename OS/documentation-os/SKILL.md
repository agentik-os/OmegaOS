---
name: documentation-os
description: Write it once, find it later, keep it true. Documentation {OS}, unit 43 of the AGENTIK {OS} suite (05 · OPERATE). Use when the user asks about documentation or invokes /documentation-os.
---

# Documentation {OS}

Write it once, find it later, keep it true.

## When to use this

Use it when the same question is answered repeatedly, when nobody can find the
document everybody agrees exists, when two documents disagree, when a document
is confidently wrong, or before writing anything that another person will later
rely on.

Typical openings: where is the doc for this, we wrote this down somewhere, this
page says something that is no longer true, I keep explaining the same thing.

Near neighbours it is confused with:

| If the real need is | The right OS is |
|---|---|
| a step-by-step procedure someone else must run | Process & SOP {OS} |
| what the assistant should remember about you | Context & Memory {OS} |
| learning material and reference reading | Knowledge {OS} |
| writing for an audience or a market | Content {OS} |
| the decision itself, not its record | Decision {OS}, Meeting {OS} |
| whether a written policy may change | Review & Governance {OS} |

## Capabilities

- Inventory a document set: what exists, where, who owns it, when it was last
  verified, what is duplicated and what is orphaned.
- Write a document to a fixed shape: title as the reader's question, answer
  first, source, owner, verified date, review date.
- Answer a question from the set and cite the exact document it came from.
- Detect drift between a document and reality, and report it to the owner
  instead of silently rewriting it.
- Merge duplicates into one surviving document with redirects from the others.
- Mark stale documents automatically at their review date so readers are warned.
- Retire documents reversibly, leaving an archive entry and a pointer to the
  replacement.
- Fix findability defects: titles, entry points, naming, search terms.

## Procedure

1. Establish the question the document answers, and the reader who has it.
2. Search the existing set first. If a document already answers it, improve that
   one rather than adding a second.
3. Name the owner. No owner, no publication.
4. Write answer first, background after, source named.
5. Stamp the four fields: question, owner, verified date, review date.
6. Place it in the canonical location and register it in the index.
7. On any answer given to a reader, cite the document and its verified date.
8. At each review date, verify against reality: confirm, correct, or flag drift
   to the owner.
9. When two documents answer one question, run the merge with both owners.
10. When a document is no longer true and no longer needed, archive it and
    redirect its entry points.

## Handoffs

| Send to | What | What they expect |
|---|---|---|
| Process & SOP {OS} | anything that is really a procedure | the raw steps and who performs them |
| Context & Memory {OS} | durable facts worth remembering | confirmed, inspectable, removable |
| Knowledge {OS} | reference material for learning | the source and its provenance |
| Review & Governance {OS} | documents that encode policy, and drift nobody will own | the document, the contradiction, the decision requested |
| Project {OS} | closeout records to be filed | the record, the topic, the owner |
| Client {OS} | anything the client is entitled to receive | the document and its verified date |
