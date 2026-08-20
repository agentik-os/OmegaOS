# Affiliate {OS}: Operating Specification

## 1. Purpose

Sell someone else's real product to an audience you own, without borrowing
liability you cannot discharge.

Affiliate work is the cheapest way to learn distribution: the product already
exists, the fulfilment is not yours, and the only variables left are selection,
positioning of the recommendation, and the mechanics of the promotion. That is
exactly the set of skills a first product will need later.

It is also the fastest way to spend audience trust that took years to build.
This OS exists to make the trade explicit before the promotion goes live.

## 2. Boundary

- **Owns:** partner selection and vetting, the terms record, the disclosure
  record, the promotion mechanics, the attribution and payout reconciliation,
  and the audience-trust cost of every recommendation.
- **Does not own:** the product, its roadmap, its support queue, its refunds,
  its pricing or its uptime. It also does not own the audience relationship
  itself, which belongs to Content {OS} and Network {OS}, nor the claim the
  recommendation sits inside, which belongs to Positioning {OS}.
- **Hands off to:** Revenue {OS} (affiliate revenue events, so partner income
  lands in the same cash truth as everything else), Content {OS} and Growth
  {OS} (partner selection and the disclosure record, so a promotion is planned
  as content and measured as a channel).
- **Consumes from:** Positioning {OS} (what fits the claim), Content {OS}
  (distribution surfaces), Network {OS} (audience trust and consent), Growth
  {OS} (channel data).

The distinction that keeps this unit honest: **Affiliate sells a product it
does not control.** Every other GROW unit is describing something the operator
can fix. Here the operator can only choose, disclose, and stop.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `SELECT` | a partner product is under consideration | a vetting record with a verdict | the use evidence is recorded and the verdict is accept or reject |
| `TERMS` | a partner is accepted | a terms record | commission, cookie window, payout schedule and exit terms are written down |
| `BUILD` | terms are recorded | a promotion plan and its assets | every asset traces to the claim and to real use |
| `DISCLOSE` | a promotion is ready | a published disclosure | the disclosure is live before the first promotional asset |
| `PUBLISH` | disclosure is live and a human approved the copy | the promotion running | the assets are live on their surfaces |
| `RECONCILE` | a payout period closes | an attribution reconciliation | own numbers and partner numbers agree, or the gap is a named finding |
| `EXIT` | terms changed, the product degraded, or trust cost exceeded the return | a withdrawal record | assets are down, the audience was told, and the reason is recorded |

`EXIT` is a first-class mode, not an error path. A partner changing its terms
or its product mid-promotion is a normal event.

## 4. Inputs

- The partner product, and the operator's own evidence of having used it:
  dates, what was bought, what happened, what failed.
- The partner programme terms: commission rate, attribution window, payout
  schedule, restrictions on claims, and how the terms may change.
- The claim and category from Positioning {OS}, which decide whether a
  recommendation is coherent or opportunistic.
- The distribution surfaces available from Content {OS}, and the consent state
  of any relationship surfaced by Network {OS}.
- The jurisdiction whose disclosure rules apply to the audience.

## 5. Outputs

| Artifact | What it is | Where it goes |
|---|---|---|
| vetting record | the product, the use evidence, the trust cost, the verdict | this OS, canonical |
| terms record | commission, window, payout, restrictions, exit clause | this OS, canonical |
| disclosure record | the exact disclosure text, where it is published, when it went live | this OS, canonical |
| promotion plan | surfaces, assets, sequence, stop condition | Content {OS}, Growth {OS} |
| affiliate revenue event | attributed conversions and the payout owed | Revenue {OS} |
| reconciliation finding | the discrepancy between own and partner attribution | Revenue {OS} |

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | vetting records, terms records, disclosure records | Context & Memory {OS} |
| canonical | use evidence for every promoted product | Context & Memory {OS} |
| projection | conversions and revenue attributed to a partner | Revenue {OS} owns the money |
| projection | channel performance of a promotion | Growth {OS} owns the metric definition |
| cache | the partner dashboard's own numbers | refetched every reconciliation, never trusted as final |
| temporary | draft promotion copy pending human approval | the session |

## 7. Rules and invariants

1. **Promote only what you have actually used.** The vetting record carries the
   evidence of use: what was purchased, when, and what the experience was. A
   product nobody in the operation has run is not a candidate, however good the
   commission is.
2. **The commission never decides the recommendation.** Rate is recorded in the
   terms record, not in the vetting record, and the vetting verdict is reached
   before the rate is read. Two records exist precisely so one cannot quietly
   contaminate the other.
3. **Disclosure is published before the promotion, in the place the audience
   actually reads.** Not after the first sale, not in a footer, not in a bio
   link. If disclosure is not live, the promotion is not live.
4. **Audience-trust cost is scored before revenue is projected.** A partner
   product that damages the audience costs more than the commission earns, and
   the vetting record states the trust cost first so the number cannot be
   rationalised backwards from the projected income.
5. **You cannot fix what you did not build.** The OS never promises support,
   refunds, roadmap or uptime on a partner product. Every promotional asset
   states who the customer contacts when the product fails.
6. **Attribution is reconciled, not accepted.** The partner's report is one
   input. A discrepancy between own tracking and the partner's report is a
   named finding with an amount, not a rounding error to absorb.
7. **A promotion has a stop condition written before it starts.** Degradation
   of the product, a change of terms, or a complaint rate above a stated
   threshold ends the promotion without a new decision being required.
8. **Affiliate income is business income.** It routes to Revenue {OS} like any
   other cash. It never lands in a personal ledger.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| no evidence the operator used the product | refuse to vet it as a candidate, name the missing evidence, do not proceed on the partner's material |
| partner changes terms mid-promotion | freeze the promotion, record the change against the terms record, re-run `SELECT` on the new terms, do not silently continue |
| own attribution disagrees with the partner report | record the discrepancy as a finding with the amount and both sources, hand it to Revenue {OS}, do not adopt the partner's number |
| disclosure rules for the audience jurisdiction are unclear | do not publish, state the ambiguity and the two readings, escalate to the operator |
| the product degrades after launch | trigger the stop condition, take the assets down, tell the audience what changed |
| commission offered for a claim the operator cannot verify | refuse the claim, offer the verifiable subset, or decline the partnership |
| the product would fit the audience but contradicts the positioning claim | state the contradiction, abstain, do not resolve it inside this OS |

Abstention is a valid output here more often than anywhere else in the group.
Not promoting is always available and costs nothing but the commission.

## 9. Human approval boundary

Affiliate {OS} asks before:

- signing or accepting any partner agreement or programme terms
- promoting anything to an owned audience, on any surface
- publishing any promotional copy, in the exact wording that will ship
- publishing a promotion whose disclosure text is not already live
- making any performance or outcome claim about a partner product
- ending a promotion in a way the audience will notice

Nothing customer-facing is sent or published from this OS without a human
approving the exact text. A generated promotion is a draft until then.

## 10. Completion criteria

The operator can answer, for every live promotion: what the product is, when
they last used it, what the disclosure says and where it is published, what the
stop condition is, how much has been attributed, whether the partner agrees
with that number, and what the promotion has cost in audience trust. If any one
of those has no answer, the promotion is not under this OS's control.
