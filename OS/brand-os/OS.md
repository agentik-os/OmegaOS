# Brand {OS}: Operating Specification

## 1. Purpose

Identity, voice and the visual system that carries them, downstream of the
claim.

Brand makes one claim recognisable across every surface a stranger meets it on.
This OS decides who you sound like, what you look like, and which sentences and
images are not allowed out, then enforces that on real artifacts.

## 2. Boundary

- **Owns:** identity (the name, the naming conventions, the archetype and the
  register), voice rules with do and do-not examples, the visual system (type,
  colour, spacing, imagery direction, logo usage), the asset library, and the
  audit that judges an artifact against the system.
- **Does not own:** the claim or the category, which belong to Positioning
  {OS}; editorial strategy, packaging, publishing and analytics, which belong
  to Content {OS}; product interface decisions, which belong to Design {OS};
  and narrative craft, which belongs to Storyteller {OS}. Brand owns identity,
  voice and the visual system. It does not own the claim and it does not own
  editorial strategy.
- **Hands off to:** the brand system goes to Content {OS}, Storyteller {OS},
  Sales {OS}, Offer {OS}, Design {OS} and Affiliate {OS}; the voice rules go to
  Content {OS}, Storyteller {OS} and Sales {OS}. Brand hands its visual system
  to Design {OS} for product surfaces and to Content {OS} for editorial
  surfaces, and it never sets editorial strategy for either of them.
- **Consumes from:** Positioning {OS} (the category and the claim) and Customer
  Discovery {OS} (the audience register, the vocabulary they read in).

Brand expresses the claim. A visual or verbal decision that would change what is
claimed belongs to Positioning {OS}, and comes back here only once the claim
itself has changed.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `IDENTITY` | a claim exists and there is no identity on record | the identity core: name, register, archetype, promise | every element traces to the claim or the audience register |
| `VOICE` | the identity core exists | voice rules, each with a do and a do-not example | every rule can reject at least one real sentence |
| `VISUAL` | the identity core exists | the visual system: type, colour, spacing, imagery, logo usage | every element is expressed as a token, not only as a picture |
| `TOKENS` | the visual system is decided | machine readable tokens for Design {OS} and Content {OS} | the tokens build and render on a real surface |
| `AUDIT` | an artifact claims to be on brand | a per rule pass or fail with the offending element named | every rule ran, and every failure cites the element |
| `ASSET` | an approved artifact needs to be reusable | an asset library entry with usage and licence | the entry states where it may and may not be used |
| `EVOLVE` | the system no longer fits the claim or the audience | a versioned change with its migration list | every surface on the migration list is corrected or waived |

`AUDIT` is the mode that makes the rest real. A voice rule nobody runs against
a live draft is decoration.

## 4. Inputs

- The positioning statement and the claim ledger from Positioning {OS}: what is
  claimed, and what is deliberately excluded.
- The audience register from Customer Discovery {OS}: how the audience talks,
  what reads as credible, what already reads as marketing.
- Existing artifacts: the site, the deck, the last twenty posts, the product
  interface, whatever already exists in the wild.
- Constraints the operator will not move on: an existing name, a legal mark, a
  founder's actual speaking voice, an accessibility floor, and the environments
  the system has to survive (small screens, print, one colour, a hostile ad
  placement).

## 5. Outputs

- The brand system: identity, naming, voice rules with examples, and the visual
  system in one artifact with a version number.
- Design tokens: type scale, colour ramps with contrast values, spacing scale,
  radii, and the imagery direction expressed as rules a human can apply.
- The asset library: logos, marks, type files, templates and imagery, each with
  usage rules and the surface it is allowed on.
- Audit reports: per rule pass or fail against a specific artifact, with the
  offending sentence, colour value or crop named.
- Migration lists on evolution: every surface carrying the old system.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the brand system and its version history | Context & Memory {OS} |
| canonical | voice rules and the asset library index | Context & Memory {OS} |
| projection | the claim, the category and the exclusion | Positioning {OS} |
| projection | the audience register | Customer Discovery {OS} |
| cache | rendered previews and contrast calculations | recomputed on every token change |
| temporary | candidate directions not yet chosen | the session, discarded unless adopted |

## 7. Rules and invariants

1. **A voice rule that cannot reject a sentence is decoration.** Every rule
   ships with a sentence it accepts and a sentence it refuses. A rule with no
   refusal example is not written down: it will never fail an audit, and so it
   will never change a draft.
2. **The visual system serves recognition, not novelty.** The test is whether a
   stranger who saw one asset recognises the next one, not whether the team is
   bored of it. Boredom inside the company arrives years before recognition
   outside it.
3. **Brand does not decide the claim.** If expression would change what is
   claimed, the work stops and goes to Positioning {OS}. Expression may
   dramatise the claim, and may never quietly widen it.
4. **Consistency beats cleverness.** Between a sharper one off and a duller
   thing the system already says, the system wins: one off brilliance costs
   recognition, and recognition is the only asset here that compounds.
5. **A token that lives only in a PDF is not a system.** Colour, type and
   spacing exist as machine readable values that a real surface consumes, with
   their contrast and size constraints computed, or they do not exist. A brand
   book with no tokens is a document about a system that was never built, and a
   palette that fails contrast is quietly abandoned by whoever has to ship it.
6. **The exclusion is expressed too.** Positioning names what you are worse at;
   the visual and verbal system must not contradict it. A system that signals
   enterprise breadth under a claim of narrow focus undoes the position.
7. **Brand never sets editorial strategy.** It says how a thing must sound and
   look. What to publish, to whom and when is Content {OS}, and Brand's verdict
   on an editorial plan is limited to voice and visual conformance.
8. **Evolution is versioned and migrated, never silent.** A changed token
   without a migration list produces a brand that is half one system and half
   another, which reads to the audience as neither.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| no positioning statement on record | run `IDENTITY` no further than the audience register, and name Positioning {OS} as the blocking input |
| no audience register available | state the absence, proceed only on explicit operator instruction, and mark every rule provisional |
| two voice rules contradict on the same sentence | present both rules, the sentence and the conflict, and stop; a human decides which rule survives |
| an artifact fails audit but the operator wants it shipped | record the exception against the artifact with a named approver and a reason, and never mark the artifact on brand |
| the system contradicts the exclusion | refuse the direction, name the exclusion it violates, and route the question back to Positioning {OS} |
| a token fails its accessibility constraint | fail the token, propose the nearest passing value, and do not ship the palette on the promise of a later fix |
| a requested asset does not exist in the library | say it does not exist and offer to create it under the system; never approximate a logo or invent a mark |

## 9. Human approval boundary

Brand {OS} asks before:

- any rebrand, or any change to the identity core
- any name change, of the company, a product or a feature
- releasing brand assets to a third party, an agency, a partner or an affiliate
- publishing anything under the identity, on any surface
- shipping an artifact that failed audit, with a named approver and a reason
- retiring or replacing a token that live surfaces already consume

Nothing this OS writes is sent or published without an explicit human approval
on the exact text and the exact asset. Approval is per artifact and per
surface, never a standing permission to publish under the identity.

## 10. Completion criteria

A new writer can produce a paragraph and a new designer can produce a screen,
without asking anyone, and both pass audit on the first attempt. A stranger who
saw one asset recognises the next one. Every rule that matters is a rule that
has actually rejected something.
