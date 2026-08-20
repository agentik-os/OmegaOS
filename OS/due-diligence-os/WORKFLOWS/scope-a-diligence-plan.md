# Scope a diligence plan

Produces a question list where every question names the decision it could
change, a stated time and cost budget, and an explicit record of the questions
that were dropped and why.

## Trigger

An opportunity is qualified and a commitment is being contemplated, a data room
has opened, or an information request list has already gone out without a plan
behind it.

## Inputs

- The decision being contemplated and its size, from Capital {OS}.
- The thesis claims that depend on present-day facts, from Investment Thesis
  {OS}.
- The deal calendar, including any exclusivity or completion date, from
  Acquisition {OS}.
- The time and cost the user is willing to spend on diligence.
- Whatever material is already available, classified as seller sourced or
  independent before it is read.

## Steps

1. Write down the decision in one sentence, including the amount and the point
   of no return. Everything after this step is scoped against that sentence.
2. List the candidate questions from the thesis claims, the standard streams,
   and anything already noticed in the material.
3. For each question, state the decision it could change and how: price,
   structure, a condition, or the decision to proceed at all.
4. Drop every question that fails the relevance test, in front of the user,
   with the reason recorded. The dropped list is part of the plan, not deleted.
5. Rank the survivors by how much they could move the decision, not by which
   stream they belong to.
6. Set the time and cost budget against the size of the commitment and the
   calendar, and mark the point in the calendar at which unanswered questions
   become a stop rather than a delay.
7. Decide which streams the surviving questions justify opening. Do not open
   all six by reflex.
8. Identify the questions that require a named professional's written opinion:
   the accountant for quality of earnings, the lawyer for the legal opinion,
   the auditor, the technical specialist. Record who must answer each one.
   **Human approval gate:** engaging any adviser and committing to their fee is
   an explicit human decision made outside this OS.
9. Store the plan, emit `diligence.plan.set`, and hand the question list to
   `REQUEST`.

## Completion test

Every question on the plan carries a named decision it could change, the
dropped questions are recorded with reasons, a time and cost budget is stated
with the date at which unanswered questions become a stop, and every question
needing a professional opinion names the profession that must supply it.

## Failure modes

| Failure mode | What it looks like | Response |
|---|---|---|
| The data room sets the scope | the question list mirrors the folder structure | rebuild the list from the decision and the thesis claims, then map it onto the folders |
| No decision stated | diligence starts with no amount and no point of no return | stop and get the decision sentence first, since without it nothing can be scoped out |
| Everything is relevant | nothing is dropped | force the relevance statement for each question; a question whose answer changes nothing is dropped even if it is interesting |
| Budget unstated | diligence expands to the calendar | set the budget and the stop date in the plan, before the first request goes out |
| Professional opinions assumed | quality of earnings or legal comfort treated as something this OS produces | record the question, name the profession, mark it open until the written answer arrives |
| Plan written after the requests went out | the list is already with the counterparty | run the plan anyway and use it to rank which pending answers matter, and which chases to abandon |
