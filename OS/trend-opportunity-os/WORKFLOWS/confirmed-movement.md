# Workflow: Confirmed movement

**Produces:** a movement record stating a direction AND a rate over a named
measurement window, resting on dated observations from independent sources, or
an explicit refusal naming what is missing.

## Trigger

Signals have accumulated on a watch and somebody is about to use the word
"trend", or a claimed trend has arrived from outside and needs checking before
anyone plans around it.

## Steps

1. **Assemble the dated signals** for the candidate. Every signal must carry an
   observation date, a capture date, a source and a locator. Quarantine anything
   missing a date: it does not enter the count, and it is reported as
   quarantined rather than silently dropped.
2. **Group by independence.** Trace each signal back to its origin. Signals that
   restate one press release, one filing, one vendor blog post or one dataset
   collapse into a single observation. Report the collapse explicitly: this is
   the step where most claimed trends lose two thirds of their evidence.
3. **Label interest.** Mark each remaining independent observation as
   disinterested, participant, vendor selling into the trend, or funded by a
   participant. Count how many disinterested observations survive. Zero is a
   blocking result.
4. **Separate attention from behaviour.** Split the observations into attention
   measures (coverage, search, posts, agendas) and behaviour measures
   (deployments, purchases, hires, migrations, cancellations, filings, price
   moves). Attention may accompany a confirmation. It may never carry one.
5. **Order by observation date and read the direction.** Plot the behaviour
   observations against their observation dates, never against their capture
   dates. If the direction only appears when capture dates are used, there is no
   movement, only a reading burst.
6. **Compute the rate over a stated window.** From what to what, between which
   two dates, on which population. Coarse is acceptable and preferred to
   precise-looking fabrication: "from 3 of 40 to 11 of 40 tracked companies
   between January and July" is a rate. "Growing fast" is not. If the
   observations cannot support a rate, stop here and return the candidate as
   direction only.
7. **Hunt the contradiction.** Actively look for signals that point the other
   way: the reversals, the churned adopters, the quiet deprecations, the flat
   segments. Record them inside the movement record. A movement record with no
   contradicting evidence section has not been tested, it has been assembled.
8. **Apply the confirmation test.** All of the following, or no confirmation:
   several dated observations, spread over time rather than clustered in one
   week, from independent sources, including at least one disinterested source,
   showing a direction in a behaviour measure, with a rate over a stated window.
9. **Confirm or refuse.** On confirmation write the movement record and emit
   `trend.movement.confirmed` to Market Research {OS} and Strategy & Portfolio
   {OS}. On refusal, write which specific condition failed and what would satisfy
   it, keep the watch open, and say plainly that this is still a candidate. A
   refusal is a delivered result.
10. **Do not extrapolate.** State what has moved and how fast. Do not project the
    curve, and do not estimate the size of the outcome: sizing belongs to Market
    Research {OS}.

## Completion test

- The record states a direction and a rate, and names the two dates the rate was
  measured between.
- The observation count is the count after independence collapse, and the
  collapse is shown.
- At least one disinterested source is named.
- Behaviour observations, not attention observations, carry the direction.
- Observations are ordered by observation date, and the capture dates are
  visible separately.
- A contradicting-evidence section exists and is either populated or explicitly
  states that a search for contradiction was run and returned nothing.
- Quarantined signals are listed with the reason they were quarantined.
- If confirmation failed, the record names the exact missing condition and what
  would satisfy it, and no downstream event was emitted.
