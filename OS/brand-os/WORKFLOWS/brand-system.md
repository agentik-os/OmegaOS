# Brand system

Turn a claim and an audience register into a versioned brand system: identity,
voice rules that can reject a sentence, and a visual system expressed as tokens
a real surface consumes.

## Trigger

A positioning statement exists and there is no brand system on record, a
category change invalidated the existing one, or the operator asks for a brand
system where only taste and a logo currently exist.

## Steps

1. **Brand {OS}** reads the positioning statement, the claim ledger and the
   exclusion from Positioning {OS}. If there is no statement, the workflow
   aborts here and names the blocker.
2. **Brand {OS}** reads the audience register from Customer Discovery {OS} and
   produces the vocabulary list: what the audience says, and what already reads
   to them as marketing.
3. **Brand {OS}** inventories existing artifacts (site, deck, last twenty
   published pieces, product interface) and produces the current state: what is
   already consistent, what is accidental.
4. **Brand {OS}** runs `/brand-core` and produces the identity core, with each
   element traced to a line of the positioning statement or an audience
   utterance. Untraceable elements are cut and listed as cut.
5. **Operator** confirms or corrects the identity core. This is the last point
   where the system is cheap to change.
6. **Brand {OS}** runs `/brand-voice` against the real drafts from step 3 and
   produces the rule set, each rule with the sentence it accepts and the
   sentence it refuses. Rules that refuse nothing are dropped and listed.
7. **Brand {OS}** runs `/brand-visual` and produces the visual system, each
   decision tested against small screens, print, a single colour and the
   exclusion from step 1.
8. **Brand {OS}** runs `/brand-tokens` and produces the token set with contrast
   ratios and minimum sizes computed. Any token failing its constraint is
   replaced before the set is emitted.
9. **Brand {OS}** runs `/brand-audit` over the step 3 inventory against the new
   system and produces the off system list: what is live today that the system
   would refuse.
10. **Human** approves the system as version 1, including the identity core, the
    name if it changed, and the plan for the off system list.
11. **Brand {OS}** runs `/brand-handoff` to Design {OS} (tokens and extension
    rules), Content {OS} (voice rules for editorial surfaces) and Storyteller
    {OS} (register constraints a story may not contradict).

## Completion test

A writer who has read only the system document produces a paragraph that passes
`/brand-audit` with zero rule failures, and a designer who has consumed only the
tokens produces a screen that passes the same audit. Every voice rule in the set
has both an accept and a refuse example, and every colour and type token carries
a computed contrast or size value. A rule with no refuse example, or a token
that exists only in the system document and not in the emitted token file, means
the workflow did not complete.

## Failure and abort

- No positioning statement at step 1: abort and produce nothing. A brand system
  built before the claim expresses whatever the founder happened to like that
  month, and it will be rebuilt when the claim arrives.
- No audience register at step 2: continue only on explicit operator
  instruction, mark every voice rule provisional, and schedule a review as soon
  as discovery data exists.
- Two voice rules contradict on the same sentence at step 6: present both rules,
  the sentence and the conflict, and stop. A human decides which rule survives.
- A visual direction contradicts the exclusion at step 7: refuse the direction,
  name the exclusion it violates, and return the question to Positioning {OS}
  rather than quietly widening the claim.
- Tokens fail accessibility at step 8 and no nearby value passes: the palette
  does not ship. A palette that fails contrast is abandoned by whoever has to
  build with it, which produces drift the system never sees.
