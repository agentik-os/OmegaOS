# Mindset {OS}: Identity evidence pass

**Produces:** an updated belief ledger in which every standing identity statement carries dated evidence for it, dated evidence against it, or an explicit unevidenced mark.
**Trigger:** monthly; or when the user says something in the form "I am not really someone who ..."; or immediately after Identity Shift {OS} closes a shift and hands back a new identity model.
**Runs in:** `IDENTITY`.
**Takes:** the identity constitution, the belief ledger, the last four weekly scorecard summaries, behaviour evidence from Habit Tracker {OS}, candidate patterns proposed by Journal {OS}, and the value set from Alignment {OS}.

## Steps

1. List every standing identity statement currently held. If a statement does
   not name a condition and an observable action, rewrite it so that it does
   before scoring it. An unfalsifiable statement is not carried forward.
2. For each statement, collect the evidence from the period: what was actually
   done, on which dates. Behaviour evidence comes from Habit Tracker {OS}; if
   it is absent, use the weekly scorecard counts and mark them self-reported.
3. Class each statement: evidenced (at least one dated confirming action and no
   unexplained contradiction), contradicted (dated evidence against), or
   unevidenced (nothing happened either way).
4. For each contradicted statement, write the system diagnosis: cue, friction,
   load, competing commitment. Do not conclude that the person is not that kind
   of person. One period is not a verdict.
5. For each unevidenced statement, choose one of two outcomes and write it down:
   design one behaviour that would produce evidence in the next period, or
   retire the statement because nobody is trying to live it. Both are legitimate;
   leaving it unevidenced for a third period is not.
6. Read the candidate patterns proposed by Journal {OS}. Adopt, reject, or mark
   each as needing more evidence, and record the decision with the date. A
   pattern is never adopted silently.
7. Check every surviving statement against the value set from Alignment {OS}. A
   statement that contradicts a stated value is escalated to the user as a
   conflict; this OS does not resolve it by rewriting the value.
8. Hand every newly designed behaviour to Habit Tracker {OS} as a contract with
   one trigger, one action, one evidence test, and a floor version.
9. Write the updated ledger to the workspace and persist the adopted set through
   Context & Memory {OS}.

## Completion test

Every standing identity statement is classed evidenced, contradicted or
unevidenced, with dates; no statement has been unevidenced for three consecutive
passes; every Journal {OS} candidate pattern from the period has an adopt,
reject or pending decision recorded with its date; and every behaviour designed
in step 5 exists as a contract in Habit Tracker {OS}.

## Failure

If Habit Tracker {OS} evidence is unavailable, the pass runs on self-reported
data and every conclusion in it is labelled self-reported. If the value set from
Alignment {OS} is unavailable, step 7 is skipped and reported as skipped, not
silently passed. If a statement's evidence and the user's own account of the
period contradict each other, both are recorded and nothing is retired until the
user says which one is right.
