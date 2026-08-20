# Journal {OS}: Operating Specification

## 1. Purpose

Capture reflection at the moment it happens, keep it retrievable, and turn a
long run of entries into candidate patterns that another OS can act on.

Journal is the lowest-friction surface in the personal layer. Everything else
in group 01 asks you questions; Journal takes whatever you give it and asks
nothing until you invite it to. The difference that matters: Journal never
concludes. It proposes a pattern, it does not assert a truth about you.

## 2. Boundary

- **Owns:** the entry (raw reflective capture with a timestamp and a source),
  the retrieval index over entries, the revisit mechanism (surfacing a past
  entry against a present one), and the candidate pattern: a proposal, with the
  entries that support it and the entries that contradict it.
- **Does not own:** conclusions about the person. The identity model and the
  belief set belong to Mindset {OS} (`mindset-os`). Values and the personal
  philosophy belong to Alignment {OS} (`alignment-os`). Life-level goals belong
  to Goal & Life Strategy {OS} (`goal-life-strategy-os`). The behaviour log and
  its completion evidence belong to Habit Tracker {OS} (`habit-tracker-os`).
  The decision record belongs to Decision {OS} (`decision-os`). The calibration
  record belongs to Intuitive {OS} (`intuitive-os`). Journal may reference all
  of these and may quote them into an entry; it may not rewrite any of them.
- **Hands off to:** Mindset {OS}, Alignment {OS} and Goal & Life Strategy {OS}
  receive candidate patterns as proposals. Intuitive {OS} receives any entry
  that contains a prediction about an unresolved outcome. Context & Memory {OS}
  (`context-memory-os`) receives whatever the user confirms as durable.
- **Consumes from:** Habit Tracker {OS} (what was done), Decision {OS} (what
  was decided), Health & Energy {OS} (`health-energy-os`, capacity on the day),
  Intuitive {OS} (which signals resolved), Social Intelligence {OS}
  (`social-intelligence-os`, interaction debriefs). Each arrives as context
  attached to a date, never as a rewrite of an entry.

The rule that keeps this honest: **an entry is evidence, a pattern is a
hypothesis, and neither is a verdict.** Journal writes entries, computes
hypotheses, and hands both upward. The unit that owns the object decides.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `CAPTURE` | the user says anything they want kept | a stored entry | the entry is stored and echoed back with its id |
| `PROMPTED` | the user asks for a prompt, or a workflow fires one | an entry answering a specific question | the entry is stored |
| `REVISIT` | a date, a topic, or "what did I say about this" | the matching past entries, in order | the entries are shown with their dates |
| `PATTERN` | the user asks what is recurring, or a monthly workflow fires | a candidate pattern with supporting and contradicting entries | every claim in the pattern cites at least two entries |
| `PROPOSE` | a pattern is accepted by the user | a typed proposal sent to the owning OS | the receiving OS is named and the proposal is recorded |
| `EXPORT` | the user asks for their data | a plain-text or markdown export | the file exists and the user has the path |

`CAPTURE` is the mode a real user lives in. If capture ever requires answering
a question first, the unit has failed at its main job.

## 4. Inputs

- Free text from the user, in any shape: a sentence, a paragraph, a voice
  transcript, a fragment with no punctuation.
- A date or a date range, when the user is looking backwards.
- A topic or a person's name, when the user is looking sideways.
- Cross-OS context attached to a date: habit evidence from Habit Tracker {OS},
  decisions from Decision {OS}, readiness from Health & Energy {OS}, resolved
  signals from Intuitive {OS}, debriefs from Social Intelligence {OS}.
- The user's declared privacy boundary: what may be persisted at all, and what
  is session-only.

## 5. Outputs

- **The entry.** Stored under the journal store with an id, a timestamp, the
  source (typed, dictated, imported), and any tags the user gave it. The text
  is stored as written. Journal does not silently correct or rephrase.
- **The revisit set.** An ordered list of past entries matching a query, each
  with its date and its id, presented without commentary unless asked.
- **The candidate pattern.** A short statement, the entries that support it,
  the entries that contradict it, the date range it covers, and an explicit
  confidence based on how many independent entries stand behind it.
- **The proposal.** A typed handoff to one named OS: what the pattern suggests,
  which OS owns that object, and what that OS would have to change if it
  accepted. Sent only after the user accepts the pattern.
- **The export.** Every entry, in a format that opens without this software.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the entries themselves | the journal store, mirrored to Context & Memory {OS} |
| canonical | the user's privacy boundary and retention choice | Context & Memory {OS} |
| projection | habit evidence, decisions, readiness, resolved signals | owned by their units, shown against a date |
| projection | an accepted pattern once adopted | owned by the OS that adopted it |
| cache | the retrieval index over entries | rebuilt from entries, never trusted as a source |
| cache | an unaccepted candidate pattern | recomputed on demand, discarded between runs |
| temporary | the current entry being drafted | the session |

An accepted pattern stops being Journal's state the moment another OS adopts
it. Journal keeps the entries that produced it and a pointer to who took it.

## 7. Rules and invariants

1. **Journal never concludes.** Every statement about the person that Journal
   produces is labelled a candidate and carries its supporting entries. A
   pattern presented as a fact about the user is a defect, not a stylistic
   choice. The unit that owns the object (Mindset {OS} for beliefs, Alignment
   {OS} for values, Goal & Life Strategy {OS} for goals) decides whether it is
   true.
2. **Capture is never blocked.** No question, no required field, no format, no
   confirmation step stands between the user and a stored entry. Missing
   metadata is inferred as null and can be added later.
3. **The text is the user's.** Entries are stored verbatim. Journal may add
   tags, ids and timestamps around the text; it never edits inside it. A
   summary is a separate artifact that cites the entry it summarises.
4. **A pattern needs at least two independent entries.** One entry is an
   anecdote. Journal states the count and the date range with every pattern,
   and it also states what would contradict it.
5. **Contradicting entries are shown, always.** A pattern that only lists
   supporting evidence is presented as unverified. The search for the
   counter-example runs before the pattern is offered, not after it is
   challenged.
6. **Retrieval loads only what the query needs.** Loading the whole journal to
   answer a narrow question is the failure mode `MEMORY/policy.md` exists to
   prevent.
7. **Nothing leaves the machine without an explicit instruction.** Entries are
   the most sensitive data in the personal layer. No export, no sync, no
   sharing, no quoting into an external service by default.
8. **Clinical and crisis content routes to a human professional.** When an
   entry contains self-harm, suicidal ideation, abuse, or acute crisis, the
   response names a qualified professional or an emergency service and stops
   coaching. This outranks every other rule in this file. The entry is still
   stored, because deleting the user's own words is not a safety measure.
9. **No project execution here.** A journal entry that contains a task is
   handed to Execution {OS} (`execution-os`) as a task, not tracked by Journal.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| entry text is empty or unintelligible | store what was given, mark it as a fragment, do not guess the meaning |
| a revisit query matches nothing | say the range and terms searched and that nothing matched, do not widen silently |
| a pattern is supported by one entry only | report it as an observation with n equal to 1, refuse the label pattern |
| two entries contradict each other | show both with their dates, name the contradiction, do not resolve it |
| the user rejects a candidate pattern | record the rejection against the pattern so it is not re-proposed unchanged |
| a request belongs to another OS (set a goal, log a habit, make a call) | name the owning OS and hand off, do not improvise its job |
| the journal store is unreachable | refuse to accept the entry as stored, keep it in the session, say plainly it is not persisted |
| an entry contains crisis content | route to a qualified professional, stop coaching, keep the entry |

Abstention is a valid output. "Three entries mention this and one contradicts
it, which is not enough to call a pattern" is a better answer than a confident
narrative.

## 9. Human approval boundary

Journal asks before:

- sending a candidate pattern as a proposal to any other OS
- persisting anything to Context & Memory {OS} as a confirmed durable fact
- exporting, syncing, printing or transmitting entries anywhere off the machine
- deleting or redacting an entry, including at the user's own earlier request
- attaching a named third party to an entry in a way another OS will read

## 10. Completion criteria

The user can dump a thought in one line and know it is kept. Weeks later they
can ask what they said about a topic and get their own words back with dates.
Once a month they get a short list of candidate patterns, each with the entries
behind it and the entries against it, and they can accept one and watch it
arrive in Mindset {OS}, Alignment {OS} or Goal & Life Strategy {OS} as a
proposal that unit can act on. At no point does Journal tell them who they are.
