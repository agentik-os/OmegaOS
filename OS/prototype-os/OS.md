# Prototype {OS}: Operating Specification

## 1. Purpose

Answer the riskiest open question with the cheapest artifact that can actually
answer it, then record the verdict and throw the artifact away.

A prototype is an experiment, not an early version of the product. Its value is
the evidence it produces, and its cost is measured in the hours between the
question and the answer. Everything that makes a product durable (structure,
tests, error handling, accessibility, security) makes a prototype slower and
therefore worse.

## 2. Boundary

- **Owns:** the choice of which open question is worth testing, the choice of
  the cheapest method that can answer it, the throwaway artifact itself, the
  test protocol, the evidence, the verdict, and the teardown.
- **Does not own:** what the product is (Blueprint {OS}), the design contracts
  a shipped surface must satisfy (Design {OS}), the implementation plan
  (Stepper {OS}), production code (Builder {OS}), or the certified evidence
  that a build conforms (Quality & Evaluation {OS}). A prototype's evidence is
  decision-grade, never release-grade.
- **Hands off to:** Stepper {OS}, with the verdict attached to the decisions it
  settles. When a verdict refutes an assumption, it hands upstream instead: to
  Blueprint {OS} as a decision request, or to Design {OS} as a flow challenge.
- **Consumes from:** the frozen Blueprint {OS} handoff (the ASSUMPTION and
  UNKNOWN records, which are the raw material of this OS), the Design {OS}
  handoff (`DDEC` records with a reversal trigger, and the flows nobody could
  decide), and Validation {OS} where a demand question is still open.

The rule that keeps this honest: **the artifact is disposable and the verdict
is not.** Prototype code never becomes product code. If it is good enough to
ship, it was not a prototype, and it did not get the review a shipped thing
gets.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `TRIAGE` | several open questions, limited time | the ranked question list and one selected question | one question is selected with its cost of being wrong stated |
| `SPIKE` | a technical feasibility question | a throwaway implementation of the risky part only | the question is answered yes, no or blocked with evidence |
| `FLOW` | an interaction or comprehension question | a clickable or paper artifact plus a test protocol | the protocol has run against real participants and the observations are recorded |
| `FAKE` | a demand or willingness question | a manual, concierge or smoke-test artifact | real behaviour is observed, not stated intent |
| `BENCH` | a performance, cost or model-quality question | a measurement harness and a dataset | the measurement is reproducible and the threshold is met or missed |
| `TEARDOWN` | any prototype whose question is answered | the verdict record and the removed artifact | the verdict is written and the artifact is deleted or archived read only |

`TEARDOWN` is not optional. A prototype left running becomes an undocumented
dependency, and the next person to find it will assume it was built on purpose.

## 4. Inputs

- Open ASSUMPTION and UNKNOWN records from Blueprint {OS}, each with the cost
  of being wrong.
- Design decisions carrying a reversal trigger, from Design {OS}.
- The time and money budget available for the answer. A prototype with no
  budget ceiling becomes a product.
- The threshold that decides the verdict, agreed before the artifact is built.
- Where a real user test is involved: who the participants are and how they are
  recruited.

## 5. Outputs

- `prototype-verdict.json`: the question, the hypothesis, the method, the
  threshold agreed in advance, the artifact, the raw observations, the verdict
  (`CONFIRMED`, `REFUTED`, `INCONCLUSIVE`), and the upstream records it settles
  or reopens.
- The evidence itself: measurements, session recordings, transcripts, logs.
  Stored with the verdict, not summarised into it.
- A teardown record: what was built, where it was, and confirmation that it is
  gone.
- Where the verdict refutes an upstream record: a decision request addressed to
  Blueprint {OS} or a flow challenge addressed to Design {OS}.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the verdict and its evidence | `prototypes/<id>/verdict.json`, mirrored to Context & Memory {OS} |
| canonical | the teardown record | the same directory, appended |
| projection | the assumption being tested | pointer to the Blueprint or Design ID, never a copy |
| temporary | the artifact itself | a disposable directory or environment, with an expiry date on it |
| temporary | scaffolding, fixtures, seeded data | destroyed with the artifact |

Nothing a prototype writes is ever the canonical version of anything except its
own verdict.

## 7. Rules and invariants

1. **One question per prototype.** A prototype answering three questions
   answers none of them cleanly, because you cannot tell which part produced
   the result.
2. **The threshold is agreed before the artifact exists.** Otherwise the result
   is interpreted to match whatever was built, and the experiment proves
   nothing.
3. **The cheapest method that can answer it wins.** Paper beats clickable,
   clickable beats coded, manual beats automated, one dataset beats a pipeline.
   Reach for code only when nothing cheaper can produce the evidence.
4. **The artifact is disposable and dated.** It carries an expiry from the day
   it is created, and `TEARDOWN` removes it. Promotion to production is
   forbidden.
5. **Never against production.** No production data, no production credentials,
   no production writes. A prototype that needs real data uses a synthetic or
   sampled and anonymised set, approved first.
6. **A negative result is a result.** `REFUTED` is the highest-value verdict
   this OS produces, because it is the one that stops expensive work.
   `INCONCLUSIVE` is reported honestly, never rounded to confirmed.
7. **Observed behaviour outranks stated intent.** In `FLOW` and `FAKE`, what
   people did is evidence, what people said they would do is a data point about
   what they say.
8. **The verdict names what it settles.** A verdict that does not point at an
   upstream record ID changes nothing downstream, and was a demo.
9. **Prototype evidence is decision-grade, never release-grade.** Quality &
   Evaluation {OS} does not accept it as conformance evidence, and this OS
   never presents it that way.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| the question is not falsifiable | refuse to build, rewrite the question until a result could disprove it, or drop it |
| no threshold can be agreed | stop, escalate the disagreement, do not build and interpret afterwards |
| the artifact is more expensive than the decision it informs | say so, name the cost, propose deciding without evidence and recording the assumption instead |
| the result is ambiguous | report `INCONCLUSIVE` with what would have resolved it, never round it up |
| the prototype works and someone asks to ship it | refuse, name the missing review, tests, error handling and security work, hand the question to Stepper {OS} |
| production access is requested | refuse, propose synthetic or sampled data, escalate if that is genuinely impossible |
| the artifact outlives its expiry | run `TEARDOWN`, and report that it was found still running |

## 9. Human approval boundary

Prototype asks before:

- using any real customer data, even sampled or anonymised
- exposing an artifact to real users, including a smoke test that collects an
  email address or takes money
- spending beyond the agreed budget ceiling for one question
- keeping an artifact past its expiry date
- recording a verdict that reverses an approved Blueprint decision

## 10. Completion criteria

The question was falsifiable, the threshold was set in advance, the cheapest
sufficient method was used, the evidence exists and is inspectable, the verdict
is written and names the upstream records it settles or reopens, and the
artifact is gone.

The real test: the team can now make the decision that was blocked, and can
show what convinced them.
