# Workflow: change verification

Produces evidence of whether an approved change did what it claimed, and a
verdict: standardise, adjust or revert.

## Trigger

The verification date of an approved change arrives. Also runs when a change is
cited as a success without anyone having checked it.

## Inputs

- The change decision, its conditions and its verification test.
- The measurement the test named, from KPI & Analytics {OS} or from the owning
  OS.
- What actually happened since the change, including anything unintended.
- The reversal path recorded at approval time.

## Steps

1. **Read the test as it was written.** Not as it is now convenient to read it.
   A test reinterpreted after the result is not a test.
2. **Collect the measurement it named.** If the measurement was never
   instrumented, the change is unverified, and that is the finding.
3. **Compare against normal variation.** A change credited with a movement
   inside the noise band has not been shown to do anything.
4. **Look for the unintended effects.** Ask the people who work inside the
   changed system what got harder. Improvements that move the cost elsewhere are
   common and rarely reported by the person who proposed the change.
5. **Check the conditions were met.** An approval with conditions that were
   never implemented is not the change that was approved.
6. **Choose the verdict:**
   - Standardise: it worked. Write it into the procedure through Process & SOP
     {OS} and into the policy set where relevant.
   - Adjust: it partly worked. Name the specific adjustment and set a new
     verification date.
   - Revert: it did not work, or it caused more than it fixed. Use the reversal
     path recorded at approval.
   - Unverified: it cannot be judged. Say so, and stop citing its benefit
     anywhere.
7. **Record the verdict** in the append-only audit trail, with the measurement.
8. **Tell everyone who changed their behaviour** because of the change, whichever
   way the verdict went.
9. **Feed the learning back.** A change that failed teaches more than one that
   worked, provided the failure is recorded rather than quietly absorbed.

## Completion test

- The verification test was read as originally written.
- The named measurement was collected, or the change is recorded as unverified.
- The movement was compared to normal variation before being credited.
- Unintended effects were actively sought from the people inside the system.
- The verdict is one of standardise, adjust, revert or unverified, and is
  recorded with its evidence.
- Everyone who changed behaviour has been told the outcome.

## Failure paths

| Situation | Response |
|---|---|
| the measurement was never instrumented | mark unverified, issue the instrumentation requirement, and set a new date rather than assuming success |
| the result is ambiguous | adjust rather than standardise, and set a second verification date with a sharper test |
| reverting is now expensive | say so plainly, decide on merit rather than on sunk cost, and record the reason if it is kept |
| the change is defended without evidence | the absence of evidence is the finding; record it as unverified and remove the claim from anywhere it is cited |
