# Disclosed promotion

Take an accepted partner from terms to a live, disclosed, stoppable promotion,
with a human approving every word the audience will read.

## Trigger

A partner has an accept verdict from `partner-vetting` and a recorded terms
record, and the operator wants to run the promotion.

## Steps

1. **The OS drafts the disclosure** (`/disclosure`) for the audience's
   jurisdiction, and states the required placement on each surface. If the
   rules are ambiguous, it says so and stops.
2. **The operator approves the disclosure text** and publishes it. The OS
   records where and when it went live.
3. **The OS builds the promotion plan** (`/promotion`): surfaces, assets,
   sequence, dates, and the stop condition, stated as a threshold, such as a
   complaint rate, a refund rate, or a change to the partner's terms.
4. **The OS drafts each asset** (`/promo-copy`). Every claim is traced to the
   use evidence in the vetting record. Untraceable claims are dropped and the
   drop list is reported. Every asset names who the customer contacts when the
   product fails.
5. **The operator approves the exact copy.** Nothing leaves this OS unapproved.
6. **The OS verifies the disclosure is live** on every surface in the plan. If
   any surface lacks it, publication is blocked for that surface only, and the
   block is reported.
7. **The OS hands the approved plan and assets to Content {OS}** for scheduling
   and publishing, and to Growth {OS} as a measurable channel with its stated
   threshold.
8. **The OS monitors the stop condition** for the life of the promotion and
   fires `/partner-exit` when the threshold is crossed, without waiting for a
   new decision.

## Completion test

Every asset in the plan is either live or explicitly blocked with a reason, and
for every live asset all of the following hold: the disclosure was live on that
surface before the asset went live (checked by timestamp, not by assertion),
the asset's claims each map to a line of use evidence, the asset names the
customer support contact, and an approval record exists carrying the exact text
that shipped.

The promotion has a written stop condition with a numeric threshold. A
promotion running without one fails this test.

## Failure and abort

- **Disclosure not live on a surface:** block that surface, publish the rest,
  report the block. Never publish undisclosed.
- **Jurisdiction ambiguous:** abort before any publication, escalate.
- **Partner changes terms after approval but before launch:** freeze, return to
  `partner-vetting` step 5 with the new terms, and reissue a verdict.
- **A claim cannot be traced to use evidence:** drop the claim, keep the asset,
  report the drop. Never soften an untraceable claim into a vaguer version of
  itself.
- **Stop condition fires:** run `/partner-exit`, take assets down, draft the
  audience notice, and record the reason against the terms record.
