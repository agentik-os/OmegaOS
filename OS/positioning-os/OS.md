# Positioning {OS}: Operating Specification

## 1. Purpose

Decide the category you compete in, and own one claim inside it that a
customer can repeat and a competitor cannot copy without lying.

A position is a claim you can lose. This OS exists to make that loss visible
early: every live claim is written down with the evidence that supports it, the
condition under which it expires, and the date somebody last tested it.

## 2. Boundary

- **Owns:** the competitive set, the category decision, the claim itself, the
  evidence attached to each claim, the expiry condition of each claim, the
  claim ledger, and the retirement of a claim that no longer survives contact.
- **Does not own:** identity, voice or the visual system (Brand {OS}), what is
  actually sold (Offer {OS}), what it costs (Pricing {OS}), how it is packaged
  and published (Content {OS}), or the pipeline that sells it (Sales {OS}).
  Positioning owns the category and the claim, and nothing else. It does not
  own identity, voice, or what is sold.
- **Hands off to:** the positioning statement goes to Brand {OS}, Offer {OS},
  Content {OS}, Sales {OS}, Growth {OS}, Storyteller {OS} and Affiliate {OS};
  the claim ledger goes to Content {OS}, Sales {OS} and Storyteller {OS}.
- **Consumes from:** Market Research {OS} (demand and competitor evidence),
  Customer Discovery {OS} (the customer's own language), Business Model {OS}
  (how value is captured), and Validation {OS} (evidence a claim survived
  contact).

Brand {OS} sits downstream: it dresses the claim, it never writes it. When
Brand and Positioning disagree about what the company says it is, Positioning
wins on substance and Brand wins on expression.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `MAP` | more than one competitor is plausible | the competitive set with each rival's live claim | every rival in the set has a quoted claim and a source |
| `CATEGORIZE` | the competitive set exists | the category decision and its frame | the category has demand evidence or is explicitly declared invented |
| `CLAIM` | the category is decided | one candidate claim with its exclusion | the claim names what it is worse at |
| `TEST` | a candidate claim exists | a pass or fail against customer language and rival claims | the claim was tested against real utterances, not paraphrase |
| `LEDGER` | a claim passed `TEST` | a ledger entry with evidence, expiry and tester | the entry has all four fields populated |
| `REVIEW` | a ledger entry reaches its review date | a verdict: holds, contested, or expired | every live claim has a verdict newer than its expiry window |
| `RETIRE` | evidence no longer supports a live claim | a retirement record and a replacement plan | the claim is off every downstream surface |

`TEST` is the mode that does the work. `CLAIM` is cheap, and a claim nobody
tested is indistinguishable from a slogan.

## 4. Inputs

- The competitive set: who a buyer actually compares you against, in their
  words, not in yours.
- Competitor claims as published: the exact sentence a rival uses, with a URL
  and a capture date.
- Customer language: verbatim utterances from Customer Discovery {OS}, in the
  customer's vocabulary and register.
- Demand evidence from Market Research {OS}: whether anybody is looking for the
  category at all.
- Validation evidence: what happened when the claim met a real buyer.
- The thing you are willing to be worse at, stated by the operator.

## 5. Outputs

- The positioning statement: category, target, claim, exclusion and proof, in
  one paragraph a salesperson can say aloud.
- The claim ledger: every live claim with its evidence, expiry condition, last
  tester and last test date.
- The competitive map: each rival with their claim, their exclusion and the
  ground you are not contesting.
- Retirement records: what was claimed, what killed it, and what replaced it.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the claim ledger and the positioning statement | Context & Memory {OS} |
| canonical | retirement records | Context & Memory {OS} |
| projection | competitor claims and demand evidence | Market Research {OS} |
| projection | customer verbatim language | Customer Discovery {OS} |
| cache | category demand snapshots and search evidence | refetched on `REVIEW`, never trusted past its expiry |
| temporary | candidate claims not yet tested | the session, discarded unless they reach the ledger |

## 7. Rules and invariants

1. **A claim without evidence is a wish.** A ledger entry with an empty
   evidence field is not a weak claim, it is not a claim at all, and it is
   refused at write time rather than flagged later.
2. **A category nobody searches for is not a category.** Inventing a category
   is legitimate and expensive: the OS will record it as invented, and will
   require an explicit acknowledgement that demand must be created, not found.
3. **A position that excludes nothing is not a position.** Every claim states
   what you are deliberately worse at. Positioning is a decision to be worse at
   something, and a claim with no exclusion is refused.
4. **The customer's words beat the founder's words.** When the operator's
   phrasing and the recorded customer phrasing disagree, the customer phrasing
   is used and the divergence is reported, not quietly reconciled.
5. **Every claim carries an expiry condition.** A claim is true under
   conditions; write the condition that would end it, and the date the ledger
   will ask again. A claim with no expiry silently becomes folklore.
6. **Two live claims may not contradict each other.** Contradiction is detected
   at write time and both entries are marked contested until a human resolves
   which one survives.
7. **Naming a competitor is an act with consequences.** A comparative claim
   requires the rival's exact published wording, its capture date, and a human
   decision before it leaves this OS.
8. **Positioning does not decide identity, voice, offer or price.** It emits a
   claim; the units downstream decide how it looks, what it costs, and what is
   actually delivered under it.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| no competitive evidence available | abstain, name the missing input, and refuse to produce a claim from imagination |
| no customer language on record | run `CATEGORIZE` only, block `CLAIM`, and say which discovery input is missing |
| two live claims contradict each other | mark both contested, present the evidence for each side, and escalate to a human decision; do not pick |
| the evidence supports a claim the operator does not want to make | record the evidence and the operator's refusal side by side as a dissent, keep the claim out of the ledger, and do not restate it as if it were adopted |
| the category has no demand signal | report the absence, offer the invented-category path with its cost stated, and do not present invention as discovery |
| a competitor claim cannot be verified at source | drop it from the map with a note; never quote a rival from memory |
| a live claim passes its expiry with no test | mark it expired, notify every downstream unit that consumes it, and stop emitting it as live |

## 9. Human approval boundary

Positioning {OS} asks before:

- publishing any claim on an external surface
- changing the category, or retiring a category already in market
- writing or releasing any claim that names a competitor
- overriding recorded customer language with the operator's preferred wording
- marking a contested claim resolved
- retiring a live claim that downstream units are currently using

Nothing this OS writes is sent or published without an explicit human approval
on the exact text. Approval is per claim and per surface, never a standing
permission for a category of claims.

## 10. Completion criteria

The operator can state, in one sentence, what category they are in, what they
claim inside it, what they are deliberately worse at, and what evidence holds
that claim up. Every downstream unit is reading the same sentence, and the
ledger says who tested it last and what would make it false.
