# Workflow: Market decision pack

**Produces:** a versioned evidence body and one bounded market decision (`GO`,
`PIVOT`, `HOLD`, `NO-GO`, `INSUFFICIENT EVIDENCE`) with its conditions, kill
criteria and expiry date, plus the Blueprint Input Manifest when the decision is
Blueprint eligible.

## Trigger

A launch, funding or build decision is pending on a market nobody has measured,
or a concept has arrived from Brainstorm {OS} on `brainstorm.concept.selected`
and someone is about to write a product definition on top of it.

## Steps

1. **State the mode and depth.** Usually `FULL_VALIDATION`, or `DILIGENCE` when
   the stakes are investment, acquisition, board or enterprise. Choose the lowest
   depth profile that can support the decision and name what it excludes out
   loud, before any work is done.
2. **Frame the decision.** Decision owner, capital and calendar time at risk,
   geography, horizon, success threshold, non-goals, risk tolerance, deliverable.
   If there is no named owner, stop and say so: a study with no decision owner
   has no stopping condition and no threshold to measure against.
3. **Recover.** Pull prior chats, files, studies, decisions and constraints into
   a canonical baseline. Attach provenance to every recovered item. List
   conflicts as `CONFLICT` records rather than resolving them by preference.
4. **Register the hypotheses.** Problem, segment, urgency, current behaviour,
   value, solution, differentiation, price, acquisition, retention, feasibility,
   regulatory, timing. Each one falsifiable, each with the method that would
   settle it and its cost.
5. **Design the research and run the source preflight.** Evidence questions,
   methods, sample, source plan, stopping rules, budget, bias controls. Then
   authority, access rights, terms, robots directives, rate limits, licensing,
   retention, personal data and permitted use, before a single request leaves the
   machine. A lane that fails preflight stops and is recorded as unavailable.
6. **Run the sizing model.** Follow [market-sizing-model.md](market-sizing-model.md).
   Emit `market.sizing.modeled`.
7. **Segment demand and choose a beachhead.** Behavioural and needs based
   segments, jobs to be done, triggers, frequency, stakes, budget, buying unit,
   switching costs. State the segments deliberately excluded and why.
8. **Map the alternatives.** Follow [competitive-map.md](competitive-map.md).
9. **Grade demand and price evidence.** Say what each signal supports and no
   more. Mention volume is not demand, search interest is not willingness to pay,
   survey intent is not purchase behaviour, competitor funding is not market
   attractiveness.
10. **Request the primary research.** Design the instrument and sampling frame,
    then emit `market.primary_research.requested` to Customer Discovery {OS}. Do
    not conduct the interviews. Continue independent desk work while it runs.
11. **Integrate what comes back.** Consume `discovery.insight.confirmed` and
    `discovery.segment.profiled`, reconcile against the desk evidence, and move
    hypothesis confidence in both directions, including down.
12. **Log the negative evidence.** Credible evidence against the idea is a first
    class record in its own ledger, not a caveat in a paragraph.
13. **Route the unsettled claims.** Any single claim that desk work and discovery
    could not settle goes to Validation {OS} for a signed test. Do not use the
    word validated here.
14. **Run the critics.** Desirability, viability, feasibility, defensibility,
    timing, ethics, data quality, sampling, incentives, regulatory, execution,
    founder market fit, pre-mortem.
15. **Decide.** Exactly one bounded decision, with conditions, kill criteria,
    next evidence, owners and an expiry date. If the evidence crosses no
    threshold in either direction, return `INSUFFICIENT EVIDENCE` with the
    cheapest next test named. Do not soften it into a conditional GO.
16. **Freeze and hand off.** On a Blueprint eligible `GO` or `PIVOT`, freeze the
    research version, produce the Blueprint Input Manifest and emit
    `market.validation.completed`. Write every canonical record through Context &
    Memory {OS}. Close with the completion status.

## Completion test

- The mode, the depth profile and its exclusions are stated at the top.
- The decision owner is named, and the success threshold was written before the
  evidence was read.
- Every material statement carries a class: FACT, MEASUREMENT, INFERENCE,
  ASSUMPTION, HYPOTHESIS, DECISION, PROPOSAL, UNKNOWN, CONFLICT, LIMITATION,
  NEGATIVE EVIDENCE or SUPERSEDED.
- Every sizing figure is reproducible from its stated inputs, with a range.
- The negative evidence ledger is non-empty, or the run states plainly that a
  genuine search for disconfirming evidence found none and describes that search.
- No interview was conducted by this OS. Every primary datum traces to a Customer
  Discovery {OS} event.
- The word "validated" appears nowhere as this OS's own verdict.
- The decision is one of the five, and carries conditions, kill criteria and an
  expiry date.
- The status is exactly one of `MARKET RESEARCH IN PROGRESS`, `MARKET RESEARCH
  BLOCKED`, or `MARKET RESEARCH COMPLETE, DECISION READY`, and the last is not
  claimed on desk research alone.
