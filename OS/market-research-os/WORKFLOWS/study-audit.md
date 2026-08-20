# Workflow: Study audit

**Produces:** a defect report on an existing market study, deck or claim, naming
every evidence, method, sampling, bias, traceability and decision defect against
the specific record it lives in. Emits `market.study.audited`.

## Trigger

A study, deck, memo or market claim is inherited from someone else, from an
agency, or from an earlier version of this OS, and a decision is about to be made
on it. Also triggered when a document uses the word "validated".

## Steps

1. **Do not re-run the study.** The job is to report what this document can and
   cannot support, not to produce a better one. If the answer is that a new study
   is needed, that is the finding, and it is costed rather than started.
2. **Extract the decision the study claims to support.** If none is stated, that
   is defect one, because a study with no decision has no threshold and therefore
   cannot pass or fail anything.
3. **Extract every load-bearing claim** and classify what it actually is: FACT,
   MEASUREMENT, INFERENCE, ASSUMPTION, HYPOTHESIS or PROPOSAL. Most defects
   surface here, as inferences written in the grammar of facts.
4. **Trace each claim to its evidence.** For each: is there a source, does the
   source say what the claim says, is the source's own boundary the same as this
   study's boundary, and what is the source's date. An untraceable claim is
   quarantined, not softened.
5. **Audit the sizing.** Are the inputs named, is the boundary stated, is there a
   range, is there a sensitivity, and does the top down figure secretly come from
   the same origin as the bottom up figure. Circular sourcing between two paths
   that look independent is a common and serious defect.
6. **Audit the primary research.** Recruitment source, inclusion and exclusion
   criteria, sample size, incentive, consent, instrument version, who ran it, and
   whether the questions asked about past behaviour or about speculative intent.
   Leading, double barrelled and loaded questions are quoted verbatim in the
   report.
7. **Audit the signal grading.** Look specifically for mention volume presented
   as demand, search interest presented as willingness to pay, survey intent
   presented as purchase behaviour, competitor funding presented as market
   attractiveness, and model output presented as primary evidence.
8. **Look for what is missing.** No negative evidence anywhere is itself a
   finding: a study that found nothing against the idea either did not look or
   did not report. Also check for the absent do nothing alternative, the absent
   losing segment, and the absent failed prior attempt.
9. **Check every use of the word "validated".** Only Validation {OS} may apply it,
   and only to a claim with a signed threshold, a completed run and a CONFIRMED
   verdict. Every other use is reported as a defect with the sentence quoted.
10. **Check freshness and expiry.** The dates on the underlying sources, the date
    of the study, and whether any stated expiry has passed. List what has
    changed in the market since.
11. **Rank the defects by decision impact.** Defects that would change the
    decision first, cosmetic defects last. An audit that lists forty equally
    weighted issues gets ignored.
12. **State what the study can still support.** An audit that only destroys is
    less useful than one that says which conclusions survive at what confidence,
    and what the cheapest work is that would repair the rest.
13. **Emit `market.study.audited`** to Context & Memory {OS} so the defects
    travel with the study rather than living in one reader's head.

## Completion test

- Every defect names the specific record, page, table or sentence it lives in.
- Every load-bearing claim is classified, and the classification is compared to
  how the study presented it.
- The sizing audit states whether the estimation paths are genuinely independent.
- The primary research audit reports sample, recruitment and instrument, or
  states plainly that the study does not disclose them.
- Every use of the word "validated" in the source document is accounted for.
- The report says which conclusions survive, at what confidence, and which do not.
- Defects are ranked by decision impact, not by section order.
- The repair path is costed, and no new study was started under cover of the audit.
- `market.study.audited` has been emitted.
