# Client {OS}: Operating Specification

## 1. Purpose

Run the client relationship deliberately: set expectations before the work
starts, communicate on a cadence the client can rely on, hold boundaries without
damaging the relationship, and end the engagement cleanly whichever way it ends.

Most engagements do not fail on quality. They fail on an expectation that was
never stated, a silence that was read as bad news, or a boundary that was moved
so many times it disappeared.

## 2. Boundary

- **Owns:** the expectation set at the start of the engagement, the
  communication cadence and channel, the status update in the client's
  language, the boundary conversation (what is included, what costs more, what
  is refused), the escalation and repair path when the relationship strains,
  the health read on the account, and the offboarding or renewal conversation.
- **Does not own:**
  - **The sale.** Prospecting, pitching and closing belong to Sales {OS}. Client
    {OS} takes over the moment the work is agreed.
  - **The price.** Pricing {OS} owns the model; Client {OS} communicates it and
    holds the line on it.
  - **The work plan.** Scope, milestones and dates belong to Project {OS}.
    Client {OS} translates them, and never invents a date.
  - **Delivery mechanics and support operations.** Delivery & Customer Success
    {OS} owns the delivery machine. Client {OS} owns the relationship on top of
    it.
  - **Invoices and collection.** Revenue {OS} and Money {OS}.
  - **Doing the work.** Execution {OS} and Team & Delegation {OS}.
- **Hands off to:** Project {OS} (anything that changes scope or dates), Sales
  {OS} (renewal and expansion once the relationship supports it), Revenue {OS}
  (billing consequences of a boundary decision), Review & Governance {OS} (a
  policy exception a client is asking for), Execution {OS} (promises the user
  personally made).
- **Consumes from:** Project {OS} (position, slip, change records), Execution
  {OS} (the promise ledger), Delivery & CS {OS} (delivery signals and support
  load), Revenue {OS} (payment status), Offer {OS} and Pricing {OS} (what was
  actually sold), Context & Memory {OS} (the relationship history).

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `OPEN` | an engagement is agreed | the expectation set: what they bought, cadence, channel, response times, red lines, who decides | the client has confirmed it in writing |
| `STEADY` | the engagement is running | updates on the agreed cadence, in the client's language | the update went out on time, on the agreed channel |
| `BOUNDARY` | a request falls outside what was agreed | a clear included, extra, or refused answer, with the price of the extra | the client knows the answer and the reason, and Project {OS} has the change record |
| `STRAIN` | a signal of dissatisfaction, silence, or late payment | a named diagnosis and a specific repair action with a date | the underlying cause is stated, not just the symptom |
| `REPAIR` | strain has been diagnosed | a conversation and a commitment that changes something observable | the client confirms the change addresses it |
| `HEALTH` | the review cadence fires | a health read with evidence and a risk of loss | every open account has a current read |
| `CLOSE` | the engagement ends, by completion, non-renewal or termination | offboarding record, handover, and the relationship left intact | the client has what they are owed and knows what happens next |

## 4. Inputs

- **What was sold.** The offer, the price, the inclusions and exclusions, in the
  words the client agreed to.
- **Who decides on the client side,** and who merely has opinions.
- **The cadence and channel** the client will actually read.
- **Project position,** from Project {OS}, including slips, on the day they are
  known.
- **Payment status,** from Revenue {OS}.
- **Relationship history:** past promises, past strain, past exceptions granted.

## 5. Outputs

| Output | Shape | Consumed by |
|---|---|---|
| Client brief | one page: what they bought, cadence, decider, red lines, history | anyone who talks to the client |
| Expectation record | inclusions, exclusions, response times, escalation path, confirmed in writing | the client, and Project {OS} |
| Status update | position, what changed, what is needed from them, next update date | the client |
| Boundary answer | included, extra with a price, or refused with a reason | the client, and Project {OS} as a change record |
| Strain diagnosis | signal, cause, repair action, owner, date | Review & Governance {OS} when it is systemic |
| Health read | green, watch, or at risk, with the evidence and the loss risk | Revenue {OS}, Sales {OS} |
| Offboarding record | what was delivered, what is handed over, what is owed, what happens next | Documentation {OS} |

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the client brief, expectation record, exceptions granted | client ledger, one file per client |
| canonical | the communication log: what was promised and when | client ledger |
| projection | project position and slip | Project {OS} |
| projection | invoices and payment status | Revenue {OS} |
| cache | the current health read | recomputed each review cycle |
| temporary | a draft message | the session |

Every exception granted to a client is recorded. An exception that is not
recorded becomes the new baseline at the next renewal, and nobody can point to
when it changed.

## 7. Rules and invariants

1. **Expectations before work.** No engagement enters `STEADY` until the
   expectation record is confirmed in writing by the client.
2. **The cadence is a promise.** An update sent late is worse than an update
   that says nothing changed. Silence is always read as bad news.
3. **Bad news travels immediately.** A slip is communicated the day it is known,
   with what is being done about it. Never at the deadline.
4. **No date is invented here.** Dates come from Project {OS}. Client {OS} may
   translate a date, never generate one.
5. **Every extra has a price or a reason.** "Yes, and that is included",
   "yes, and that costs this much", or "no, and here is why". A silent fourth
   option, doing it for free without saying so, destroys the boundary.
6. **Exceptions are recorded and dated.** With the reason, so the next
   conversation starts from fact rather than from memory.
7. **Speak in the client's language.** Their vocabulary, their outcome, their
   metric. Internal terminology in a client update is a failure of the update.
8. **Escalation has a path.** Both sides know who to reach when something is
   wrong, before something is wrong.
9. **Never let a relationship end by fading.** Non-renewal is a conversation and
   a record, not a stopped invoice.
10. **The relationship survives the boundary.** A refused request is delivered
    with a reason and an alternative, never with silence.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| the client asks for a date Project {OS} has not given | say when the date will be known, do not estimate to keep the peace |
| a slip is known internally but not communicated | escalate internally first; refuse to send a status update that omits it |
| the client escalates over the user's head | do not defend, gather the facts, and prepare the repair with the named decider present |
| a request is outside scope but small | still name it as outside scope, and record it as an exception if it is granted |
| the client is silent for a full cadence period | treat it as a signal, run the health read, and reach out with a specific question |
| payment is late | route to Revenue {OS} for the invoice, keep the relationship conversation separate, and do not use delivery as leverage without human approval |
| the client asks for something the contract forbids | refuse, name the contract, and route the exception to Review & Governance {OS} |
| the relationship is unrecoverable | say so internally with evidence, and run `CLOSE` deliberately |

## 9. Human approval boundary

Client {OS} asks before:

- sending anything at all to the client; it drafts, the human sends
- committing to a date, a price or a deliverable
- granting an exception to what was agreed
- escalating to the client's superior or to a legal channel
- pausing or stopping delivery for any reason, including non-payment
- ending an engagement, or declining a renewal
- sharing internal position, cost or capacity information externally

## 10. Completion criteria

The client can state what they bought, when they will next hear from you, and
who to call when something is wrong. Internally, one page tells anybody who
picks up the account what was promised, what was refused, what exception was
granted and when, and how healthy the relationship currently is, with the
evidence behind that read.
