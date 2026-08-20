# Workflow: Signed test spec

**Produces:** an immutable test specification whose threshold and stopping rule
were agreed before any data existed.

## Trigger

One claim has been selected from the claim register, and someone is about to
"just try something and see".

## Steps

1. **Restate the claim** exactly as registered. If the claim has drifted since
   registration, stop and re-register it. Testing a drifted claim answers a
   question nobody asked.
2. **Name the decision the test informs** and its reversibility. An irreversible
   decision earns a stricter threshold and a larger sample; a reversible one
   earns the smallest test that can move the needle.
3. **Choose the instrument.** Take the cheapest one that can still return a
   kill. Typical ladder, cheapest first: existing data already in hand, a
   concierge run done by hand for one customer, a fake door, a smoke test with a
   real call to action, a price ladder, a paid pilot, a letter of intent, a
   pre-sale. Reject any instrument that cannot produce the failing result.
4. **Define the observation.** What exactly counts as one data point, who
   generates it, and what makes a data point invalid.
5. **Set the threshold.** A number and a direction. "At least 6 of 40" not
   "meaningful interest". State the noise band: the range in which the result
   means nothing.
6. **Set the stopping rule.** When the test ends: a sample count, a calendar
   date, or an early stop condition. Both an early kill and an early confirm
   condition where they exist.
7. **Set kill criteria.** What specifically dies in the plan if the claim is
   killed: which feature, which segment, which revenue line, which bet.
8. **Cost it.** Money, calendar days, people contacted, and reputational
   exposure. If the cost exceeds the value of the information, say so and
   propose the weaker claim that is affordable.
9. **Check the sample.** If the affordable sample cannot support the threshold,
   report that plainly and offer the honest weaker claim rather than running an
   underpowered test and reading tea leaves.
10. **Sign.** The claim owner accepts the threshold and stopping rule in
    writing. Record the timestamp. From here the spec is immutable.
11. **Route approvals.** Anything touching a real person, a public surface or
    money goes to the human approval boundary now, before the run.
12. **Emit `validation.test.signed`.**

## Completion test

- The spec names one claim, one instrument, one threshold, one stopping rule.
- The threshold is a number with a direction, and the noise band is stated.
- A reader who dislikes the idea would accept that the failing result kills it.
- The signature timestamp precedes the first observation. No exceptions.
- Cost is stated in money, days and people contacted.
- Required approvals are listed and their status is visible.
