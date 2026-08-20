# Workflow: Asset thesis drafting

Produces the asset thesis: a statement of what a rational third party would be
buying, in terms that are not the owner. Or, where no such statement survives
interrogation, the finding that the operator currently owns a job.

## Trigger

Any of:

- The OS is being run for the first time on this business.
- The business changed materially: a new revenue model, a new customer class, a
  founder departure, an acquisition, a product line closed.
- The existing thesis is older than a year.
- The operator is about to talk to a buyer, a partner or an investor and needs to
  say what is being offered.

## Steps

1. **Ask the question plainly.** If someone bought this business tomorrow and the
   owner did not come with it, what would they own. Take the operator's answer
   verbatim before analysing it.
2. **Decompose the answer into claims.** Split it into discrete assertions:
   customers, contracts, brand, product, process, data, IP, team, distribution.
   Each becomes a claim to test rather than a phrase to accept.
3. **Test each claim for owner substitution.** Ask, for each: does this survive
   the owner leaving. A claim that depends on the owner's relationships,
   judgement, credentials or reputation fails and is marked owner-dependent.
4. **Test each surviving claim for transfer.** Could it be handed to a new owner:
   is the contract assignable, is the IP held by the entity rather than the
   person, is the customer relationship with the business. Read
   `ownership.entity.registered` and `ipasset.registered` for the answers rather
   than asking the operator to recall them.
5. **Count what is left.** The claims that survive both tests are the thesis. If
   none survive, the honest output is that the business is currently a job, and
   that is written down as the finding, with the specific dependencies that make
   it one.
6. **Write the thesis in one paragraph.** Name the transferable source of value,
   who the buyer would be, and what they would be paying for that they could not
   simply hire. No adjectives that a competitor could also use.
7. **Record the failed claims.** Keep the owner-dependent and non-transferable
   claims with their reasons. They are the input to the owner-dependence audit
   and they tend to be reasserted later.
8. **Check the thesis against the entity structure.** If the thesis rests on
   assets the operating entity does not hold, report the contradiction and route
   it to Ownership {OS} as a proposal. Note explicitly that any restructuring to
   resolve it is a question for the operator's lawyer and accountant, not an
   action this OS takes or advises on.
9. **Publish on approval.** With the operator's explicit approval, write the
   thesis to Context & Memory {OS} and emit
   `strategy.asset_thesis.published`. Then run the owner dependence audit.

## Completion test

All of the following hold:

- Every claim in the operator's original answer has been tested for owner
  substitution and for transfer, with the result recorded.
- The published thesis names at least one transferable source of value that is
  not the owner, or the output states plainly that none survived and lists the
  dependencies responsible.
- Failed claims are retained with their reasons rather than discarded.
- Any contradiction with the entity structure is reported and routed as a
  proposal, with the professional question named and unanswered.
- The thesis was published only after explicit operator approval.
