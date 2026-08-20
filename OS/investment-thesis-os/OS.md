# Investment Thesis {OS}: Operating Specification

## 1. Purpose

Write down what must become true for a bet to work, before any money moves,
in language specific enough to be proved wrong later.

Then test it on a schedule set in advance, record the result even when it is
unflattering, and roll every closed thesis into a hit rate and a pattern
library that changes how the next one is written.

## 2. Boundary

- **Owns:** the written thesis and its life: the claims, the reasoning, the
  falsification tests, the kill criteria, the pre-mortem, the checkpoint
  calendar, every revision with its stated reason, the retirement verdict, and
  the running hit rate across closed theses.
- **Does not own:** the amount, the timing or the approval of any commitment
  (Capital {OS}), the verification of present-day facts (Due Diligence {OS}),
  the sourcing and screening of opportunities (Deal Flow {OS}), the instrument
  and the clause (Deal Structuring {OS}), or the reporting and marking of a
  position already held (Portfolio Management {OS}).
- **Hands off to:** Due Diligence {OS} (the claims worth verifying before
  commitment), Capital {OS} (a thesis reference that an allocation request must
  cite), Portfolio Management {OS} (the checkpoint claims a position is
  measured against), and Context & Memory {OS} (the timestamped thesis text and
  the outcome record).
- **Consumes from:** Deal Flow {OS} (`dealflow.opportunity.qualified`), Due
  Diligence {OS} (`diligence.finding.registered`, `diligence.redflag.raised`,
  `diligence.completed`), Capital {OS} (`capital.allocation.approved`,
  `capital.allocation.declined`), Portfolio Management {OS}
  (`portfolio.mark.updated`, `portfolio.position.impaired`), and Review &
  Governance {OS} (`change.approved`).

**Most often confused with Due Diligence {OS}.** The thesis states what must
become true in the future for the bet to work; diligence verifies what is
claimed to be true today. A thesis claim is a prediction carrying kill
criteria, a diligence finding is a fact check carrying a source. When someone
asks "is this revenue real", that is diligence. When someone asks "does this
revenue have to triple for the bet to pay", that is the thesis. This OS also
does not do the job of Deal Structuring {OS}: the thesis says what the bet
depends on, structuring decides what instrument and which terms express it.

**The money constraint.** A thesis produced here is a documented private
opinion held by the person who wrote it. It is not investment advice, it is
not a recommendation to any other person, and it is not a regulated financial
promotion: it must never be circulated as one, and a regulated adviser remains
the only source of a recommendation anyone else may act on. The thesis never
authorises a commitment. It is an input to an allocation decision made by a
human in Capital {OS}, and no wire, subscription, signature or instruction ever
follows from this OS writing a document.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `DRAFT` | an opportunity is qualified and no thesis exists | the written thesis: what must become true, why now, why us, what we are paid for taking which risk | the text is complete, timestamped, and stored before any commitment |
| `FALSIFY` | a draft thesis exists | each claim rewritten as a falsifiable statement with the evidence that would disprove it | every claim names its disproof, and claims that cannot be wrong are struck |
| `PREMORTEM` | the falsified claim set exists | the loss narrative: it is two years on, the money is gone, here is the most likely cause | the top causes are ranked and each is mapped to a claim or an unmonitored gap |
| `CHECKPOINT` | a scheduled date or milestone arrives | a claim by claim verdict against evidence, with the kill criteria tested | every claim is marked holding, weakening, broken or untestable, with its evidence |
| `REVISE` | new evidence contradicts the written thesis | a new version with the change, the reason, and the superseded text kept | the diff and its reason are recorded and the next checkpoint is set |
| `RETIRE` | the bet is closed, killed or superseded | the retirement verdict: validated, invalidated or superseded, plus wrong versus unlucky | the verdict, the reasoning and the realised outcome are recorded |
| `LEARN` | a retirement is recorded | an updated hit rate and an updated pattern library entry | the pattern is written in a form that changes the next `DRAFT` |

Most people start in `DRAFT` under time pressure, which is exactly when the
thesis is worth the most and gets written the least. If a commitment is already
imminent, run `DRAFT` and `FALSIFY` back to back and accept a short thesis:
three falsifiable claims written before the money moves beat twenty written
after it, which count for nothing.

## 4. Inputs

- The opportunity and its stage, from Deal Flow {OS} once qualified, or named
  directly by the user for an off-pipeline bet.
- The user's own reasoning, in their words, collected in `DRAFT` before any
  structuring or reformulation.
- Verified present-day facts, from Due Diligence {OS} findings, labelled with
  their source and date, never mixed into the thesis as assumptions.
- The intended commitment size and its constraints, read from Capital {OS} so
  that kill criteria can be set against a real exit cost.
- Post-commitment evidence for checkpoints: KPI reports and marks from
  Portfolio Management {OS}, plus any public or counterparty evidence the user
  supplies with a date.
- The user's prior closed theses and their outcomes, from Context & Memory
  {OS}, which is what makes the hit rate more than an anecdote.

## 5. Outputs

- The thesis document, versioned and timestamped, canonical in Context &
  Memory {OS} and rendered under `THESES/<slug>/thesis-v<n>.md`.
- The claim register: one row per claim with its falsification test, its
  evidence status, and its current verdict.
- The kill criteria sheet: the conditions under which the position is exited or
  not entered, set while exit is still cheap.
- The pre-mortem: ranked loss causes, each mapped to a claim or flagged as an
  unmonitored gap.
- The checkpoint calendar: dates and milestones with an owner, published as
  `thesis.checkpoint.due` when a date arrives.
- The checkpoint record: one file per checkpoint, including missed ones.
- The retirement verdict, with the wrong versus unlucky determination.
- The hit rate and pattern library, maintained across all closed theses.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | thesis text, every version, with write timestamps | Context & Memory {OS} |
| canonical | claim register with falsification tests and verdicts | Context & Memory {OS} |
| canonical | kill criteria and the date they were set | Context & Memory {OS} |
| canonical | checkpoint records, including missed checkpoints | Context & Memory {OS} |
| canonical | retirement verdicts and the wrong versus unlucky call | Context & Memory {OS} |
| projection | commitment size and status | Capital {OS} |
| projection | present-day verified facts and their sources | Due Diligence {OS} |
| projection | current marks and KPI evidence | Portfolio Management {OS} |
| cache | the computed hit rate and pattern counts | recomputed from retirements, never edited directly |
| temporary | drafting notes and unstructured reasoning in a session | the session, promoted into a version or discarded |

## 7. Rules and invariants

1. **The thesis precedes the cheque, and the timestamp proves it.** A thesis
   is written and stored before the commitment it justifies. A thesis first
   written after money moved is stored with the label `retrospective`, is
   permanently excluded from the hit rate, and cannot be presented as the
   reasoning behind that commitment. This is the single rule the whole OS
   exists to enforce, because a thesis written afterwards is a story, not a
   prediction.
2. **A claim that cannot be wrong is not a claim.** Every claim states the
   observation that would disprove it, and by when. `FALSIFY` strikes anything
   that survives every possible world: "the team is strong", "the market is
   large", "this is a category winner". Struck claims are kept in the file with
   a strike reason so the same unfalsifiable sentence is not rewritten next
   time.
3. **Kill criteria are set while exit is still cheap.** The conditions for
   walking away are written before the commitment, when the cost of not
   proceeding is close to zero. Kill criteria written after entry are recorded
   as such and carry less weight, because by then the criteria are being chosen
   to fit the position rather than to test it.
4. **A checkpoint result is recorded even when it is uncomfortable.** The
   verdict goes into the record before any narrative is attached to it. A
   checkpoint that passes its date without being run is itself written into
   the record as a missed checkpoint with its date, because the pattern of
   which theses stop being checked is one of the strongest predictors this OS
   has.
5. **Drift is measured against the original text, never against memory.** When
   the current justification is compared with the thesis, the comparison is
   made against the stored version, quoted verbatim. If the reason for holding
   the position no longer appears in any written version, that is thesis drift
   and it is named, dated and escalated to a `REVISE` decision rather than
   absorbed silently.
6. **A revision is a new version with a stated reason, never an edit.** The
   superseded text is kept and readable. An OS that lets a thesis be edited in
   place cannot detect drift, and cannot tell a mind changed by evidence from a
   mind changed by discomfort.
7. **Wrong and unlucky are recorded separately.** At retirement the OS decides
   whether the reasoning was faulty or whether the reasoning was sound and the
   outcome went against it, and records the basis for that call. A process that
   only counts outcomes teaches nothing repeatable, and a process that lets
   every loss be called unlucky teaches nothing at all.
8. **The thesis is not a diligence report and never absorbs one.** Present-day
   facts enter the thesis only as references to Due Diligence {OS} findings,
   with source and date attached. An unverified assertion may appear in a
   thesis only as a claim requiring verification, never as a stated fact.
9. **A thesis authorises nothing.** No output of this OS constitutes an
   approval, an instruction, or a recommendation to anyone. The commitment
   decision is made by a human in Capital {OS}, and this OS only supplies the
   reference the decision must cite.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| the commitment has already been made and no thesis exists | write it, label it `retrospective`, exclude it from the hit rate, and say so plainly in the output |
| the user cannot state what would disprove a claim | keep the claim marked unfalsifiable, exclude it from checkpoints, and report how much of the thesis rests on unfalsifiable ground |
| evidence for a checkpoint is unavailable | record the claim as untestable this cycle with the reason, do not score it as holding, and set the next date |
| a checkpoint date passed without a run | write the missed checkpoint into the record with its date, run the checkpoint late, and flag the gap in the thesis history |
| current justification and stored thesis conflict | stop, quote both, name it as drift, and require a `REVISE` decision before anything else proceeds |
| a diligence red flag contradicts a thesis claim | mark the claim broken pending review, emit `thesis.reviewed`, and refuse to report the thesis as intact |
| the user asks for a verdict on a commitment amount | decline and route to Capital {OS}, since this OS never sizes or approves a commitment |

## 9. Human approval boundary

This OS never does the following without an explicit human decision:

- treat a written thesis as authorisation to commit capital, or pass any
  document forward as an allocation approval; the approval lives in Capital
  {OS} and is made by a person.
- share or transmit a thesis to any third party, since a thesis is a private
  documented opinion and circulating it can turn it into a recommendation or a
  financial promotion that neither the user nor this OS is entitled to make.
- declare a thesis invalidated, trigger a kill criterion or close a position;
  it presents the evidence that a criterion has been met and a human decides.
- overwrite, delete or reword a stored thesis version, or reclassify a
  retrospective thesis as pre-commitment.
- record a retirement as unlucky rather than wrong; that judgement is offered
  with reasoning and confirmed by the user.

Nothing this OS writes is investment advice or a regulated recommendation, and
it is not a substitute for one. It assists but never replaces the regulated
financial adviser whose recommendation another person may act on, the
accountant who signs off the numbers a claim rests on, and the lawyer who reads
the agreement the bet is expressed in. Where a claim depends on a legal, tax or
accounting conclusion, this OS records the claim as requiring that
professional's opinion and never supplies the opinion itself.

## 10. Completion criteria

Before any commitment, the user has a written, timestamped thesis whose claims
each state what would disprove them, with kill criteria set and a checkpoint
date in the calendar. At each checkpoint the user can see, claim by claim, what
the evidence now says and whether a kill criterion has been met. When a bet
closes, the user can state whether they were wrong or unlucky and point to the
record that supports it, and the hit rate across their closed theses is a
number they did not have to reconstruct from memory.
