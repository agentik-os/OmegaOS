# Intuitive {OS}: Signal capture

**Produces:** one immutable signal record: the claim rewritten as an observably
true or false statement, its domain, a confidence from 0 to 100, the base rate,
the disconfirmer, the resolution condition, the resolution date, the capture
timestamp, and any physical state marker at capture time.

**Trigger:** the user has a pull, a doubt or a certainty and the outcome is not
yet known. Phrases: "something is off about this", "I have a feeling this
works", "write this down before I forget", "I think this falls apart". Also
fired by Decision {OS} when a call is being framed and a gut input exists, and
by Journal {OS} when an entry contains an unlogged signal.

**Runs in:** `CAPTURE`.

**Takes:** the user's claim in their own words; the domain list and calibration
record from Context & Memory {OS}; the sleep debt, stress load or acute fatigue
marker from Health & Energy {OS} where connected.

## Steps

1. Establish that the outcome is not yet known. Ask directly. If it is already
   known, the record is written marked `retrospective`, is never scored, and
   the user is told so before anything else is collected.
2. Take the claim verbatim, then rewrite it into a statement that will be
   observably true or false on a date. Read the rewrite back and get explicit
   agreement. The rewrite is what gets stored, not the original phrasing.
3. Assign the domain. Use an existing domain from the calibration record where
   one fits: a new domain starts at zero resolutions and therefore at
   `uncalibrated`, so splitting domains too finely delays every weight.
4. Ask for the confidence, an integer from 0 to 100. Refusal is allowed: the
   signal is then written with confidence absent and marked unscorable, and the
   user is told it will count for nothing at review time.
5. Ask for the base rate: how often outcomes like this one happen anyway. This
   is the reference forecast the Brier score is measured against. If the user
   cannot give one, use the domain's historical resolution rate and mark the
   base rate as derived.
6. Ask what would make this wrong. Refuse any answer that names no observable
   event. If the user cannot produce one, offer two candidate disconfirmers
   derived from the claim and ask them to pick or edit one. No disconfirmer, no
   record.
7. Set the resolution condition (the observation that settles it) and the
   resolution date (when that observation will be available). Both are
   required.
8. If the signal is about a person, check that it reads as a prediction about
   behaviour with a date, not a verdict about character. Rewrite it or refuse
   it. An unresolved suspicion is never persisted as a fact about a person.
9. Pull the state marker from Health & Energy {OS} if connected and attach it.
   It is not used to discount the signal, only to make a state pattern visible
   at review.
10. Write the record with its capture timestamp through Context & Memory {OS}
    and state plainly that it is now immutable.
11. If Decision {OS} requested this capture, return the signal packet to it
    with the domain's current tier and weight, or the label `uncalibrated`.

## Completion test

A record exists in the signal log containing all of: a claim that can be
observed true or false, a domain, a disconfirmer that names an observable
event, a resolution condition, a resolution date later than the capture
timestamp, and either a confidence from 0 to 100 or an explicit unscorable
mark. The user has been told the resolution date.

## Failure

- The outcome is already known: write the record marked `retrospective`, say
  that it will never be scored and never affect a weight, and stop.
- The user will not state a disconfirmer, and rejects both offered candidates:
  refuse to write a signal. Offer to route the note to Journal {OS} as a
  reflective entry instead, where no prediction is claimed.
- The claim has no resolution condition that could ever be observed: say so
  plainly, name it as unresolvable in principle, and do not store it as a
  signal.
- Health & Energy {OS} is absent: write the record with no state marker and
  note the gap, rather than asking the user to estimate their own physical
  state after the fact.
- Context & Memory {OS} is unavailable: do not write to a local working copy
  and call it captured. Report that the signal cannot be persisted, show the
  user the full record text so they can keep it themselves, and retry at the
  next session.
- The signal concerns self-harm, harm from another person, an acute crisis or a
  medical symptom: stop the capture and route to a qualified human professional
  or emergency services, immediately.
