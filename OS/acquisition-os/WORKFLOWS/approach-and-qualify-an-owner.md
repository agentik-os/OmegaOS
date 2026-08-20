# Approach and qualify an owner

Produces first contact with a named owner and a recorded verdict on whether
this business is genuinely for sale, before any valuation work is spent.

## Trigger

A target inside the buy box is worth contacting, either found by the search
campaign or handed over from Deal Flow {OS} as a qualified opportunity.

## Inputs

- The current buy box, and the specific reason this target fits it.
- Whatever is publicly known about the business and the owner.
- The referral path, if there is one, from Network {OS}.
- The operator's own voice: how they actually write to a stranger.
- The written thesis and kill criteria from Investment Thesis {OS}, if a thesis
  already exists for this category.

## Steps

1. Confirm the target sits inside the current buy box. If it does not, either
   decline it or amend the buy box in writing with the date and the reason.
   Never proceed on an unwritten exception.
2. Choose the approach path: direct, through a referrer, or through an
   intermediary. A referral path is recorded, because it creates an obligation.
3. Draft the approach in the operator's voice. It states who is writing and why,
   and it contains no price, no value indication and no offer.
   **Human approval gate:** the operator reviews and sends it. The OS never
   sends first contact.
4. Record the contact: date, channel, and exactly what was said.
5. On a response, run the motivation questions: why now, what happens to them
   after a sale, what their timetable is, who else is in the process, and what
   they would want the buyer to preserve.
6. Record the reason for sale in the owner's own words, dated, without
   smoothing it into something more convenient.
7. Test the answers against three failure patterns: an owner testing the market
   with no intention to sell, an owner whose price expectation is set by a
   number they heard, and a business that cannot survive the owner leaving.
8. Produce the verdict: qualified, unqualified, or too early with a stated
   review date.
9. If qualified, prepare the non-disclosure agreement.
   **Human approval gate:** a human signs it, after a lawyer has approved the
   template being used.
10. If unqualified, close the contact politely, record the reason, and return
    the target to the search list with its review date.
11. Emit `acquisition.approach.sent`, and `acquisition.target.identified` when
    the verdict is qualified.

## Completion test

The record contains: the buy box version this target was tested against, the
approach as sent and its date, the owner's reason for selling in their own
words, the answers to the motivation questions, and an explicit verdict. No
valuation work has been started unless the verdict is qualified.

## Failure modes

| Failure | What happens |
|---|---|
| the target is outside the buy box | it is declined, or the buy box is amended in writing first, never both quietly |
| the owner does not respond | a bounded follow up sequence runs, then the target returns to the list with a review date |
| the owner will not state a reason for selling | the verdict is unqualified, and valuation work does not begin |
| the owner asks for a price at first contact | decline to indicate value, explain what has to happen first, hand the question to the valuation step |
| a referrer expects a fee | stop, route the fee arrangement to a human and a lawyer, do not agree terms in a message |
| the owner sends confidential information before an agreement is signed | pause, do not read further than needed, get the agreement signed by a human first |
