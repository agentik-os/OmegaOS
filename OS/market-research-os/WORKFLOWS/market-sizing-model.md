# Workflow: Market sizing model

**Produces:** an auditable TAM, SAM and SOM built from named inputs, with ranges,
sensitivity and at least two independent estimation paths that are compared
rather than averaged. Emits `market.sizing.modeled`.

## Trigger

A market size is needed for a decision, or a size is already circulating in a
deck and nobody can name the inputs that produced it.

## Steps

1. **Define the boundary before any number.** Who counts as a buyer, in which
   geography, for which use occasion, over which time window, at which point in
   the value chain. Register the boundary as a `DECISION` record. Every figure
   downstream is a function of it, and a boundary changed silently later is the
   most common way a sizing model becomes fiction.
2. **Name the unit.** Annual revenue, seats, transactions, households, devices,
   procedures. Mixed units inside one model is a defect, not a rounding issue.
3. **Build the top down path.** Start from a published market figure, then state
   its own boundary, its year, its publisher and the publisher's incentive.
   Record it as a quotation of someone else's method, never as a `MEASUREMENT` of
   yours. Apply the filters that narrow it to your boundary, one at a time, each
   with its own source.
4. **Build the bottom up path.** Count the addressable units directly: firms in
   the segment, people with the trigger, transactions per period. Multiply by a
   price or value per unit that has evidence behind it. This path is usually the
   defensible one, because every input is separately checkable.
5. **Build the value based path where it applies.** What the problem costs the
   buyer today in money, time or risk, and what share of that a solution could
   credibly capture. Useful where no category exists yet and both other paths are
   circular.
6. **Compare the paths, do not average them.** If they disagree by more than a
   factor you can explain, that disagreement is the finding. Record a `CONFLICT`,
   name the axis of disagreement, and investigate the input that drives it.
7. **State every assumption as a record.** Each with a value, a source or the
   reasoning behind it, a confidence, and the validation path that would settle
   it. An assumption with no owner and no validation path is a guess wearing a
   number.
8. **Run the ranges.** Low, base and high for each material input, and the
   resulting range on the output. A point estimate with no range is not a model.
9. **Run the sensitivity.** Which single input moves the output most. Rank them.
   If one unverified assumption dominates the result, say so explicitly and route
   that assumption to Validation {OS} rather than shipping the point estimate.
10. **Derive SAM and SOM honestly.** SAM narrows TAM by what you can actually
    serve: channel access, regulation, language, capacity, product scope. SOM
    narrows SAM by what you could win in the stated horizon, given a named
    acquisition mechanism and a named constraint. A SOM that is a flat percentage
    of SAM is a placeholder and is labelled as one.
11. **Cross check against reality.** Compare against the revenue of incumbents,
    observable transaction volumes, category advertising spend, or the size of an
    adjacent market with a known ratio. State which cross checks passed and which
    did not.
12. **Record and emit.** Write the model, its inputs and its lineage to canonical
    state, and emit `market.sizing.modeled` to Business Model {OS} and Strategy &
    Portfolio {OS}.

## Completion test

- The market boundary is stated as an explicit record, and the unit is one unit.
- At least two independent estimation paths exist and their disagreement is
  reported rather than averaged away.
- Every input is traceable to a source, a first party dataset, or a labelled
  assumption with a validation path.
- Every published figure used is attributed with its own boundary, year and
  publisher, and is never presented as this study's own measurement.
- The output is a range, and the sensitivity ranking names the input that
  dominates it.
- SAM and SOM each state the specific constraint that narrows them.
- A reader can reproduce the base case from the stated inputs without asking a
  question.
- `market.sizing.modeled` has been emitted.
