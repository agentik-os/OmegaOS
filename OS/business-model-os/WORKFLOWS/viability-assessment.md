# Workflow: Viability assessment

**Produces:** a verdict on whether this model clears a bar the owner stated
first, with the breakeven volume set against the volume the pipeline can
plausibly produce.

## Trigger

Unit economics exist and someone is about to decide to build, fund, hire against
or continue the model. Also runs when a Validation {OS} verdict changes an input
the previous assessment rested on.

## Steps

1. **Get the bar, in writing, before anything is computed.** What this model has
   to clear to be worth doing: a gross margin, a payback period, a return on the
   capital committed, or a contribution figure per period. It is the owner's
   number, not a generic industry benchmark. If the owner will not state one,
   stop here, deliver the economics with no verdict, and record that no bar was
   set. That is a legal stop, not a failure.
2. **Restate the unit economics as they stand**, with every origin label intact.
   If an input has changed since they were computed, recompute rather than
   reasoning over a stale figure.
3. **Compute the breakeven volume:** units per period at which contribution
   covers the fixed base. State the fixed base it was computed against, since a
   breakeven volume with an unstated fixed base is unfalsifiable.
4. **Get the plausible pipeline volume from outside this OS.** Market sizing from
   Market Research {OS}, current conversion from Revenue {OS}, or the measured
   throughput of the channels named in the canvas. Record the source and the
   date. An internal target is not a pipeline volume and is refused as one.
5. **Set the two numbers side by side.** Breakeven volume against plausible
   volume, in units, in the same period. Write the gap as a number.
6. **Check the payback period against the cash available.** A payback longer
   than the runway makes a mathematically profitable model unfundable, and that
   is a different verdict than a poor ratio.
7. **Issue exactly one verdict.**
   - `VIABLE`: clears the bar, and the breakeven volume is inside what the
     pipeline can plausibly produce.
   - `VIABLE UNDER CONDITIONS`: clears the bar only if named conditions hold.
     Each condition is stated as a number with the direction it must move in.
   - `NOT VIABLE`: does not clear the bar, or requires a volume the pipeline
     cannot produce. State both numbers and what would have to change by how
     much for this to flip.
   - `INSUFFICIENT DATA`: the load-bearing inputs are all assumed. List the
     claims that would most cheaply change the verdict, in order.
8. **Do not close the gap by moving an input.** If the model only clears the bar
   after a conversion rate, a price or a retention figure is revised upward
   without new measurement, the verdict stands as it was and the size of the
   adjustment that would have been required is reported instead. This is the
   single behaviour this workflow exists to enforce.
9. **Attach the assumption register**, ranked by how much the verdict moves if
   each assumption is wrong. Emit `business_model.assumption.registered` for
   anything not already registered.
10. **Route the approvals.** Anything leaving for an investor, a board or a
    partner, and any public commitment to a revenue mechanism, goes to the human
    approval boundary before it is sent.
11. **Emit `business_model.viability.assessed`** to Strategy & Portfolio {OS} and
    Blueprint {OS}. Supersede any previous assessment rather than overwriting it,
    and record which input changed and whether it changed because something was
    measured or because someone re-guessed.

## Completion test

- The bar was written down before the assessment ran, and it is the owner's.
- The breakeven volume is stated with the fixed base it was computed against.
- The plausible pipeline volume carries an external source and a date, and is
  not an internal target.
- Both volumes appear in the same units and the same period, with the gap stated
  as a number.
- Exactly one verdict is issued, and a `VIABLE UNDER CONDITIONS` verdict states
  every condition as a number with a direction.
- A `NOT VIABLE` verdict names what would have to change by how much, and no
  input was revised to avoid issuing it.
- The assumption register is attached and ranked by impact on the verdict.
- The previous assessment, if any, is superseded and still readable, with the
  reason for the change recorded.
