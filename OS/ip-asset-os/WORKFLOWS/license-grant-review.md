# Workflow: Licence grant review

Run before any licence is granted out, and before any licence is taken in.
Produces a licence record, a conflict check, and the brief a lawyer drafts from.

## Trigger

- A counterparty asks to use an asset, or offers you the use of theirs.
- Terms have been verbally agreed and nothing is written down.
- A signed licence has come back and needs to be filed against the register.
- An existing licence is up for renewal, renegotiation or termination.

## Steps

1. **Identify the asset in the register.** If it is not registered here, stop and
   run the inventory workflow on that asset first. You cannot grant what you have
   not recorded.
2. **Check title before terms.** If the asset's title status is `unproven` or
   `disputed`, do not proceed to terms. Granting rights in an asset you cannot
   prove you own is the specific failure this OS exists to prevent. Route to
   counsel with the gap named.
3. **Check encumbrances.** List what already constrains the asset: open-source
   obligations in what ships, an inbound licence with restrictions on
   sublicensing, an existing exclusive, an assignment to another entity.
4. **Capture the seven terms.** Direction (out or in), counterparty, exclusivity,
   territory, term and renewal, field of use, and revocation trigger. Missing
   terms are recorded as missing, not guessed.
5. **Run the exclusivity conflict check.** Compare against every existing grant
   on the same asset. Where an exclusive would overlap an existing grant in
   territory or field of use, refuse to record it, present both grants side by
   side and escalate.
6. **Note what this OS does not decide.** The royalty basis is recorded, the
   price is not set here: that is Pricing {OS}. How the licence is packaged and
   sold is Offer {OS}. Whether the licensee pays is Revenue {OS}. The tax
   treatment of the royalty stream is a question for a tax professional in the
   relevant jurisdiction, and it is written into the brief as an open question,
   never answered here.
7. **Produce the counsel brief.** The facts, the asset's title evidence, the
   encumbrances, the seven terms, the conflict check result, and what the user
   wants to achieve. Ask before instructing anyone. The lawyer drafts and the
   parties execute; this OS does neither.
8. **Record as `unexecuted`.** Until the signed document is in hand, the record
   carries `unexecuted` and its terms are treated as a draft.
9. **Route the change.** Send the change request to Review & Governance {OS} and
   wait for `change.approved` before committing the grant to the register.
10. **File the executed document and flip the status.** Copy terms from the
    signed document, not from the negotiation thread, where they differ. Set the
    record to `executed`, name the human who signed, and emit
    `ipasset.license.granted`.
11. **Push the dates.** Term end, renewal window and any reporting or audit
    obligation go to the calendar with lead times and a named human owner.

## Completion test

The licence record exists with all seven terms present or explicitly marked
missing; the asset's title status was `proven` before terms were captured; the
exclusivity conflict check ran and its result is recorded; `change.approved` was
received; the executed document is filed and its signer is named; and every date
in the licence is in the calendar with a lead time. A record still marked
`unexecuted` after the parties believe the deal is done is an open item, and the
workflow is not complete.
