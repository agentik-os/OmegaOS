# Customer Discovery {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

They run in order. A round plan gates recruiting, the guide gates the first
interview, coding gates the saturation verdict, and no insight is confirmed
before the round is coded.

| Workflow | Trigger | Produces |
|---|---|---|
| [discovery-round-plan.md](discovery-round-plan.md) | someone wants to talk to users, or Market Research {OS} emits `market.primary_research.requested` | a round plan naming the decision it feeds, the segment, the target N, the stopping rule, the budget, and the consent and retention policy |
| [interview-guide.md](interview-guide.md) | a round plan is approved and the first interview is scheduled | a versioned guide asking about past behaviour, including the questions that would disprove the hypothesis |
| [coded-interview-round.md](coded-interview-round.md) | at least one interview has been run, or existing recordings need to become countable | every transcript coded against one versioned codebook, with a measured saturation verdict |
| [confirmed-insight.md](confirmed-insight.md) | the round is coded and saturation has been checked | insight records with N, participant ids and a verbatim quote per participant counted |
| [segment-profile.md](segment-profile.md) | insights span more than one kind of participant, or quotes contradict in a patterned way | segments defined by behaviour, profiled with jobs, pains, workarounds and evidence per claim |
