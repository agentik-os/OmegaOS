# Claim ledger

Take a raw belief about why customers pick you and turn it into a ledger entry
that can be defended, dated and eventually falsified.

## Trigger

Any of: a new claim is proposed by the operator, a downstream unit asks for a
claim it can publish, a competitor changes their published claim, or a
`/position-review` marks an existing claim expired and a replacement is needed.

## Steps

1. **Operator** states the belief in their own words, unedited. This is the
   starting text, not the claim.
2. **Positioning {OS}** pulls the customer language corpus from Customer
   Discovery {OS} and produces the ranked verbatim terms for the problem, plus
   the list of operator terms that no customer ever used.
3. **Positioning {OS}** rewrites the belief in customer vocabulary and shows
   both versions side by side. The operator confirms the rewrite or corrects
   it; the divergence is kept, not erased.
4. **Positioning {OS}** runs `/position-map` and produces each rival's live
   claim, quoted from source with a capture date. Unverifiable rival claims are
   dropped from the comparison.
5. **Positioning {OS}** forces the exclusion: the operator names what this
   claim makes them worse at. An empty exclusion aborts the workflow.
6. **Positioning {OS}** runs `/position-test` and produces two verdicts:
   recognition against customer utterances, and distinctiveness against rival
   claims. Either failure returns to step 3.
7. **Positioning {OS}** assembles the evidence set: the utterances, the rival
   comparison, and any Validation {OS} record of the claim meeting a buyer.
8. **Operator** states the expiry condition: the specific event or measurement
   that would make this claim false, and the review date.
9. **Positioning {OS}** writes the ledger entry with claim, evidence, exclusion,
   expiry condition, tester and date, then runs `/position-conflict` across the
   whole ledger and produces either a clean result or a contested pair.
10. **Positioning {OS}** emits the updated statement and ledger to Brand {OS},
    Offer {OS}, Content {OS}, Sales {OS}, Growth {OS}, Storyteller {OS} and
    Affiliate {OS}.

## Completion test

The ledger entry exists with all six fields populated (claim, evidence,
exclusion, expiry condition, tester, date), `/position-test` recorded a pass on
both recognition and distinctiveness with the specific utterance and rival
claim cited, and `/position-conflict` returns no contested pair involving this
entry. Any empty field, any missing citation, or any contested pair means the
workflow did not complete.

## Failure and abort

- No customer language corpus: abort after step 2, name Customer Discovery
  {OS} as the missing input, and produce nothing. A claim invented without
  customer language is the failure this workflow exists to prevent.
- Empty exclusion at step 5: abort. Record the attempt so the same undifferentiated
  claim is not proposed again next quarter.
- Distinctiveness failure at step 6 twice in a row on the same ground: abort
  and escalate to the category decision, because the problem is the category,
  not the wording.
- Contested pair at step 9: the entry is written as contested, the emission in
  step 10 is withheld, and a human decides which claim survives.
- The operator refuses the customer wording at step 3: the divergence is
  recorded as a dissent and the claim proceeds only under explicit human
  approval, marked as operator wording rather than customer wording.
