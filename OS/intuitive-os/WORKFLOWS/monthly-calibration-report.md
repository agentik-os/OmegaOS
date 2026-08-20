# Intuitive {OS}: Monthly calibration report

**Produces:** the calibration report: per domain the resolved count, recorded
misses, hit rate, mean Brier score, skill against the base rate, tier, weight
and staleness; plus the overdue signals closed this cycle, every tier change
with the evidence that caused it, and every domain flagged with a
capture-quality defect.

**Trigger:** a month closes. Also fired early when a domain's unresolvable rate
crosses 30 percent, or when the user asks "is my gut actually any good", "how
many of my calls came true", "should I still be trusting this".

**Runs in:** `REVIEW`, calling `RESOLVE` for each due signal and `CALIBRATE`
after the last resolution lands.

**Takes:** the signal log, resolution records and calibration record
(Context & Memory {OS}); decision outcomes for signals attached to a call
(Decision {OS}); the state markers recorded at capture (Health & Energy {OS}).

## Steps

1. List every signal whose resolution date fell inside the closing month, plus
   every signal still open from earlier cycles. Print the two lists separately.
2. Resolve each due signal through `RESOLVE`: read the recorded claim,
   disconfirmer and resolution condition first, then take the outcome, then
   write the verdict. Pull the outcome from Decision {OS} where the signal was
   attached to a call, because that outcome was recorded independently of the
   signal.
3. Close as `unresolvable`, with reason `outcome never observed`, every signal
   that has now been open across two review cycles. Record the reason on each
   one.
4. Hold any contested resolution unresolved and route it to human approval.
   Contested means the recorded outcome and the user's account disagree, or the
   evidence is indirect.
5. Compute the Brier score for every signal resolved this cycle, against the
   base rate reference forecast recorded at capture.
6. Recompute each domain with the nine month recency half-life: resolved count,
   recorded misses, hit rate, mean Brier, skill against the base rate, and the
   age of its newest resolution.
7. Assign the tier per domain: `uncalibrated` below 12 resolved or below 3
   misses; `provisional` at 12 to 29 resolved with positive skill; `calibrated`
   at 30 or more with skill 0.10 or better across the most recent 20;
   `counter-indicative` at 20 or more with skill below 0.
8. Apply staleness: a domain whose newest resolution is older than 12 months is
   marked `stale` and its weight halved. Past 24 months it reverts to
   `uncalibrated`.
9. Route every promotion, and every move to a higher weight tier, through human
   approval. Demotions and staleness discounts apply automatically.
10. Compute the unresolvable rate per domain. Flag any domain over 30 percent
    with a capture-quality defect, block its promotion, and run a capture pass
    over its last ten records to find what made them unfalsifiable.
11. Build the confidence calibration curve per calibrated domain: bucket the
    resolutions by stated confidence (50 to 59, 60 to 69, and so on) and print
    the actual hit rate inside each bucket. Systematic overconfidence shows
    here and nowhere else.
12. Cross the resolutions against the state markers from Health & Energy {OS}.
    Report any pattern as an observation, never as a rule: high sleep debt at
    capture is a correlate to look at, not a discount to apply.
13. Emit the resolution records to Journal {OS} for pattern extraction. Emit
    any domain that has stayed `counter-indicative` across two consecutive
    reports to Mindset {OS} as a candidate belief to examine, as a proposal and
    not as a finding.
14. Persist the updated calibration record through Context & Memory {OS}.

## Completion test

Every signal due in the closing month has a verdict or a stated reason for
being held. Every signal open across two cycles is closed. Every domain has a
tier, a weight, and the resolved count that tier rests on printed beside it.
Every tier change names the evidence that caused it. Every domain over 30
percent unresolvable is flagged and blocked from promotion.

## Failure

- No resolutions this month: report the record unchanged, print the domain
  ages, apply staleness where a threshold was crossed, and say plainly that no
  new evidence arrived. Do not restate last month's numbers as progress.
- Decision {OS} is absent: resolve from the user's account alone and mark each
  such resolution as self-reported, which is a weaker source and is labelled on
  the record.
- The user disputes a verdict already written: do not overwrite it. Record the
  dispute beside the resolution, route it to human approval, and note it as a
  capture-quality defect since the written claim and the remembered claim
  diverged.
- A domain has resolutions but zero recorded misses: keep it `uncalibrated`
  regardless of count, and say why. A record of confirmations only is not
  evidence of skill.
- Context & Memory {OS} is unavailable: produce the report from what can be
  read, mark it as computed on a partial record, and do not write any tier
  change until the canonical store is reachable.
