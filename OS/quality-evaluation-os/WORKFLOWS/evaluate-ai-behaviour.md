# Workflow: Evaluate AI behaviour

**Mode:** `EVAL`
**Produces:** scored evaluations over a stored dataset, bound to the model and
prompt version that produced them.

## Trigger

The product contains model-driven behaviour and is being certified. Also
triggered whenever the model, the prompt, the retrieval corpus or the tool set
changes, because each of those invalidates the previous scores.

## Preconditions

- The Blueprint AI behaviour contract and the Design AI interaction contracts
  are pinned.
- A dataset exists or can be built, with expected behaviour per case.
- The model and prompt versions in the build are identifiable.

## Steps

1. **Write down what the AI is contractually supposed to do.** Task success
   criteria, what it must ground its claims in, what it must refuse, what it
   must escalate, and what it must never assert. Without this the evaluation
   measures taste.
2. **Build the dataset.** Real cases where possible, adversarial cases
   deliberately, and edge cases from the defect ledger. Every case carries its
   expected behaviour. Store it; a score without its dataset is unreproducible.
3. **Cover the four axes.** Task success (did it do the job), groundedness (is
   every claim supported by the source it cites), refusal correctness (does it
   refuse what it must and not what it must not), and stability (does the same
   input produce an acceptable answer across runs).
4. **Record the versions.** Model id, prompt version, retrieval corpus version,
   tool set. A score belongs to a configuration, not to a product.
5. **Score over the dataset, not over impressions.** Report the distribution:
   pass rate, failure modes by category, and the worst cases in full rather
   than averaged away.
6. **Compare against the threshold.** The threshold comes from the Blueprint
   contract. Where none exists, that absence is itself a finding and goes back
   to Blueprint {OS}.
7. **Regress against the previous certified configuration.** A rise in average
   score that hides a new catastrophic failure mode is a regression, not an
   improvement.
8. **Open defects on failures.** Against the AI contract, with the failing
   cases attached.
9. **Hand adversarial findings to Security {OS}** where a failure is an
   injection, exfiltration or privilege issue rather than a quality issue.

## Completion test

By inspection of the evaluation record:

- every AI contract from Blueprint and Design has at least one scored
  evaluation;
- the dataset is stored alongside the scores;
- the model, prompt, corpus and tool versions are recorded;
- results are reported as a distribution with failure modes, not as a single
  number;
- a comparison against the previous certified configuration exists, or is
  marked as first certification;
- every failing case is either a defect or an explicitly accepted behaviour
  with an owner.

An evaluation reported as a single average with no dataset and no version
stamp fails this test.

## Failure paths

| What happens | What the workflow does |
|---|---|
| no threshold exists in any contract | record the absence as a finding for Blueprint {OS}, and report the measurement without a verdict |
| the dataset is too small to be meaningful | say so, report the confidence honestly, do not present it as certification |
| scores vary widely across runs | that instability is the finding; report the spread and open a defect against stability |
| the failure is an injection or exfiltration | route to Security {OS}; it is not a quality defect and must not be published as one |
| the model provider changed silently under the build | invalidate the scores, re-record versions, rerun |
| real customer data is needed for a realistic dataset | stop, ask for approval, and prefer synthetic or anonymised sampling |
