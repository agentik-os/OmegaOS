# Workflow: Segment profile

**Produces:** segments defined by what people do rather than by what they look
like, each profiled with its jobs, pains, workarounds and the evidence per
claim.

## Trigger

Any of:

- Confirmed insights exist across more than one kind of participant.
- Quotes contradict each other in a patterned way rather than randomly.
- Business Model {OS} or Positioning {OS} needs a customer segment grounded in
  something other than an industry list.

## Steps

1. **Start from behaviour, not from attributes.** Group participants by what
   they actually did: the workaround they built, the trigger that made them act,
   the solution they fired, how they buy, who signs.
2. **Test each candidate grouping against the codebook.** A grouping is real when
   the codes inside it differ from the codes outside it. A grouping whose members
   share a size band but the same codes is not a segment, and it is reported as
   not a segment.
3. **Name each segment by its behaviour or its job,** never by a demographic
   bucket alone. The name is what everyone downstream will repeat, so it should
   carry the distinguishing behaviour in it.
4. **Profile each segment:** the defining behaviour, the job to be done and its
   circumstance, the pains, the workarounds, what they use today, what triggers a
   change, and who is involved in the decision.
5. **State the N per segment,** not just for the round. A round of 12 that splits
   into three segments has three groups of four, and the confidence in each
   segment is the confidence of a group of four.
6. **Carry the evidence per claim.** Every line in a profile points at the
   insight records and the quotes behind it. A profile line with no evidence
   pointer is deleted, not softened.
7. **Carry the bias label.** If a segment was recruited entirely through one
   channel, the profile says so, and says what that channel over-represents.
8. **Name the segments you did not reach.** The people who declined, who never
   answered, or who were screened out are part of the picture, and their absence
   shapes every conclusion drawn from the ones who did show up.
9. **Stop at behaviour.** Do not write a persona, a name, a photo, a day in the
   life, a tone of voice or a message. Those belong to Positioning {OS} and Brand
   {OS}, and writing them here means the marketing story is built on the research
   step instead of on the evidence.
10. **Write the profiles to canonical state** and emit
    `discovery.segment.profiled`.

## Completion test

- Each segment is separated from the others by a behaviour or a job, and the
  codebook shows that difference.
- Each segment states its own N, not the N of the whole round.
- Every claim in every profile points at an insight record and a quote.
- Each segment carries the bias label of the channel it was recruited through.
- Groups that differ only demographically are reported as not segments.
- The people the round failed to reach are named.
- The output contains no persona, no messaging and no tone of voice.
