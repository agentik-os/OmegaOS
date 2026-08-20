# Workflow: Reading extract

**Produces:** typed extracts from one source, each with a locator, at the grain
you will actually use.

## Trigger

Any of:

- A source has been read or partly read.
- A source you ingested is now needed for a specific job (a memo, a model, a
  decision).
- A `corpus-audit` reported a source with no extracts.

## Steps

1. **Restate the purpose** recorded at ingest. If the purpose has changed, say
   so: extraction for "understand the argument" and for "run the method
   tomorrow" produce different extracts from the same pages.
2. **Choose the grain.** CLAIM (an assertion with a scope condition), MODEL (a
   structure with named parts and relations), METHOD (steps you could execute),
   NUMBER (a value with unit, population and year), QUOTE (verbatim), TERM (a
   definition the author uses in a specific way).
3. **Work through the source in order,** recording one extract per idea. Do not
   merge two ideas into one extract because they appeared in the same paragraph.
4. **Attach the locator to each extract** as you write it: page, chapter,
   section, paragraph, or timestamp. An extract whose locator is added later
   from memory is not trustworthy and is marked as reconstructed.
5. **Separate the author from yourself.** Verbatim text is a QUOTE and is
   exact. Your restatement is a paraphrase and is marked as yours. A conclusion
   you drew is your inference and is attributed to you, never to the author.
6. **Capture the scope condition of every claim.** "This works" is not an
   extract. "This works for teams under twelve people who ship weekly" is.
7. **Record what you skipped.** Chapters not read, sections skimmed. This is
   what lets a later `ASK` say honestly that the corpus contains the book but
   not that part of it.
8. **Link.** Point each extract at extracts elsewhere in the corpus that it
   supports, depends on, or contradicts. Record contradictions as
   contradictions.
9. **Emit `librarian.extract.delivered`** and write the extracts to canonical
   state.

## Completion test

- Every extract has a type, a locator and a source record it belongs to.
- No extract mixes the author's assertion with your inference.
- Every CLAIM carries its scope condition; every NUMBER carries unit,
  population and year.
- The skipped sections are recorded.
- Contradictions with existing extracts are linked, not silently resolved.
