# Workflow: Renewal and deadline sweep

Produces the dated obligations that, if missed, permanently destroy value.

## Trigger

- Monthly, on the calendar.
- Before any period the user will be unreachable.
- Whenever a new registration, application or licence is recorded.
- Whenever a registry status is refreshed and a date changes.

## Steps

1. **Collect every dated obligation.** Trademark renewals, patent annuities,
   design renewals, domain expiries, licence term ends and renewal windows,
   reporting and audit obligations under inbound licences, use requirements
   (a mark unused in a jurisdiction can become vulnerable), and any office
   deadline currently open.
2. **Verify each date at source.** A date from the registry or from counsel,
   stamped with the day it was read. A date remembered by the user is recorded as
   `unverified` and scheduled for verification.
3. **Escalate unknown dates first.** An obligation whose date is unknown is
   treated as urgent, not as absent. It goes to the top of the list.
4. **Assign a lead time.** Long enough to instruct a professional, gather what
   they need, and pay a fee before the hard deadline. A lead time measured in
   days for something that takes a month is not a lead time.
5. **Name a human owner per obligation.** Not the OS, and not "someone". A named
   person who will receive the task.
6. **Name the professional to instruct.** For each obligation that requires a
   filing, name who files it. This OS files nothing: a missed trademark renewal
   or patent annuity can extinguish the right permanently, with no appeal and no
   way to buy it back, which is why a right of real value belongs on a
   professional docket in addition to this calendar. State that explicitly for
   every obligation on a registered right.
7. **Push tasks.** Emit `ipasset.renewal.due` and send each obligation to
   Execution {OS} as a dated task with its lead time and owner.
8. **Confirm the fee path.** Note who pays and from where. This OS never pays a
   fee and never authorises a payment: it records that the payment is required
   and by when.
9. **Close the loop on the previous sweep.** For every obligation that has passed
   since the last run, record what actually happened: filed, renewed, allowed to
   lapse deliberately, or missed. A deliberate lapse is a valid outcome and is
   recorded as a decision with a reason. A miss is recorded as a miss.

## Completion test

Every dated obligation in the register has a verified date or is flagged
`unverified` and scheduled; every obligation has a lead time, a named human
owner and, where a filing is required, a named professional; every obligation is
a task in Execution {OS}; and every obligation from the previous sweep has a
recorded outcome. An empty calendar for a user holding registered rights means
the sweep did not run, not that nothing is due.
