# Workflow: Confirmed insight

**Produces:** insight records that each state their N, name the participants
behind them, and carry a verbatim quote per participant counted.

## Trigger

A round is coded and the saturation check has been run, or an upstream OS is
waiting on `discovery.insight.confirmed` to close a gap.

## Steps

1. **Read the codebook, not your memory of the interviews.** Candidate insights
   come from code frequency and code co-occurrence across participants, not from
   the two sessions that were most enjoyable.
2. **State each candidate insight as a claim about behaviour.** What people do,
   in what circumstance, at what cost. "Teams re-key the export by hand every
   Monday because the integration drops rows" is an insight. "Users want better
   integrations" is a summary of your own hopes.
3. **Count independent participants.** Two people from the same team, recruited
   through the same introduction, describing the same shared frustration are one
   data point plus a corroboration. Write it that way on the record.
4. **Attach a verbatim quote per participant counted.** Exact words, with the
   participant id. If the exact words are gone because the session ran on notes,
   the contribution is recorded as an observation and the insight's verbatim
   count drops accordingly.
5. **Apply the floor from the round plan.** Default 3 independent participants
   in the same segment. At or above the floor, the insight is confirmed. Below
   it, the finding stays a named anecdote with its quote, and the record states
   what N it would need to be confirmed.
6. **Label the sample bias on the insight itself,** carried from the recruiting
   channels. An insight built entirely from a founder's own network says so in
   its own record, so it still says so after somebody pastes it into a deck.
7. **Record contradicting evidence** on the same insight: participants who were
   asked and did not show it, and what they did instead. An insight with no
   record of who failed to show it cannot be checked by anyone later.
8. **Separate reported intent from observed behaviour.** Anything that came from
   a future-tense question is filed as intent and never counted toward a
   behavioural N.
9. **Extract the jobs to be done** behind the confirmed insights: the
   circumstance, the progress the person wanted, the current solution, the
   workaround, what they would fire. List hand-built workarounds first.
10. **Anonymise by default.** Quotes leave this OS without names or company
    names unless the participant explicitly permitted attribution, and that
    permission is on the record.
11. **Report the negatives.** Insights that failed to reach N, and pains that
    turned out not to be expensive enough for anyone to have done anything
    about, are delivered in exactly those words. A round that discovered the
    problem is not painful enough has done its job.
12. **Write the insight records to canonical state** and emit
    `discovery.insight.confirmed` for the confirmed set only.

## Completion test

- Every confirmed insight states an N and lists the participant ids behind it.
- Every participant counted contributes at least one verbatim quote.
- Independence is assessed, and shared-source participants are marked as
  corroboration rather than counted twice.
- Every insight carries the sample bias label of the channels it came from.
- Contradicting evidence is recorded on the insight, not omitted.
- Nothing from a future-tense question is counted as behaviour.
- Findings under the floor appear as named anecdotes with the N they would need,
  not as confirmed insights.
- Quotes leaving the OS are anonymised unless recorded permission says otherwise.
