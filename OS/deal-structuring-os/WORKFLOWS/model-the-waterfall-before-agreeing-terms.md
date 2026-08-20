# Model the waterfall before agreeing terms

Produces the cash each party receives at a low, a middle and a high exit, under
the proposed structure, before anyone agrees to it in a message.

## Trigger

A term is proposed by either side: a preference, a ratchet, a seller note, a
deferred payment, a pool change, a new class of share. The trigger is the
proposal, not the negotiation meeting, because terms are usually agreed
informally long before they are written.

## Inputs

- The complete existing cap table, including old convertibles, warrants,
  unissued options and any side letter that changes economics.
- The chosen instrument and the reason the alternatives were rejected.
- The price range from Acquisition {OS}, or the proposed commitment from
  Capital {OS}.
- Verified numbers from Due Diligence {OS}, with the unverified ones marked.
- Three exit values agreed with the operator: a bad outcome, an average outcome
  and a good one.

## Steps

1. Load the cap table and validate it. If any instrument is missing, stop and
   name it. A waterfall on a partial cap table is confidently wrong.
2. Confirm the three exit values with the operator, and write down why each was
   chosen. The middle one carries the analysis.
3. Model the pre and post cap table under the proposed structure, including
   pool creation and its timing.
4. Run the waterfall at each exit value: what each party receives in cash, and
   which conversion decision each holder would rationally take.
5. Isolate the proposed term: run the model with it and without it, and take the
   difference. That difference is the term's price.
6. Label every input that is still unverified, and state which conclusions
   depend on it.
7. If the term is a metric linked payment, model the case where the party
   controlling the metric optimises against it.
8. Present the middle case first, then the low, then the high. A structure that
   only works at the high exit is reported as such in plain words.
9. Rank the term against the other open terms by cash value, and place it in the
   trade set or the walk away set.
10. Record the result in the term register, so nobody later agrees to a term
    that has already been priced and rejected.
11. **Human approval gate:** agreeing the term, in any channel including an
    informal message, is a human decision taken with the number in front of them.
12. Emit `structure.waterfall.modelled`.

## Completion test

For the proposed term, the record shows the cash each party receives at all
three exit values with and without it, the price of the term, who pays that
price, every unverified input it rests on, and an explicit human decision to
accept, trade or reject it.

## Failure modes

| Failure | What happens |
|---|---|
| the cap table is incomplete | the model refuses to run and names the missing instruments |
| the operator will not pick a low exit value | the workflow stops, since a model without a downside case answers the wrong question |
| a term's value cannot be quantified | it is reported as unquantifiable and ranked qualitatively, never given an invented number |
| an input is unverified and material | the model runs, the input is labelled, and the dependent conclusions are named |
| the term was already agreed informally before pricing | it is priced anyway, and the gap between the agreed and the priced position is reported |
| tax treatment would change the ranking | the workflow states the question for the adviser and marks the ranking provisional |
