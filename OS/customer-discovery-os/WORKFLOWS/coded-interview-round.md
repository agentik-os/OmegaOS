# Workflow: Coded interview round

**Produces:** every transcript of the round coded against one versioned
codebook, with a measured saturation verdict.

## Trigger

At least one interview has been run, or a pile of existing recordings and notes
needs to become something countable.

## Steps

1. **Debrief within the hour of each session.** Three things that surprised you,
   the quotes you want to keep, and what you would change in the guide. A session
   with nothing surprising in it is flagged: the working assumption is that you
   talked and they agreed.
2. **Transcribe.** Attribute speaker turns and keep the participant's exact
   wording, including hedges, self-corrections and contradictions. Do not tidy
   the language. The wording is the evidence.
3. **Record the session metadata:** participant id, date, guide version, consent
   status, recording status, recruiting source and its bias label. A session that
   ran on notes only is marked as lacking verbatim evidence.
4. **Open or continue the codebook.** Continue the codebook from prior rounds
   where one exists. Restarting a codebook per round destroys every count that
   crosses rounds.
5. **Code the first transcript.** Work through it in order. Every segment is
   either coded or explicitly marked as noise. Skipping ahead to the parts you
   remember liking is how a round confirms whatever you already believed.
6. **Add codes with written definitions.** A new code is added only when
   something genuinely does not fit an existing one, and it arrives with a
   definition and the date it was added. Never widen an existing code's meaning
   silently to absorb a new case: that is how two things become one number.
7. **Code as you go, not in one batch at the end.** Coding after each interview
   is what makes the saturation curve measurable while the round is still running
   and still cheap to change.
8. **Record new codes per interview.** After each transcript, log how many codes
   were added. This is the saturation curve.
9. **Check the stopping rule** from the round plan after the minimum N is
   reached. Default: two consecutive interviews adding no new code. When it
   fires, stop recruiting. When it does not, either keep going or stop and label
   the round unsaturated. There is no third option and no rounding up.
10. **Flag contaminated sessions.** Any session where the interviewer demoed,
    pitched or defended the idea is marked, and everything after that moment is
    excluded from behavioural counts while remaining readable.
11. **Reconcile contradictions rather than resolving them.** When participants
    contradict each other in a patterned way, that is a signal of more than one
    segment, and it is carried forward to segment profiling instead of averaged
    away.
12. **Write transcripts, codebook and session records to canonical state** and
    emit `discovery.interview.recorded` per session.

## Completion test

- Every session has a debrief written the same day it happened.
- Every transcript is fully coded, with noise explicitly marked as noise.
- The codebook has one version number, and every code has a written definition
  and an added-on date.
- The saturation curve exists as numbers per interview, not as an impression.
- The round is labelled saturated or unsaturated, and the label matches the rule
  written in the plan.
- Contaminated sessions are flagged and excluded from behavioural counts.
- Every session record names the guide version it was run against and the
  recruiting source with its bias label.
