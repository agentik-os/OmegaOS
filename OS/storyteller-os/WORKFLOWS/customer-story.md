# Customer story

Turn a verified customer outcome into a story that the customer has consented
to, on the surfaces they consented to, with every number traceable.

## Trigger

Delivery & Customer Success {OS} produces consented case study material, Sales
{OS} needs a proof story for a segment, or a customer volunteers a result.

## Steps

1. **Delivery & Customer Success {OS}** supplies the verified outcome evidence:
   what changed for the customer, measured how, over what period, and by whom.
2. **Storyteller {OS}** produces the evidence gap list: every claimed number
   with no source, and every outcome asserted without measurement. Nothing on
   that list enters the story.
3. **Network {OS}** supplies the relationship and consent status of the
   customer contact, including who at the customer is allowed to speak.
4. **Storyteller {OS}** runs `/interview` with the customer or with the account
   owner and produces the customer's own account of the change, in the
   customer's words, without leading questions.
5. **Storyteller {OS}** runs `/truthcheck` and produces the truth class of every
   element, separating what the customer measured from what they believe
   caused it. Attribution is an interpretation unless it was tested.
6. **Storyteller {OS}** runs `/customerstory` and produces the shaped story:
   the situation, the pressure, the choice, the change, and the evidence beside
   the claim rather than under it.
7. **Storyteller {OS}** produces the consent request naming the exact text, the
   exact surfaces, the customer's name and logo usage, and an expiry date.
8. **Customer** approves the exact text and the surface list, or requests
   changes. An approval of an earlier draft never carries to a later one.
9. **Storyteller {OS}** records the consent with `omega-story add-consent`
   including the surface scope and the expiry, and records each claim with
   `omega-story add-claim` including its truth class and source.
10. **Storyteller {OS}** runs the release verdict workflow for each intended
    surface, and hands the READY object to Content {OS} for packaging and to
    Sales {OS} as proof.
11. **Storyteller {OS}** schedules the consent expiry review, so a case study
    does not outlive the permission that allowed it.

## Completion test

The story object exists with: every number traced to a Delivery & Customer
Success {OS} measurement, every attribution labelled as measured or
interpreted, a consent record naming the exact approved text, the surface list
and an expiry date, and a release verdict per surface. A number with no source,
an attribution presented as measurement, or a consent record with no expiry
means the workflow did not complete.

## Failure and abort

- The customer will not consent, or consents only privately: the story stays in
  the bank as private, DO NOT PUBLISH is recorded per public surface, and an
  anonymised version is offered as a separate object rather than an edit of
  this one.
- The outcome cannot be measured, only felt: keep the story, label the outcome
  as interpreted, and refuse to present it as a result. A felt improvement is a
  legitimate story and an illegitimate proof.
- The customer's account and the measured evidence contradict each other: hold
  at VERIFY, present both with their truth classes, and let the operator and
  the customer resolve it. Do not publish the version that reads better.
- The customer approves and later withdraws: the consent record is updated, the
  verdict flips to DO NOT PUBLISH, and Content {OS} and Sales {OS} are notified
  with the surfaces to withdraw from.
- Consent reaches its expiry with no renewal: the story is marked expired for
  public surfaces, not quietly left live. An expired consent is a withdrawn
  consent.
