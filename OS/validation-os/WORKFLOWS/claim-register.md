# Workflow: Claim register

**Produces:** every claim a plan depends on, written so it can be false, ranked
by what it costs to be wrong about it.

## Trigger

Any of:

- A plan, deck, concept or blueprint is about to receive budget or build time.
- Brainstorm {OS} emits `brainstorm.concept.selected`.
- Market Research {OS} returns GO or PIVOT and the founder wants the risk
  surface named before funding.
- Two people disagree and neither can state what evidence would settle it.

## Steps

1. **Collect the artifact and its ancestry.** The plan itself, plus any prior
   verdicts on the same subject from Context & Memory {OS}. A claim already
   settled is not re-registered; it is cited.
2. **Extract every material statement the plan depends on.** Include the ones
   nobody wrote down: pricing assumptions inside a revenue line, an assumed
   channel behind a growth number, an assumed integration behind a feature.
3. **Rewrite each as a falsifiable claim.** Subject, magnitude, window. Reject
   any wording that cannot be false. "The market is large" is not a claim.
   "At least 300 UK dental practices spend over 400 pounds a month on patient
   recall today" is.
4. **Label the current basis of each claim.** One of: EVIDENCE (with source),
   INFERENCE, ASSUMPTION, PREFERENCE. Preferences leave the register and go to
   Decision {OS}; they are not testable and pretending otherwise wastes budget.
5. **Assign an owner per claim.** The person who will act on the verdict. An
   unowned claim gets no test budget.
6. **Score each claim on two axes:** cost of being wrong (what the plan loses if
   it is false and discovered late) and current confidence (how much real
   evidence stands behind it now).
7. **Rank and cut.** Order by expected cost of error. State explicitly which
   claims you are choosing not to test and why that is acceptable.
8. **Write the register to canonical state** and emit
   `validation.claim.registered`.

## Completion test

- Every claim in the register can be false, and a reader can say what
  observation would falsify it.
- Every claim has an owner and a basis label.
- The register names the untested claims and the reason each was excluded.
- No claim in the register is already settled by an existing verdict.
- The top claim is the one whose being wrong would cost the most, not the one
  that is easiest to test.
