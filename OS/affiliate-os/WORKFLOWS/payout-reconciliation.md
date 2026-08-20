# Payout reconciliation

Turn a partner's dashboard number into a reconciled revenue event, or into a
named discrepancy, and never into an unchecked assumption.

## Trigger

A payout period closes for any live partner. Runs on every period, including
periods with zero attributed conversions, because a zero that should not be
zero is the finding this workflow exists to catch.

## Steps

1. **The OS collects own tracking** for the period: clicks, conversions and
   amounts recorded on the operator's side, per surface.
2. **The OS collects the partner report** for the same period, as a cache entry
   with its retrieval timestamp. It is never promoted to canonical.
3. **The OS aligns the periods.** Attribution windows differ from calendar
   months; a conversion attributed by the partner outside the operator's period
   is listed separately rather than silently netted.
4. **The OS computes the gap** per surface and in total, as a count and as an
   amount.
5. **Any gap beyond the tolerance stated in the terms record becomes a named
   finding**: the amount, both sources, the surfaces involved, and the most
   likely mechanism (window mismatch, blocked tracking, rejected conversion).
6. **The OS emits the revenue event** (`/affiliate-revenue`) to Revenue {OS}
   with the reconciled amount, the period, the partner and the attribution
   source. Findings travel with it.
7. **The operator decides what to raise with the partner.** Any message to the
   partner is drafted here and approved before sending.
8. **The OS updates the partner's return figure** in the portfolio, which feeds
   the next `/affiliate-review`.

## Completion test

For the period, all of the following exist: an own-tracking figure, a partner
figure with a retrieval timestamp, an explicit alignment of the two periods, a
computed gap in count and amount, a finding for every gap beyond tolerance, and
exactly one revenue event handed to Revenue {OS} carrying the reconciled amount.

A period closed by adopting the partner's number without an own-tracking figure
fails this test, and is reported as unreconciled rather than as agreed.

## Failure and abort

- **Own tracking unavailable:** do not adopt the partner number. Emit the
  period as unreconciled with the partner figure attached as an unverified
  claim, and raise the tracking gap as the finding.
- **Partner report unavailable:** emit the own figure as provisional, mark the
  period open, and retry at the next close.
- **Gap exceeds the terms tolerance repeatedly across periods:** escalate to
  the operator as a partnership-level finding, not a per-period one, and put
  the partner into the next `/affiliate-review` with a recommendation to pause.
- **Partner rejects conversions without stated reason:** record each rejection
  as a finding. Do not net rejections into the total silently.
