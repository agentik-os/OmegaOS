---
name: librarian-os
description: Your reading and source corpus turned into retrievable understanding. Librarian {OS}, unit 11 of the AGENTIK {OS} suite (02 · DISCOVER & DECIDE). Use when the user asks about librarian or invokes /librarian-os.
---

# Librarian {OS}

Retrieve from what you already have, with a locator behind every claim.

## When to use this

Reach for Librarian when:

- You remember a book made an argument and you cannot reconstruct it.
- You are about to research something your own shelf already answers.
- You finished a book and want extracts you will still be able to use in a year.
- Two authors you trust disagree and you want the disagreement laid out with
  citations rather than resolved by whoever you read last.
- You need a quote with a page number for something you are writing.
- Your notes have become a pile: sources with no extracts, highlights with no
  locators, three copies of the same paper.

Near neighbours, and the line between them:

| Confused with | Difference |
|---|---|
| Research {OS} | Librarian retrieves from what you already have. Research goes and gets what you do not have. Check Librarian first: it is cheaper, faster, and the source is already vetted by the fact that you kept it. |
| Knowledge {OS} (group 08) | That unit runs a shared organisational knowledge base over operational documents for a team. Librarian serves one person over the material they chose to read. |
| Context & Memory {OS} | That unit is where canonical records live and how they persist. Librarian is what puts source records and extracts there. |
| Market Research {OS} | Market evidence about a market at a point in time, with a decision attached. Librarian has no opinion about markets, it has sources. |
| Brainstorm {OS} | Brainstorm invents. Librarian supplies the mechanisms and analogies that Brainstorm collides. |

## Capabilities

- Ingest a source of any format (book, paper, article, transcript, recording,
  internal document) into a canonical record with edition, date and copy
  location.
- Extract at the grain of use: a claim with its scope condition, a model with
  its parts, a method with its steps, a number with its unit and population, a
  quote verbatim.
- Attach a locator to every extract: page, chapter, section, paragraph or
  timestamp.
- Answer a question strictly from indexed material, cite every sentence, and
  state plainly what the corpus does not cover.
- Compare several sources on one subject and preserve their disagreements with
  attribution.
- Build shelves: named subsets with an explicit inclusion rule.
- Record a reread as a delta against the previous reading rather than replacing
  it.
- Audit the corpus for extracts without locators, sources without extracts,
  duplicates and stale editions.
- Track licence and quotation limits per source so an export cannot leak more
  than it may.

## Procedure

1. **Identify the source exactly.** Title, author, edition or version, date,
   format, and where the copy lives. An ambiguous identity produces duplicate
   records later, which is the most common corpus defect.
2. **State what the source is for** before extracting. The purpose sets the
   grain: an argument you will use, a method you will run, numbers you will
   cite.
3. **Ingest.** Create the source record. Index the text if the format allows it.
   Record licence and quotation limits.
4. **Extract.** One extract per idea. Type it: CLAIM, MODEL, METHOD, NUMBER,
   QUOTE, TERM. Attach the locator. Mark your own commentary as yours.
5. **Link.** Connect the extract to the sources it agrees with, contradicts or
   depends on. A contradiction is recorded as such, not resolved.
6. **Answer.** For a question, retrieve candidate extracts first, then assemble
   the answer only from what was retrieved, citing each part. Name what the
   corpus does not cover instead of filling the gap.
7. **Hand the gap over.** If the answer needs material outside the corpus, stop
   and hand the residual question to Research {OS} with the part already
   answered attached, so the search starts smaller.
8. **Supersede on reread.** New extracts point at the previous ones. Nothing is
   deleted.
9. **Audit periodically.** Run the corpus audit and fix the defects it names,
   starting with extracts that lost their locator.

## Handoffs

| To | What it receives | What it does with it |
|---|---|---|
| Research {OS} | `librarian.extract.delivered`, plus the residual question | starts its search already knowing what your shelf answers |
| Brainstorm {OS} | `librarian.extract.delivered`: models, mechanisms, analogies | collides them into concepts |
| Trend & Opportunity {OS} | `librarian.source.indexed` with dates | uses dated source material as signal history |
| Business Model {OS} | `librarian.extract.delivered`: mechanisms and case economics | builds the model on precedent rather than intuition |
| Context & Memory {OS} | every source record and extract | makes them durable and reachable from every other OS |

Received from: Context & Memory {OS} (`memory.context.compiled`), Research {OS}
(`research.evidence.compiled`, when a finished memo is filed back into the
corpus as a source in its own right).
