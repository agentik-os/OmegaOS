# Workflow: Research memo

**Produces:** a memo answering one framed question, in which every claim carries
a source, a date and a class, and which a hostile reader can check without this
OS.

## Trigger

Someone needs to know something, the answer is not already in the corpus, and a
decision is waiting on it.

## Steps

1. **Frame the question.** Rewrite the topic as a question with a scope, a time
   window, a population and the decision it feeds. Write the out-of-scope list
   explicitly: a question with no edges has no completion test. Get the asker to
   agree this is the question they meant. Emit `research.question.framed`.
2. **Check the corpus first.** Query Librarian {OS} and read what Context &
   Memory {OS} already established. State what is answered already and what the
   residual gap is. If the gap is empty, stop here and say so, naming the
   extract. That is a complete and successful session.
3. **Split the gap into sub-questions.** Each one must be answerable by some
   identifiable kind of source. A sub-question no source type could settle is
   really an unknown, and belongs in the unknown register from the start.
4. **Write the search plan.** For each sub-question: the source types, where they
   live, and the search paths. Then the stopping rule (sub-questions covered, N
   independent sources per load-bearing claim, or the budget) and the cost.
5. **Route approvals.** List every step that needs a human decision: paid
   sources, restricted terms, contacting a person, personal data, credentials.
   Send them to the human approval boundary now, before gathering.
6. **Gather and log.** Retrieve, and record every attempt in the source ledger
   with locator, date accessed and access outcome. Log every paywall, block, dead
   link and rate limit with its reason. Blocked is a result, not silence.
7. **Vet every source.** Tier, who produced it, how they know, when, who funded
   it, what they gain, and whether the primary behind it is reachable. Retrieve
   the primary where a source is summarising one. Keep failed sources in the
   ledger, marked, so the next session does not rediscover them.
8. **Extract claims with provenance.** One claim per line with source, locator,
   date of access, date of the evidence, and a class: `fact`, `measurement`,
   `inference`, `assumption`, `unknown`, `contested`. Measurements carry method
   and population. Your own reasoning is labelled `inference`.
9. **Triangulate.** For each load-bearing claim, name the independent origins.
   Where several citations trace to one origin, mark it `circular` and count it
   as single-sourced.
10. **Run the contradiction pass.** Where sources disagree, keep both sides.
    Never average. Class those claims `contested` and emit
    `research.claim.contested` for any a decision rests on.
11. **Verify the citations.** Reopen every citation behind a load-bearing claim.
    Anything that cannot be reopened is removed from the answer and dropped to
    `unknown`, with the failure recorded.
12. **Write the unknown register.** What could not be established, where you
    looked, what blocked you, and what access would settle it.
13. **State confidence.** Per load-bearing claim and for the answer overall, with
    the reason: independent sources, tier, recency, method quality, size of the
    remaining unknown. Name the cheapest action that would raise it.
14. **Write the memo.** Answer first. Then evidence, contradictions, unknowns,
    confidence, and the full source ledger including everything blocked. No
    verdict, no "validated", no GO or NO-GO. Emit `research.evidence.compiled`.
15. **File it back.** Write question, plan, ledger, claim set, contradictions and
    memo to canonical state, and hand the memo to Librarian {OS} so the next
    corpus check finds it. Emit `research.memo.published`.

## Completion test

- The memo answers the framed question, not an adjacent one, and the framing is
  reproduced in it.
- Every claim carries a source, a date and one of the six classes. No
  unclassified sentence appears in the answer.
- Every citation was reopened during step 11, and the ones that failed are gone
  from the answer and present in the record.
- Blocked and paywalled sources appear in the ledger with their reason. The
  reader can see what was not reachable.
- Contradictions are shown with both sides. No claim was averaged into a middle
  number no source supports.
- The unknown register is present, even when empty, and says so explicitly.
- Confidence is stated with reasons a reader can check, not as a bare adjective.
- The memo contains no verdict and does not use the word "validated".
- A reader with the memo and an internet connection can reach every claim
  without asking this OS anything.
