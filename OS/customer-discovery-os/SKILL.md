---
name: customer-discovery-os
description: Talk to real users and extract what they actually need. Customer Discovery {OS}, unit 17 of the AGENTIK {OS} suite (02 · DISCOVER & DECIDE). Use when the user asks about customer discovery or invokes /customer-discovery-os.
---

# Customer Discovery {OS}

Talk to real people about what they already did, and code the result so someone
else can act on it.

## When to use this

Reach for Customer Discovery when:

- You are about to build something and nobody on the team has spoken to a user
  about it in the last month.
- You have a concept from a brainstorm and you want to know whether the pain it
  addresses is real, expensive and already being worked around.
- Market Research came back with a gap desk work cannot close: what people
  actually do inside the workflow, why they switched, what they hacked together.
- You have twenty interviews already recorded and nothing coded, so every
  meeting argues about whose favourite quote wins.
- Someone says "users want X" and cannot name a single person who did anything
  about X.
- You need to know whether you are looking at one market or three, and the
  difference is behavioural rather than demographic.
- You inherited a research deck and want to know how many people were actually
  spoken to, how they were recruited, and what was measured.

Near neighbours, and the line between them:

| Confused with | Difference |
|---|---|
| Market Research {OS} | Market Research is the market and customer evidence body: sizing, segments, competition, pricing evidence, and a bounded market decision. It never conducts an interview. It requests one from this unit and consumes the coded result. |
| Validation {OS} | Discovery talks to people to learn, and may change its questions mid round because it learned something. Validation runs an instrument to decide, with a threshold signed before the data exists, and may never change it. The same interview can serve both; only Validation pre-registers what result would stop it. |
| Research {OS} | Research answers a stated question with defensible outside sources: papers, reports, filings, public data. Discovery generates primary evidence that does not exist anywhere until you go and get it. |
| Brainstorm {OS} | Brainstorm invents and evolves concepts and converges on one. Discovery does not generate ideas; it supplies the pains, jobs and workarounds that make a brainstorm about something real. |
| Positioning {OS} | Positioning writes the marketing persona, the category and the message. Discovery supplies the behavioural segment profile it is built from. A segment profile describes what people do; a persona is a communication device. |
| Delivery & Customer Success {OS} | That unit talks to existing paying customers about the product they are using now, to keep them. Discovery talks to people to learn what is true about their work, and contacting a paying customer for discovery spends a relationship that unit owns. Approval required, every time. |

## Capabilities

- Turn a vague "we should talk to users" into a round plan with a learning goal,
  a named decision, a target N and a stopping rule that is a number.
- Write a screener that selects on observable behaviour rather than on job title,
  including the answers that disqualify.
- Plan recruiting across the channels actually available, label the bias of each,
  and say what the resulting sample can and cannot support.
- Draft recruiting messages that state honestly what the conversation is and is
  not, including the incentive.
- Write an interview guide in past tense: opening, behaviour questions, probes
  that dig for the last occurrence, kill questions that would disprove your own
  hypothesis, and a closing that asks for the next introduction.
- Run the interview: stay quiet, follow the surprise, never demo, never defend.
- Handle consent and recording explicitly, and continue on notes when recording
  is refused.
- Capture a debrief within the hour, while what surprised you is still legible.
- Transcribe, attribute turns, and keep the verbatim wording intact.
- Build and version a codebook, code every transcript against it, and keep a
  code's definition stable across rounds so counts mean something.
- Measure saturation with a code growth curve and report whether the stopping
  rule actually fired.
- Produce insight records with N, participant ids and a verbatim quote per
  participant counted, and demote anything under N to a named anecdote.
- Extract jobs to be done: the circumstance, the progress the person is trying to
  make, what they use today, what they hacked together, what they are firing.
- Build segment profiles that differ on behaviour and job rather than on
  demographics.
- Audit someone else's research: how many people, recruited how, asked what, and
  what the claims actually rest on.

## Procedure

1. **Define the learning goal and the decision it feeds.** Write the sentence
   "we will do X instead of Y depending on what we learn". If that sentence
   cannot be written, stop here and say so. A round with no decision behind it
   produces slides.
2. **Define the screener.** Select on something observable and recent. "Has
   personally cancelled a vendor contract in the last six months" is a screener.
   "Decision maker" is a hope. State the disqualifying answers, including the
   ones that catch professional panel respondents.
3. **Plan recruiting and label the bias.** List the channels available, what
   each will over-represent, and how many participants each can realistically
   yield. Set the target N and the stopping rule now, before anyone is
   contacted. Route the first contact to human approval.
4. **Write the guide in past tense.** Open with an easy factual question about
   their work. Then the last occurrence of the behaviour you care about, walked
   through step by step: what happened, what they did, what it cost, what they
   tried before. Add probes ("what did you do next", "how did you handle it",
   "what would have happened if you did nothing"). Add at least two kill
   questions designed to disprove your own hypothesis. Close by asking who else
   they know with the same problem.
5. **Get consent.** Before the recorder starts: what is captured, where it goes,
   who sees it, how long it is kept, and that they can withdraw. Record the
   answer. If they decline recording, mark the session as notes only.
6. **Run it.** Target roughly 80 percent of the talking from them. Follow every
   surprise off the guide, then come back. Ask for the story, not the opinion.
   Never demo, never pitch, never defend. When you hear a judgement ("it is a
   nightmare"), ask for the last instance of it.
7. **Debrief within the hour.** Three things that surprised you, three quotes you
   want to keep, and what you would change in the guide. Memory of an interview
   degrades faster than anyone believes, and a debrief written the next morning
   is a reconstruction.
8. **Transcribe.** Keep the participant's words intact, including the hedges and
   the contradictions. Attribute turns. A cleaned-up transcript destroys the
   evidence you came for.
9. **Code.** Code every transcript against the versioned codebook. Add a new code
   when something genuinely does not fit, with a written definition, and never
   silently widen an existing code's meaning to absorb it.
10. **Check saturation.** Plot new codes per interview. Stop when the rule in the
    plan fires: by default two consecutive interviews adding no new code, and
    never before the minimum N. If it did not fire, say the round is unsaturated
    and say it in every artifact that leaves this OS.
11. **Confirm insights with counts and quotes.** For each candidate insight,
    count independent participants, list their ids, and attach one verbatim quote
    per participant counted. Under the floor, it stays a named anecdote. Emit
    `discovery.insight.confirmed` only for the ones that reach N.
12. **Extract jobs to be done.** For each confirmed pain, write the circumstance,
    the progress the person wanted, the current solution, the workaround and what
    they would fire. A workaround someone built by hand is the strongest signal
    in the whole round: it is a person paying in their own time.
13. **Profile segments.** Group participants by behaviour and job, not by
    industry or size. A segment is real when its members do something different,
    not when they look different. Emit `discovery.segment.profiled`.
14. **Hand off.** Coded result back to whoever requested it, insights sharp
    enough to be falsifiable to Validation {OS}, segment profiles to Business
    Model {OS} and Positioning {OS}, jobs to Blueprint {OS}. Write everything
    canonical to Context & Memory {OS} so the next round continues the codebook
    instead of restarting it.

## Handoffs

| To | What it receives | What it does with it |
|---|---|---|
| Market Research {OS} | `discovery.insight.confirmed`, `discovery.segment.profiled` | folds primary evidence into the market evidence body and closes the gap that produced `market.primary_research.requested` |
| Validation {OS} | `discovery.insight.confirmed` | turns an insight into a falsifiable claim and designs a test with a signed threshold |
| Brainstorm {OS} | `discovery.insight.confirmed` | uses pains, jobs and workarounds as generative material for concepts |
| Blueprint {OS} | `discovery.insight.confirmed` | writes jobs to be done into the product definition as evidence rather than assumption |
| Business Model {OS} | `discovery.insight.confirmed`, `discovery.segment.profiled` | grounds the customer segment and value proposition blocks in observed behaviour |
| Positioning {OS} | `discovery.segment.profiled` | builds the marketing persona and message from the behavioural segment |
| Context & Memory {OS} | `discovery.round.planned`, `discovery.interview.recorded` | keeps plans, sessions, transcripts and the codebook durable across rounds and OS units |

Received from: Market Research {OS} (`market.primary_research.requested`),
Brainstorm {OS} (`brainstorm.concept.selected`), Validation {OS}
(`validation.test.signed`, when Validation borrows an interview as its
instrument and this unit runs it under the frozen protocol).
