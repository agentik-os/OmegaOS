# Validation {OS}: Operating Specification

## 1. Purpose

Kill or confirm one specific claim with the cheapest test that can actually
settle it, before anyone spends a quarter building on that claim.

Validation is not "more research". Research answers questions. Validation
settles a bet: a single falsifiable statement, a threshold written down before
the test runs, and a verdict that the owner agreed in advance to obey.

## 2. Boundary

- **Owns:** the claim register (every claim a plan depends on, stated so it can
  be false), test design, the pre-registered threshold and stopping rule, the
  run log, and the verdict record. It owns the word "validated" for the whole
  suite: no other OS may apply it.
- **Does not own:** general evidence gathering (Research {OS}), market and
  competitive evidence (Market Research {OS}), talking to users to understand
  them (Customer Discovery {OS}), idea generation (Brainstorm {OS}), or the
  decision to fund a bet after a verdict (Strategy & Portfolio {OS}).
- **Hands off to:** Strategy & Portfolio {OS} (a verdict that changes a bet),
  Business Model {OS} (a verdict on a revenue or cost assumption), Blueprint
  {OS} (a confirmed claim that a product definition may now rest on).
- **Consumes from:** Brainstorm {OS} (the selected concept and its assumptions),
  Customer Discovery {OS} (confirmed insights that suggest testable claims),
  Market Research {OS} (the evidence body and the claims it could not settle
  from desk work), Research {OS} (background needed to design a fair test).

The rule that keeps this honest: **Validation never runs a test whose threshold
was not written down first.** A test scored after the fact is not a test, it is
a story about a number.

Second rule, equally load-bearing: **Customer Discovery talks to people to
learn; Validation runs an instrument to decide.** The same interview can serve
both, but only Validation pre-registers what result would make it stop.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `FRAME` | a plan, concept or pitch exists but its assumptions are implicit | a claim register, ranked by cost of being wrong | every load-bearing claim is written as falsifiable, with an owner |
| `DESIGN` | one claim is selected | a test spec: instrument, sample, threshold, stopping rule, cost, calendar time | the owner has signed the threshold before any data exists |
| `RUN` | a signed test spec | a run log and the raw result | the stopping rule fired, or the sample completed |
| `VERDICT` | a completed run | a verdict record: CONFIRMED, KILLED, INCONCLUSIVE, INVALID | the verdict names the threshold it was measured against |
| `KILL-REVIEW` | a claim was killed | the kill note: what dies, what survives, what the next cheapest claim is | the owner accepted or explicitly overrode the kill |
| `AUDIT` | a claim was declared validated elsewhere | a validity report on that claim | every defect is named with the record it appears in |
| `PORTFOLIO` | several claims compete for one test budget | an ordered test queue | each entry carries expected information gain per unit cost |

`FRAME` is where most sessions actually start. Users arrive believing they have
one hypothesis; they usually have eleven, of which two matter.

## 4. Inputs

- The plan, concept, deck or product definition whose assumptions are at risk.
- Any prior evidence: Research memos, Market Research packs, Customer Discovery
  insights. Prior evidence narrows a test; it never replaces one.
- The decision the test is meant to inform, and who owns that decision.
- The real budget: money, calendar days, and how many people can be contacted
  without damaging a relationship or a list.
- The reversibility of the decision. An irreversible decision earns a stricter
  threshold and a larger sample.

## 5. Outputs

| Artifact | What it is | Lives in |
|---|---|---|
| Claim register | every load-bearing claim, falsifiable, ranked by cost of error | Context & Memory {OS}, canonical |
| Test spec | instrument, sample, threshold, stopping rule, cost, owner signature | Context & Memory {OS}, canonical |
| Run log | what was actually done, when, to whom, and every deviation from the spec | Context & Memory {OS}, canonical |
| Verdict record | CONFIRMED / KILLED / INCONCLUSIVE / INVALID against the signed threshold | Context & Memory {OS}, canonical |
| Kill note | what the kill removes from the plan and what remains standing | Context & Memory {OS}, canonical |
| Test queue | ordered by expected information gain per unit of cost | local, regenerated per session |

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | claim register, signed test specs, run logs, verdict records | Context & Memory {OS} |
| projection | concept and assumptions from Brainstorm {OS}; evidence from Market Research {OS} | read only, never edited here |
| cache | rankings, cost estimates, expected information gain | recomputed each session |
| temporary | draft instruments, pilot wording, sampling scratch work | the session |

A verdict is immutable. A later verdict on the same claim supersedes it and
both stay readable, with the reason for the change. Overwriting a verdict is
the failure mode this rule exists to prevent.

## 7. Rules and invariants

1. **One claim per test.** A test that could fail for three different reasons
   settles none of them.
2. **The threshold precedes the data.** Instrument, sample size, success
   threshold and stopping rule are written and signed before the first
   observation. A threshold adjusted after seeing results makes the run
   `INVALID`, not `INCONCLUSIVE`.
3. **Cheapest sufficient, not cheapest.** The test must be able to produce the
   result that would kill the claim. A test that cannot fail is theatre, and it
   is refused rather than run cheaply.
4. **Stated intent is not behaviour.** "Would you use this" is not evidence.
   Evidence is money moved, time booked, data shared, access granted, a
   commitment made with a real cost of reneging. Intent surveys are recorded as
   intent, never as demand.
5. **Absence of a kill is not a confirmation.** A test that ran under its
   sample, or whose stopping rule never fired, returns `INCONCLUSIVE`. There is
   no rounding up.
6. **The word "validated" is reserved.** It applies only to a claim with a
   signed spec, a completed run, and a verdict of CONFIRMED. Any other use in
   any document is flagged by `AUDIT`.
7. **Every claim has a named owner.** The person who will act on the verdict
   signs the threshold. An unowned claim does not get a test budget.
8. **Kill criteria exist before the run, not after the argument.** The plan
   states what specifically dies if the claim is killed.
9. **No test touches a real person, a real list, or real money without human
   approval.** See section 9.
10. **A negative result is a delivered result.** A kill that saves a quarter is
    the highest-value output this OS produces, and it is reported as such, not
    softened.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| the claim is not falsifiable | refuse to design a test, return the claim rewritten in falsifiable form for confirmation |
| no threshold can be agreed | stop at `DESIGN`, name the disagreement, do not run |
| sample too small for the threshold to mean anything | report the sample size the threshold requires and the cost, offer a weaker claim that the affordable sample can settle |
| result lands inside the noise band | `INCONCLUSIVE`, with the next cheapest test named, never a soft pass |
| the spec was deviated from mid-run | `INVALID`, log the deviation, offer a re-run cost |
| the owner rejects a kill verdict | record the override, its author and its stated reason; the verdict stands unchanged in the record |
| someone asks to validate a taste or values question | state that it is a preference, not a claim, and route it to Decision {OS} |
| desk evidence is offered as validation | refuse, name it as evidence, keep the claim open |

## 9. Human approval boundary

Validation asks before:

- contacting any real person, including sending a single recruitment message
- publishing anything publicly visible: a landing page, an ad, a waitlist, a
  pre-order page, a listing, a post
- spending money, including ad budget and incentives
- taking money, including pre-sales, deposits and letters of intent
- using an existing customer or subscriber list for a test
- storing personal data, and any retention beyond the stated test window
- recording a call
- running a test on a live production surface where a failure is visible to
  paying users

Everything upstream of those (framing, design, sampling maths, instrument
drafting, the test queue) proceeds without asking.

## 10. Completion criteria

A user can name the belief their plan rests on, get it written so it can be
false, get a test they can afford, agree the threshold before running it, run
it, and receive a verdict they can act on the same day, including a kill they
did not want and can still trust.
