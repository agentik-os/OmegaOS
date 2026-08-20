# Workflow: Exit readiness assessment

Score how sellable the business is against what a buyer will actually test, and
turn the answer into a gap list with owners and dates.

**Mode:** `ASSESS`
**Produces:** a readiness score, a classified gap list, and `exit.readiness.scored`
**Typical duration:** one working session, plus the time advisers take to answer

## Trigger

Any of:

- the operator asks whether the business could be sold, or for how much
- a buyer, a broker or an investor makes an unsolicited approach
- the operator sets a target window for stepping out of the operation
- a prior assessment is more than twelve months old
- a structural change lands: a new entity, a co-founder joining or leaving, a
  material customer concentration, an acquisition of the operator's own

The unsolicited approach is the most common trigger and the worst starting
point. Run this workflow anyway, and run it before replying.

## Steps

1. **Fix the shape and the window.** Which liquidity shape is contemplated
   (full sale, partial sale, secondary of the owner's shares, management buyout,
   acquihire, licensing buyout, wind-down), the honest reason behind it, and the
   earliest acceptable date, the target date, and the date after which the
   operator no longer wants to be running this. Record the reason. A sale to
   fund a next venture and a sale to escape an unbearable operation accept
   different structures and tolerate different timelines.

2. **Pull the projections, do not rebuild them.** Read the entity map from
   Ownership {OS} (`ownership.entity.registered`), the IP schedule from
   IP & Asset {OS} (`ipasset.title.assigned`), the measured value drivers from
   Business Strategy {OS} (`strategy.value_driver.measured`), and the financial
   history from Revenue {OS} and the operator's accountant. Cite each source. If
   a projection is missing, record it as missing and continue; do not
   reconstruct another OS's truth here.

3. **Test the three that explain most of the price gap.** Customer and revenue
   concentration, contract assignability, and key person dependence. For each,
   state the current position with a number where one exists, and state what a
   buyer will conclude from it. These three account for most of the distance
   between an owner's expected price and a buyer's offer, and none of them is
   fixed quickly.

4. **Walk the buyer's test list.** Corporate records and cap table, financial
   history and its basis, tax filings and their currency, customer contracts and
   their assignment clauses, employment and contractor agreements, IP ownership
   and assignments, licences and permits, litigation and disputes, data
   protection posture, and the operating dependence on the owner personally. For
   each, record the current state. Record absence as absence.

5. **Score, and show the arithmetic.** Produce the readiness score with the
   components visible. A score without its components is a number the operator
   cannot act on.

6. **Classify every gap.** Exactly one of:
   - **paperwork**, closed by producing or locating a document, owned here
   - **value**, meaning the business is worth less than the owner assumes,
     owned by Business Strategy {OS}
   - **structural**, meaning an entity in the wrong place, an unassigned IP
     right, an unsigned founder or contractor agreement, owned by Ownership {OS}
     or IP & Asset {OS} and requiring counsel

   Assign an owner and a date to each gap. Structural gaps are sequenced first
   regardless of size, because they are the slow ones and they are the ones a
   buyer finds late.

7. **Name the three gaps that most change the outcome**, and say why for each.
   A gap list of forty items with no ranking is a list nobody works.

8. **Route the gaps out.** Value gaps to Business Strategy {OS}. Structural
   gaps to Ownership {OS} or IP & Asset {OS} and into an adviser question pack
   for counsel. Dated preparation work to Execution {OS}. Paperwork gaps stay
   here and feed the diligence readiness index.

9. **Emit `exit.readiness.scored`** with the score, the component breakdown, the
   gap count by classification, and the date of the assessment.

10. **State what this assessment is not.** It is an internal readiness view. It
    is not a valuation, not a formal opinion of sellability, and not legal, tax
    or accounting advice. Where a gap turns on a legal or tax question, name the
    professional who owns the answer rather than answering it here.

## Completion test

The assessment is done when all of the following hold:

- the liquidity shape, the reason and the three dates are recorded
- every line of the buyer test list has a state, and no line is blank
- every gap carries exactly one classification, an owner and a date
- structural gaps are sequenced ahead of paperwork gaps
- the three highest-impact gaps are named, with the reason for each
- value gaps have been handed to Business Strategy {OS} and structural gaps to
  Ownership {OS} or IP & Asset {OS} and to counsel
- `exit.readiness.scored` has been emitted

It is not done if any gap is unclassified, if any test line is blank, or if an
absent document has been recorded as present on the strength of a recollection.
