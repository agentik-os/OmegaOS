# Workflow: Indexed source

**Produces:** one canonical source record, indexed where the format allows, with
its licence limits and copy location recorded.

## Trigger

A source enters your world: a book bought, a paper saved, a transcript
generated, an article kept, an internal document received. Run this at arrival,
not at completion. Sources ingested "later" are the ones that never get
ingested.

## Steps

1. **Identify the source exactly.** Title, author or authors, edition or
   version, publication date, format. For a paper, the DOI. For a book, the
   ISBN of the edition you actually hold. For a web page, the URL and the
   capture date.
2. **Check for a duplicate.** Search the corpus by author and by title
   fragments before creating a record. A second edition is a new record that
   points at the first; a second copy of the same edition is not.
3. **Record where the copy lives.** Path, shelf, device, or the fact that it is
   a physical book on a physical shelf. A record whose copy cannot be found
   cannot be verified later.
4. **Record the licence facts.** Yours, borrowed, library, restricted,
   confidential. How much may be quoted, and whether the text may leave the
   machine. Anything ambiguous is recorded as restricted.
5. **State the purpose.** One line: what you expect this source to be for. This
   sets the grain of extraction later, and it is the difference between fifty
   useless highlights and six usable extracts.
6. **Index the text** if the format allows it. If it does not (a physical book,
   a locked PDF), record that the source is present but unindexed, so a later
   `ASK` can say the corpus contains it but cannot search it.
7. **Approve before any external service** is used for OCR, transcription or
   embedding. That crosses the human approval boundary.
8. **Emit `librarian.source.indexed`** and write the record to canonical state.

## Completion test

- The record carries title, author, edition or version, date, format, copy
  location and licence note.
- No duplicate record exists for the same edition.
- The purpose line is present and specific enough to set an extraction grain.
- The record states honestly whether the text is searchable or only present.
- Nothing left the machine without approval.
