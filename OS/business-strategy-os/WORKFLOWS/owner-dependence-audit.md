# Workflow: Owner dependence audit

Produces the owner-dependence assessment: the list of decisions, relationships,
credentials and pieces of knowledge that exist only in the owner's head, each
with what would have to be true for someone else to hold it.

## Trigger

Any of:

- The asset thesis has just been published or revised.
- The review cadence has elapsed and dependence was last scored more than one
  interval ago.
- The operator says they cannot take time off, cannot delegate, or is the
  bottleneck.
- A buyer, investor or lender has asked what happens if the founder leaves.
- A key person is leaving, joining, or changing role.

## Steps

1. **Confirm the thesis is current.** Read the asset thesis from Context &
   Memory {OS}. If none exists, stop and run `/thesis` first: dependence scored
   without a thesis measures a business nobody has defined.
2. **Enumerate the decision classes.** Sales, pricing, delivery and quality,
   hiring and firing, vendor and supplier relationships, technical judgement,
   financial control, customer relationships, legal and contractual, and any
   class specific to this business. Add the specific ones; never drop the
   standard ones because they feel obvious.
3. **Assign each class to a holder.** For every class, the answer is a named
   person, a document, or a system. "It depends" is not an answer, and neither
   is a role title. Where the answer is the owner, record it as the owner.
4. **Decompose each owner-held class into decisions.** For each class the owner
   holds, list the actual recurring decisions inside it. "The owner does sales"
   is not usable. "The owner sets discount limits above 15 percent and is the
   only person who prices custom scope" is.
5. **Establish the credential and access dependencies.** Which licences,
   certifications, bank mandates, signing authorities, platform accounts and
   personal guarantees are attached to the owner rather than the business. Read
   the entity structure from Ownership {OS} to see which ones the entity could
   hold instead.
6. **Test each relationship for transferability.** For every material customer,
   supplier or partner relationship: is the counterparty's relationship with the
   business or with the person. Where it is with the person, record it, and
   check whether the contract is assignable.
7. **Score each class.** Three levels only: transferable (a document or a system
   holds it), transferable with work (someone could hold it after named work is
   done), or owner-locked (nobody else can hold it as things stand). Do not
   invent a finer scale; the extra resolution would be opinion.
8. **Label the evidence.** Each score is verified where it traces to a document,
   a system record or an upstream metric, and unverified where it rests on the
   operator's account. State the split explicitly at the end.
9. **Derive the remediation.** For each owner-locked and each transferable-with-
   work class, name what would have to exist: a documented procedure, a hire, a
   system, a contract amendment, a credential moved to the entity.
10. **Route the outputs.** Documentation gaps to Process & SOP {OS} as specific
    named procedures. Roles the owner must stop occupying to Team & Delegation
    {OS}. Credential and contract items to Ownership {OS} as proposals only, with
    the note that any entity, contract or guarantee change goes to the operator's
    lawyer or accountant before it is acted on. Dated remediation work to
    Execution {OS} on the operator's approval.
11. **Emit and store.** Write the assessment to Context & Memory {OS} and emit
    `strategy.owner_dependence.scored`.

## Completion test

All of the following hold:

- Every decision class has a holder that is a named person, a document or a
  system, with the owner named where the owner is the answer.
- Every owner-held class is decomposed into specific recurring decisions, not
  role labels.
- Every score carries a verified or unverified label, and the split is stated.
- Every owner-locked and transferable-with-work class has named remediation.
- Nothing was executed by this OS: every output left as a fact or a proposal.
