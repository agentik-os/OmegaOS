# Workflow: Discovery round plan

**Produces:** a round plan naming the decision it feeds, the segment, the target
N, the stopping rule, the budget, and the consent and retention policy, approved
before anyone is contacted.

## Trigger

Any of:

- Someone says "we should talk to some users" and nobody has written down why.
- Market Research {OS} emits `market.primary_research.requested` with a gap desk
  work could not close.
- Brainstorm {OS} emits `brainstorm.concept.selected` and the concept rests on a
  pain nobody has checked with a human.
- A round is already half done with no plan behind it, and you want to know what
  the interviews already run can honestly support.

## Steps

1. **Name the decision.** Write the sentence "we will do X instead of Y
   depending on what we learn". If nobody can complete it, stop. Report that the
   round has no decision behind it and offer to help find the one that is
   actually stuck. This is a legal stop, not a failure.
2. **Write the learning goal** as a question about behaviour, not about opinion.
   "How do operations managers currently handle a failed overnight batch" rather
   than "do operations managers like our idea".
3. **Define the segment as observable behaviour.** Something the person has done
   recently and could prove. Reject definitions that select on identity or job
   title alone; they recruit people who sound right and behave differently.
4. **Set the target N and the minimum N.** The minimum is the floor below which
   no insight can be confirmed. Default floor for a confirmed insight is 3
   independent participants in the same segment; if the plan sets another number,
   the plan states why.
5. **Set the stopping rule as a number.** Default: stop after two consecutive
   interviews adding no new code to the codebook, and never before the minimum N.
   Write it now, while nobody knows what the answers will be.
6. **List the recruiting channels and label each one's bias.** Your own network,
   an existing customer list, a paid panel, cold outbound and a community all
   over-represent something different. Name what each over-represents. If the
   only viable channel is a friendly one, say the round is biased before it
   starts rather than after it produces a flattering answer.
7. **Set the budget:** money for incentives, calendar days, and how many people
   can be contacted without spending a relationship or burning a list.
8. **Write the consent and retention policy.** What is captured, where it is
   stored, who can see it, how long it is kept, and how a participant withdraws.
   This is what the participant will be told, so write it in words a person would
   actually say out loud.
9. **Declare the session type.** Pure discovery, or a concept reaction session.
   Mixing the two inside one interview contaminates the behavioural evidence, so
   if both are needed they are separate sessions and the plan says so.
10. **Route approvals.** Contact, incentives, recording, personal data storage
    and any use of the paying customer list go to the human approval boundary
    now, before recruiting starts.
11. **Write the plan to canonical state** and emit `discovery.round.planned`.

## Completion test

- The plan states a decision that changes based on the result, in one sentence.
- The segment is defined by a behaviour someone could have evidence of.
- The stopping rule is a number, not an adjective.
- The minimum N for a confirmed insight is stated.
- Every recruiting channel carries a written bias label.
- The consent and retention policy states what is kept and for how long.
- Required approvals are listed with their status, and none of them is already
  spent.
