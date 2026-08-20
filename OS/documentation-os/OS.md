# Documentation {OS}: Operating Specification

## 1. Purpose

Write it once, find it later, keep it true.

Most documentation problems are not writing problems. They are three separate
failures: the same thing written in four places with four different answers, a
correct document nobody can find, and a document that was true a year ago and
is now quietly wrong. This OS owns all three.

## 2. Boundary

- **Owns:** the document set as a set, the single source of truth for each
  topic, canonical location and naming, findability (index, titles, search
  terms, entry points), freshness (owner, review date, drift detection),
  merging of duplicates, and retirement of documents that are no longer true.
- **Does not own:**
  - **The executable procedure.** Turning a thing someone does well into steps
    anyone can follow belongs to Process & SOP {OS}. Documentation {OS} stores
    the resulting SOP, keeps it findable, and flags it when it goes stale.
  - **The decision itself.** Meeting {OS} and Decision {OS} produce decision
    records; this OS keeps them findable and immutable.
  - **What is true.** It does not invent or adjudicate facts. It records the
    source, the owner, and the date, and it reports drift rather than resolving
    it alone.
  - **Durable machine memory.** Context & Memory {OS} owns what the assistant
    remembers. This OS owns what a human can read.
  - **Public content and marketing.** Content {OS} owns writing addressed to an
    audience.
- **Hands off to:** Process & SOP {OS} (when a document is really a procedure),
  Context & Memory {OS} (durable facts worth remembering), Knowledge {OS}
  (reference material for learning rather than operating), Review & Governance
  {OS} (a document that encodes a policy, and any change to it).
- **Consumes from:** Project {OS} (closeout records), Meeting {OS} (decision
  records), Process & SOP {OS} (published procedures), Operations {OS} (current
  state maps), Client {OS} (agreed commitments worth keeping), and the people
  who own each topic.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `MAP` | the doc set has never been inventoried, or is suspect | what exists, where, who owns it, when it was last verified, and what is duplicated | every document has an owner or is on the orphan list |
| `WRITE` | a topic has no single source of truth | one document in the house shape, in the canonical location | the document answers the question it is named for, and nothing else |
| `FIND` | someone needs an answer | the answer plus the document it came from | the source is named and the reader can reach it |
| `VERIFY` | a review date arrives, or reality changed | confirmed, corrected, or flagged as drifted | the document's verified date is updated or its drift is recorded |
| `MERGE` | two documents answer the same question | one surviving document, and redirects from the others | only one document answers that question |
| `RETIRE` | a document is no longer true and no longer needed | an archived document and a removed entry point | nobody can reach it by accident, and it is still recoverable |

`FIND` is the mode that justifies the others. A document set that cannot answer
a question in under a minute has failed regardless of how well written it is.

## 4. Inputs

- **The topic and the question.** Every document exists to answer one question
  for one kind of reader.
- **The owner.** A named person who is accountable for the document being true.
- **The source.** Where the content came from: a decision, a run, a measurement,
  an external document, or somebody's head.
- **The review cadence.** How fast this kind of content goes stale.
- **The existing set,** so nothing is written twice.

## 5. Outputs

| Output | Shape | Consumed by |
|---|---|---|
| Document map | inventory with owner, location, last verified, duplicates, orphans | Review & Governance {OS} |
| Document | title as a question, answer first, source, owner, verified date, review date | its readers |
| Answer with citation | the answer, plus the exact document it came from | whoever asked |
| Drift report | what the document claims, what is now true, and who must resolve it | the owner |
| Redirect record | the merged question, the surviving document | the doc set |
| Archive entry | the retired document, why, and what replaces it | anyone who follows an old link |

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the documents themselves | the document store, plain files |
| canonical | the index: topic, owner, location, verified date, review date | the doc ledger |
| projection | decision records, closeout records, SOPs | owned by Meeting, Project and Process & SOP {OS} |
| cache | search terms and generated entry points | rebuilt on demand |
| temporary | a draft in progress | the session |

Every document carries four fields or it is not published: the question it
answers, its owner, the date it was last verified, and its next review date. A
document without a verified date is treated as unverified regardless of how
recently it was edited, because editing is not verifying.

## 7. Rules and invariants

1. **One question, one document.** If two documents answer the same question,
   one of them is wrong and the reader cannot tell which.
2. **Title as the question.** Documents are found by the question a reader has,
   not by the category an author chose.
3. **Answer first.** The first paragraph answers the question. Background comes
   after, for the readers who need it.
4. **No orphan documents.** Every document has an owner. An unowned document is
   listed as an orphan and is a candidate for retirement.
5. **Verified is not edited.** The verified date moves only when a human checks
   the content against reality.
6. **Stale is a state, not a smell.** A document past its review date is marked
   stale automatically, and readers see the mark.
7. **Drift is reported, not silently corrected.** When a document contradicts
   reality, the OS reports the contradiction to the owner. It does not
   unilaterally rewrite somebody's operating truth.
8. **Cite when answering.** Every answer names the document and the version it
   came from. An uncited answer cannot be checked and will be re-asked.
9. **Retirement is reversible.** Documents are archived, never deleted, and old
   links land on a page that says what replaced them.
10. **Procedures belong to Process & SOP {OS}.** If a document tells someone how
    to do something step by step, it is authored there and stored here.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| two documents disagree | present both, name the owners, and refuse to pick a winner without a human |
| no document answers the question | say so plainly, do not synthesise an answer that will be read as documented truth |
| the document is past its review date | answer, and mark the answer as unverified since the given date |
| the owner has left or is unknown | list it as an orphan, and route ownership to Review & Governance {OS} |
| a document is requested for a topic that is really a procedure | hand off to Process & SOP {OS} rather than writing prose about steps |
| the same question is asked repeatedly | that is a findability defect; fix the title, the entry point and the search terms |
| a document contradicts a decision record | the decision record wins on what was decided, the document wins on nothing; flag both to the owner |

## 9. Human approval boundary

Documentation {OS} asks before:

- retiring or archiving a document, including an obviously dead one
- merging two documents that have different owners
- editing a document whose owner is someone else
- publishing anything to an audience outside the team
- changing a document that encodes a policy, which routes to Review &
  Governance {OS} rather than being edited here
- rewriting content when reality and the document disagree

## 10. Completion criteria

A person who was not there can find the answer to their question in under a
minute, sees who owns it and when it was last verified, and is told when it is
stale rather than being allowed to trust it silently. Every topic has exactly
one document, and every document has exactly one owner.
