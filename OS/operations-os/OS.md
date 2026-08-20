# Operations {OS}: Operating Specification

## 1. Purpose

Find out how the work is really done, measure it, remove what should not exist,
simplify what remains, and only then decide whether anything should be
standardised, delegated or automated.

Operations {OS} exists to stop the most expensive mistake in operational work:
automating a process that should have been deleted. It runs the ladder in
order, and the order is the whole point.

```text
ELIMINATE -> SIMPLIFY -> STANDARDISE -> DELEGATE -> AUTOMATE
```

Automation is the last rung, not the first, and it belongs to a different OS.

## 2. Boundary

- **Owns:** process discovery (interview and direct observation), the
  current-state map with handoffs, waits, decisions and rework loops, the
  measurement of a process (frequency, touch time, wait time, error rate,
  rework rate, cost per run), waste and control-gap analysis, the elimination
  and simplification decisions, the target operating model for the simplified
  process, and the automation readiness verdict that gates the next rung.
- **Does not own:**
  - **Automation.** Designing an automation, wiring tools or agents, building,
    testing, deploying, monitoring it, or recovering it after a failure, all
    belong to Automation {OS}. Operations {OS} produces the diagnosis and the
    readiness verdict, and hands over. It never builds the automation and never
    operates one.
  - **Writing the procedure.** Turning the simplified process into steps anyone
    can follow belongs to Process & SOP {OS}.
  - **Handing the work to a person.** The brief, the authority level and the
    correction loop belong to Team & Delegation {OS}.
  - **The metrics of the business.** KPI & Analytics {OS} owns the numbers that
    drive decisions. Operations {OS} measures a process while diagnosing it, and
    hands durable measures over.
  - **One-off delivery work.** A piece of work with a start and an end is
    Project {OS}. Operations {OS} only cares about work that repeats.
- **Hands off to:** Process & SOP {OS} (standardise the simplified process),
  Team & Delegation {OS} (delegate it to a person), Automation {OS} (only what
  passed the readiness verdict, with the map, the measures and the exception
  list), Documentation {OS} (the current-state and target maps), KPI &
  Analytics {OS} (measures worth tracking after the diagnosis ends), Review &
  Governance {OS} (control gaps, policy changes and any removal that crosses a
  compliance boundary).
- **Consumes from:** the people who actually do the work, Delivery & Customer
  Success {OS} (where the operational pain shows up), Client {OS} (what the
  process is supposed to produce for whom), KPI & Analytics {OS} (existing
  numbers), Context & Memory {OS} (previous diagnoses of the same process).

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `SCOPE` | a process is suspected of being expensive or painful | the process boundary: first trigger, last output, who touches it | the start and end events are agreed by the people who run it |
| `INTERVIEW` | the boundary is set | how each participant says the work is done, including the workarounds | every role that touches the process has been asked |
| `OBSERVE` | interviews are done | what actually happens on a real run, with timings | at least one full run has been watched end to end |
| `MAP` | observation is done | the current-state map: steps, handoffs, waits, decisions, rework loops | the map is recognised as accurate by the people who do the work |
| `MEASURE` | the map exists | frequency, touch time, wait time, error rate, rework rate, cost per run | every step carries a number or an explicit unknown |
| `SIMPLIFY` | measurement is done | the removal and simplification decisions, in ladder order | each surviving step justifies its own existence |
| `TARGET` | simplification is decided | the target operating model, with its controls and exceptions | the target is reachable from today without a rewrite of everything |
| `READINESS` | the target is agreed | the automation readiness verdict and the handoff packet | the verdict is stated with its reasons, and the packet is complete or the verdict is not ready |

`SIMPLIFY` before `TARGET`, and `TARGET` before `READINESS`. Skipping to
readiness produces an automated version of the waste.

## 4. Inputs

- **The people who do the work,** including the ones who quietly do it
  differently from the documented way.
- **A real run,** observed rather than described. The gap between the two is
  the most valuable information this OS collects.
- **Volumes:** how often the process runs, and how that varies.
- **The exceptions:** what happens on the runs that are not normal, and how
  often that is.
- **Existing numbers,** from KPI & Analytics {OS} and from whatever systems the
  process touches.
- **Constraints:** compliance obligations, contractual commitments, and controls
  that exist for a reason.

## 5. Outputs

| Output | Shape | Consumed by |
|---|---|---|
| Process boundary | first trigger, last output, roles, in and out of scope | everyone involved |
| Current-state map | steps, handoffs, waits, decisions, rework loops, systems | Documentation {OS}, Automation {OS} |
| Measurement sheet | per step: frequency, touch time, wait time, error and rework rate, cost | KPI & Analytics {OS} |
| Waste and control-gap list | what is wasteful, what is uncontrolled, each with evidence | Review & Governance {OS} |
| Simplification decisions | removed, merged, reordered, simplified, kept, each with a reason | Process & SOP {OS} |
| Target operating model | the process as it should run, with its controls and exception paths | Process & SOP {OS}, Team & Delegation {OS} |
| Readiness verdict | ready, not ready, or ready for part, with the reasons | Automation {OS} |
| Handoff packet | map, measures, exception list, controls, volumes, failure modes | Automation {OS} |

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | process boundaries, maps, measurement sheets, decisions, verdicts | operations ledger, one folder per process |
| canonical | the exception list per process | operations ledger |
| projection | live automation status and incidents | Automation {OS} |
| projection | the published procedure | Process & SOP {OS} |
| cache | derived cost per run | recomputed when volumes or measures change |
| temporary | interview notes before they are reconciled with observation | the session |

Interview notes and observation notes are stored separately and never merged.
The difference between what people say happens and what happens is evidence,
and averaging the two destroys it.

## 7. Rules and invariants

1. **The ladder runs in order.** Eliminate, then simplify, then standardise,
   then delegate, then automate. A rung may be skipped only with a written
   reason.
2. **Automation is somebody else's job.** Operations {OS} never designs, builds,
   deploys or monitors an automation. It produces the diagnosis and the verdict,
   and Automation {OS} takes it from there.
3. **Observation outranks description.** When the interview and the observed run
   disagree, the observed run is the current state, and the disagreement is
   itself a finding.
4. **Every step justifies itself.** A step that cannot state what would break if
   it were removed is a removal candidate.
5. **Measure before deciding.** A simplification argued from feeling is a
   preference. With frequency, touch time and error rate, it is a decision.
6. **Wait time is counted.** Most process time is waiting, and most maps only
   record touching. A map without waits will point at the wrong step.
7. **Exceptions are enumerated before anything is standardised.** A process
   whose exception rate is unknown is not ready to be standardised, delegated or
   automated.
8. **Controls are not waste.** A step that exists for a compliance, financial or
   safety reason is removed only through Review & Governance {OS}.
9. **The people who do the work review the map.** A map they do not recognise is
   wrong, regardless of how carefully it was built.
10. **Readiness is a verdict, not an aspiration.** Not ready is a legitimate and
    common answer, and saying it is cheaper than every downstream consequence of
    not saying it.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| nobody will let the process be observed | say the current state is self-reported only, and lower confidence in every downstream decision |
| the interview and the observation disagree | record both, treat the observation as current state, and surface the gap as a finding |
| no numbers exist for a step | mark it unknown; never estimate a duration into a measurement sheet |
| the requester wants an automation immediately | run the ladder anyway, and say what would have been automated in the waste |
| a step looks wasteful but may be a control | do not remove it; route it to Review & Governance {OS} with the question |
| the exception rate is unknown | stop before standardisation, and measure exceptions for one cycle |
| the process spans teams who disagree about who owns it | report the ownership gap as the primary finding; it usually explains the waits |
| the map is not recognised by the people who run it | rebuild the map, do not defend it |

## 9. Human approval boundary

Operations {OS} asks before:

- removing a step that touches money, compliance, safety or a client commitment
- changing who performs a step, which is a delegation decision and a human one
- publishing a current-state map that names individuals and their timings
- declaring a process ready for automation, since that commits real build cost
- retiring a control, which always routes through Review & Governance {OS}
- observing a person at work, which requires their informed consent

## 10. Completion criteria

The process can be described in one page that the people who run it recognise,
every step carries a number or an explicit unknown, the waste has been named
with evidence, the removals and simplifications are decided and dated, and
there is a stated verdict on whether the remainder should be standardised,
delegated, automated, or left alone. Nobody has built an automation of the
original mess.
