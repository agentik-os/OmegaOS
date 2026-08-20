# Workflow: the document inventory

Produces a truthful picture of a document set: what exists, who owns it, when it
was last verified, what is duplicated and what nobody owns.

## Trigger

The document set has never been inventoried, has been inherited from someone
else, or has lost the team's trust.

## Inputs

- Every location documents actually live in, including the ones that are not
  the official one.
- The people who might own each topic.
- The questions readers are actually asking, from support threads, repeated
  messages and onboarding sessions.

## Steps

1. **Find every location.** The official store, plus the shared drive, the chat
   pins, the repository readme files and the personal notes people circulate.
   An inventory of only the official location describes a set nobody uses.
2. **List every document** with its title, its location, its last edit date and
   its apparent topic.
3. **Assign the question.** For each document, write the reader question it
   answers. Documents that answer no question are candidates for retirement.
4. **Group by question.** Two or more documents on one question is a duplicate
   cluster; record it, do not resolve it yet.
5. **Assign an owner.** A named person, not a team. Anything without one goes on
   the orphan list.
6. **Record last verified**, which is not last edited. If nobody can say when it
   was last checked against reality, it is unverified.
7. **Set a review cadence per topic.** Prices and processes go stale fast;
   architecture decisions go stale slowly.
8. **Rank the risk.** Unverified, high-traffic documents that would cause a
   wrong action are the top of the list. Everything else waits.
9. **Publish the map**, and send the orphan list to Review & Governance {OS} so
   ownership is assigned by someone with the authority to assign it.

## Completion test

- Every discovered document appears in the inventory with a location and a topic.
- Every document has an owner, or is on the orphan list.
- Every document has a last-verified date or is explicitly marked unverified.
- Duplicate clusters are recorded, with the question they compete on.
- The top ten risks are ranked by traffic multiplied by consequence of being
  wrong.

## Failure paths

| Situation | Response |
|---|---|
| a location is inaccessible | record it as unknown territory rather than assuming it is empty |
| nobody will own a document that is clearly in use | keep it, mark it unowned and unverified, and escalate; do not quietly adopt ownership |
| the set is enormous | inventory the top-traffic twenty percent first and say the rest is uninventoried |
| documents live inside a tool that cannot be exported | record their existence and location; an index entry is worth more than a copy nobody updates |
