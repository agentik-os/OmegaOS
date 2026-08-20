# Screen an inbound opportunity

Produces a recorded qualify, pass or abstain decision on one opportunity,
inside a stated time budget, with the reason attached to its source.

## Trigger

Anything arrives that could become a deal: a teaser, a forwarded deck, a broker
listing, a referral message, a conversation at an event. The trigger is arrival,
not interest. Every arrival is screened, including the ones that look obviously
wrong, because the pass reason is data.

## Inputs

- The raw item as the counterparty sent it, kept verbatim.
- The source: the person, channel or list it came through.
- The current screen, versioned, from Context & Memory {OS}.
- The current allocation policy and pacing from Capital {OS}, which says what
  is fundable at all this period.
- The register, to check for a duplicate before creating a record.

## Steps

1. Check the register for an existing record of the same target. If one exists,
   merge, keep every referring source, and stop here.
2. Log the opportunity once. Store the counterparty's own words separately from
   any summary of them. Attribute exactly one source, or mark it unattributed
   and flag it.
3. Confirm the screen version being applied, and note it on the record. If no
   screen exists, stop and run the screen workflow first.
4. Start the time budget. The default is the budget written into the screen.
5. Apply the screen criteria in order, cheapest disqualifier first. Record which
   criterion decided the outcome.
6. Check the outcome against the current allocation policy. An opportunity that
   passes the screen but is outside what Capital {OS} can fund this period is
   qualified and parked with that reason stated, never silently dropped.
7. If the budget is exhausted before a decision, stop screening. Record that the
   budget was exceeded and treat it as the signal that the open question belongs
   to Due Diligence {OS}, not to more screening.
8. Produce the decision: qualify, pass, or abstain with the missing criterion
   named.
9. On pass, draft the pass message and record the reason against the source.
   **Human approval gate:** the pass is sent by a human, not by the OS.
10. On qualify, set stage, next action, owner and date, then build the handoff
    packet for the receiving OS.
11. Emit `dealflow.opportunity.qualified` or `dealflow.opportunity.passed`.

## Completion test

The register contains one record for this opportunity with: an attributed
source, the screen version applied, the criterion that decided it, the elapsed
screening time, and either a delivered pass with its reason or a stage with a
next action, an owner and a date. No field is blank and no field says unknown
without a flag.

## Failure modes

| Failure | What happens |
|---|---|
| no written screen | the workflow stops before step 5 and the screen is written first |
| the source cannot be established | the record is created as unattributed and flagged in the funnel report |
| the screen has no criterion for this case | abstain, name the gap, and ask whether the screen needs a new version |
| the screen and the allocation policy disagree | stop and report the conflict to Capital {OS}, do not resolve it inside the screen |
| the counterparty asks for a value or an offer during screening | decline in role, hand to Acquisition {OS} or Deal Structuring {OS} |
| the budget is exceeded repeatedly on the same category | that is a screen defect, not a decision problem, and triggers a screen revision |
