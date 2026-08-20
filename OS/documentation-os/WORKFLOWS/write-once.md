# Workflow: write it once

Produces exactly one document for one reader question, in the canonical
location, with the four fields that keep it usable a year from now.

## Trigger

Someone is about to write a document, or has just answered the same question for
the second time.

## Inputs

- The reader's question, in the reader's own words.
- Who the reader is and what they will do with the answer.
- The source of the content: a decision, a run, a measurement, or a person.
- The existing document set.

## Steps

1. **Search first.** If a document already answers this question, stop writing
   and improve that document instead. This step prevents the duplicate that
   later has to be merged by two owners who disagree.
2. **Check the shape of the need.** If the answer is a sequence of steps someone
   must perform, hand off to Process & SOP {OS} and store the result here.
3. **Write the title as the question.** "How do we onboard a new client" beats
   "Client onboarding documentation".
4. **Answer in the first paragraph.** If a reader stops after three sentences
   they should already have the answer.
5. **Then give the background**, the exceptions and the edge cases, in that
   order.
6. **Name the source.** Where this came from, and how someone could check it.
7. **Stamp the four fields:** the question, the owner, the verified date, the
   review date.
8. **Place it in the canonical location** and register it in the index with the
   search terms real readers would type, including the wrong ones.
9. **Link from where the question is asked**, not only from the index. A
   document found only by search is found by nobody who does not know it exists.
10. **Tell the owner they own it.** Ownership assigned without telling the owner
    is an orphan with extra steps.

## Completion test

- No other document answers the same question, or the duplicate was merged.
- The title is the reader's question.
- The answer appears in the first paragraph.
- The four fields are present: question, owner, verified date, review date.
- The document is reachable from at least one place where the question is
  actually asked.
- The owner has acknowledged ownership.

## Failure paths

| Situation | Response |
|---|---|
| the author does not know the answer, only the topic | do not publish; record the question as unanswered and name who could answer it |
| the topic is contested | publish the decision record instead, and let the document cite it |
| the content will be stale within weeks | set a short review date and say so at the top, or do not write it at all |
| the writer wants one long document covering ten questions | split it; a document answering ten questions is found for none of them |
