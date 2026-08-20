# Workflow: Supervise and debrief a run

Watch a running agent without babysitting it, verify its claim independently,
and turn the run into a change to the brief.

## Trigger

- An agent has been dispatched and nobody is watching it.
- An agent reports that it is finished.
- A run failed, timed out, or exhausted its budget.

## Steps

1. **Record the verification command before the run starts.** It comes from the
   brief's done test. Deciding how to verify after seeing the output is how a
   bad run gets accepted.
2. **Poll cheaply.** Capture the agent's rendered state on an interval measured
   in minutes, not seconds. The supervisor is a classifier, not a second worker.
3. **Strip your own messages from the capture before classifying.** Text sent
   into a session echoes back into it, and a classifier that matches its own
   nudge will fire forever.
4. **Classify into exactly one of four states.**
   - *Working:* the run is active. Say nothing. Silence is the correct output
     most of the time.
   - *Stalled:* the turn ended with work still available. Mechanical, so answer
     mechanically with a nudge.
   - *Blocked:* nothing is runnable. Never nudge. A nudge here is not
     persistence, it manufactures thrash. Escalate.
   - *Asking:* the agent is waiting on a judgment. Escalate to the owner and
     never answer on their behalf.
5. **Probe for available work rather than guessing** when the state is ambiguous
   between stalled and blocked. If the probe is unreadable, assume work: a wrong
   nudge is recoverable, a silent stall is not.
6. **Bound the nudges on the absence of progress.** When a progress measure
   advances, reset the budget. Stop only after N nudges that produced nothing,
   then say plainly that this needs a human. A flat cap stops a healthy long run
   for no reason.
7. **On a completion claim, run the verification command yourself.** Report the
   agent's claim and the verification result as two separate facts, so they can
   visibly disagree.
8. **On failure, capture the evidence before anything is cleaned up:** the last
   output, the failing command, the budget consumed, the state at the time.
9. **Debrief, always,** including after a clean success. Record what was
   produced, what it cost, which granted tools went unused, which brief block
   turned out to be missing, and where the agent went outside its boundary.
10. **Amend the brief or state that it stands.** An amendment names the block it
    changes. "No change" is a legitimate outcome only when it is written down
    with a reason.
11. **Remove unused grants** discovered by the debrief.
12. **Stage the debrief** to Context & Memory {OS} and send the run to Evaluation
    {OS} with its rubric attached.

## Completion test

- The verification command was fixed before the run and was run by the
  supervisor, not by the agent.
- The agent's claim and the verification result are both recorded, separately.
- Every supervision action maps to one of the four states and names the evidence
  for the classification.
- No nudge was sent to an agent classified as blocked, and no question was
  answered on the owner's behalf.
- The nudge budget was reset on real progress and its exhaustion, if reached,
  produced an escalation rather than more nudges.
- A debrief exists for this run, and it either amends the brief or records why
  it stands.
- Unused tool grants have been removed.
- The run reached Evaluation {OS} with its rubric.

A run without a debrief is an unfinished run, even when it succeeded. A success
nobody examined is where the next failure is hiding.
