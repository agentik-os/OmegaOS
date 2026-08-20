# Workflow: Triage defects

**Mode:** `TRIAGE`
**Produces:** a ranked defect ledger where every entry has severity, impact,
reproduction, workaround, owner, and where required an acceptance authority.

## Trigger

Findings exist, from testing, from evaluation, from an audit, or from
production, and someone has to decide what blocks a release and what does not.

## Preconditions

- Each finding is written down with what was observed and where.
- The Blueprint requirement or Design contract each finding violates can be
  identified, or the finding is explicitly marked as violating no stated
  contract.

## Steps

1. **Separate defect from observation.** A defect violates a stated contract.
   An observation is something that could be better, and it is labelled as
   such. Mixing the two makes the ledger unreadable and the verdict soft.
2. **Reproduce.** A defect with no reproduction is recorded as unreproduced
   with its observed conditions, never closed and never treated as fact.
3. **Score severity by consequence, not by irritation.** Data loss, money,
   safety, legal exposure, security and lockout rank above anything cosmetic,
   however visible.
4. **State the impact concretely.** Who is affected, in what situation, how
   often. "Sometimes fails" is not an impact statement.
5. **Find the workaround, or state that there is none.** The absence of a
   workaround usually matters more to the release decision than the severity
   score.
6. **Assign an owner.** A person, not a team, and not the ledger itself.
7. **Route.** A code defect becomes a Stepper step for Builder {OS}. A
   definition defect becomes a decision request for Blueprint {OS}. A surface
   contract defect goes to Design {OS}. An exploitable weakness goes to
   Security {OS} and is not published elsewhere first.
8. **Decide what blocks.** Blocking defects are named as blocking. Non-blocking
   defects that stay open into the verdict need an acceptance authority: a
   named human who accepts the residual risk.
9. **Feed the verdict.** The ledger is what turns `CONFORMS` into `CONFORMS
   WITH KNOWN DEFECTS`, with the accepted list attached.

## Completion test

By inspection of the defect ledger:

- every finding is classified as defect or observation;
- every defect names the contract it violates, or is explicitly marked as
  violating no stated contract;
- every defect carries severity, impact, reproduction (or an unreproduced
  marker), workaround (or the explicit absence of one) and a named owner;
- every defect still open at verdict time carries an acceptance authority;
- every defect is routed to exactly one destination OS.

A ledger entry with an owner of "team" or a severity with no impact statement
fails this test.

## Failure paths

| What happens | What the workflow does |
|---|---|
| severity is contested | escalate to the acceptance authority rather than averaging two opinions, and record the disagreement |
| the defect only appears in production | keep it open, record the environment difference, and do not close it because the test environment is clean |
| the owner refuses the defect | record the refusal and the reason in the ledger; a rejected defect stays visible |
| a defect is an exploitable weakness | move it to Security {OS} immediately, restrict its distribution, do not put the reproduction in a public tracker |
| the ledger is being trimmed to make a release look better | refuse; the verdict names known defects, and a trimmed ledger is a fabricated verdict |
