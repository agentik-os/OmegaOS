# Workflow: Red team pass

**Produces:** a ranked list of the ways the leading concept fails, where every
attack carries a repair, an experiment, or a kill recommendation, plus the kill
criteria that were written before anyone was emotionally attached.

## Trigger

A concept is leading and someone is about to commit: a freeze, a handoff, a
spend, a hire, or a calendar quarter. Also triggered on `/redteam`, and whenever
the three cells agreed too quickly for the agreement to be informative.

## Steps

1. **Fix the target.** Name the exact concept version under attack, its genome
   (`BS-GEN`) and its surface decision (`BS-SRF`). A moving target absorbs every
   objection by quietly becoming a different concept.
2. **Steelman it first.** Write the strongest honest case for the concept, in
   the words its most competent advocate would use. An attack on a weak
   restatement proves nothing.
3. **Premortem.** It is eighteen months later and this failed completely. Write
   the account of how, in the past tense, with the specific first domino named.
   Vague failure ("adoption was slow") is rejected; name the mechanism.
4. **Attack the load-bearing loci.** For each locus the genome marked as
   load-bearing, ask what happens if it is false, if it is half true, and if a
   competitor gets it for free.
5. **Incentives and abuse.** Who is worse off if this works, and what do they do
   about it. How does a bad actor extract value from it. What does a rational
   user do that the concept did not intend. What does the concept reward that it
   did not mean to reward.
6. **Second-order effects.** What becomes true after this succeeds, and does the
   concept survive its own success. Include the version of the concept that
   works and creates a worse problem than the one it solved.
7. **Second-cheapest killer.** Name the single cheapest fact that, if true,
   makes the whole concept worthless. Then name the second, because the first is
   usually the one the founder already anticipated.
8. **Repair or kill, no bare objections.** Every attack above ends in exactly
   one of: a repair (a concrete structural change to the genome), an experiment
   (`BS-EXP`, the cheapest test that would settle whether the attack lands), or
   a kill recommendation. An objection with none of the three is incomplete and
   is not recorded as a finding.
9. **Write the kill criteria.** State what specifically would make this concept
   dead: which observation, at what level, by when. Write them now, before the
   result exists, and record them as `BS-DEC` with status experiment-first.
10. **Rank by damage times likelihood.** Order the surviving findings so the
    founder reads the one that can actually hurt them first.
11. **Record and route.** Findings become `BS-ARG` against the concept, repairs
    become new genome versions with parentage, experiments join the queue,
    external-fact dependencies become `BS-HYP` routed to Research {OS} or
    Validation {OS}.
12. **State the delta.** Answer explicitly: what changed because of this pass.
    If nothing material changed, say so and stop rather than running the same
    attack with different words.

## Completion test

- The steelman exists and precedes the first attack.
- The premortem names a specific first domino, not a mood.
- Every load-bearing locus was attacked at least once.
- Incentives, abuse and second-order effects each produced at least one finding
  or an explicit statement that this concept has no meaningful exposure there,
  with the reason.
- Every recorded finding carries a repair, an experiment or a kill
  recommendation. Zero bare objections survive into the output.
- Kill criteria are written as observations at a stated level by a stated date,
  and they were written before any result existed.
- Findings are ranked by damage times likelihood, not by order of discovery.
- No finding asserts an external fact. Anything requiring one is a `BS-HYP` with
  a falsifier and a named destination.
- The pass ends with an explicit answer to "what changed because of this round".
- If independent agents were unavailable, the output discloses that the passes
  were separated rather than parallel, and no transcript was invented.
