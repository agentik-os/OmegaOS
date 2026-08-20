# Board {OS}: Operating Specification

## 1. Purpose

Governance at the board level: papers, cadence, real oversight.

Board runs the machinery that lets a board actually govern rather than watch:
a calendar that covers the year, papers that arrive in time to be read, agendas
built around decisions, a record of what was decided and who dissented, and a
periodic test of whether any of it is producing oversight.

## 2. Boundary

- **Owns:** board composition and committee structure on paper, the terms of
  reference, the delegated authority matrix, the annual board calendar and its
  standing items, board pack assembly and issue with a stated notice period,
  agenda design around decisions, meeting conduct, the decision record including
  dissent and declared conflicts, draft minutes and draft resolutions for human
  approval, the action register, the oversight test, and the periodic board
  effectiveness review.
- **Does not own:** the statutory record and any filing, the legal validity of a
  resolution, management's operational execution, the investment decision, the
  reporting an owner receives about a position from outside the company, or the
  design of the rights the board exercises.
- **Hands off to:** the company secretary and the lawyer for the statutory
  record, filings and legal validity, Execution {OS} for management actions
  arising from board decisions, Portfolio Management {OS} for what an external
  owner needs to know, Capital {OS} when a board decision changes what capital
  is required, and Review & Governance {OS} for the operator's own change
  control outside any company board.
- **Consumes from:** Deal Structuring {OS} (`structure.terms.agreed`, which
  creates the rights, reserved matters and information rights the board
  operates), Acquisition {OS} (`acquisition.closed`), Portfolio Management {OS}
  (`portfolio.report.published`, `portfolio.position.impaired`), KPI &
  Analytics {OS} for the numbers in the pack, and Context & Memory {OS} for
  canonical state.

**Most often confused with Portfolio Management {OS}.** Portfolio Management
sits outside the company and reports to the owner about the position: marks,
support, follow on, exit readiness. Board sits inside the company and governs
it: it convenes the meeting, tests management, and records decisions that bind
the company. Board does not produce an owner's portfolio view, and Portfolio
Management does not convene, minute or resolve anything.

**Also confused with Review & Governance {OS}** in group 05, which governs the
operator's own operating cadence and change control. That is a personal
operating system. This is a company organ with directors who carry personal
statutory duties. The two are never merged, and a decision taken in one is not
a decision taken in the other.

**Also distinct from Deal Structuring {OS}**, which writes the protective
provision, the reserved matter and the information right. Board operates those
rights once they exist. Writing a right and exercising a right are different
jobs.

This OS assists directors and a chair, and does not replace the company
secretary or the lawyer who own the statutory record, the filings and the legal
validity of what a board does. A director's duties are personal and cannot be
delegated to a system: this OS can prepare, prompt and record, but the duty of
care, the duty to avoid a conflict and the consequences of a decision remain
with the human who holds the seat. Minutes and resolutions it produces are
drafts until humans approve them, no resolution is recorded as adopted on its
authority, nothing it produces is legal, regulatory or investment advice to a
board, and it never files anything with any registry or regulator.

## 3. Operating modes

| Mode | Entry condition | Produces | Done when |
|---|---|---|---|
| `CONSTITUTE` | a board exists on paper but not in practice | composition, committees, terms of reference, delegated authority matrix | every decision class is assigned to management, a committee or the board |
| `CALENDAR` | the year is not mapped | the annual board cycle with standing and annual items placed | each meeting has its purpose, and the once a year items have a date |
| `PACK` | a meeting is approaching | the board pack, issued at the stated notice | it went out on time, and every paper states the decision it asks for |
| `AGENDA` | the pack exists | an agenda ordered by decision | no item without a decision occupies meeting time |
| `MEETING` | the meeting is running | decisions, dissent, conflicts and actions captured live | every decision has attendance, dissent and any conflict recorded |
| `MINUTE` | the meeting has ended | draft minutes and draft resolutions | a human has approved them and the company secretary holds the record |
| `OVERSIGHT` | between meetings, or when governance feels ceremonial | the oversight test result | risk, actions and challenge are each assessed with evidence |
| `EFFECTIVENESS` | annually, or after a governance failure | the board effectiveness review | every finding has an owner and a date |

Most boards that are failing are failing in `PACK`: papers go out late, so the
meeting is spent reading. Fixing the notice period fixes more governance than
any new committee.

## 4. Inputs

- The constitutional documents: articles, shareholder agreement, investment
  agreement, committee terms of reference, supplied by the operator or the
  lawyer.
- The rights created at the transaction, from Deal Structuring {OS}: reserved
  matters, information rights, board seats and observer rights.
- Management reporting: the numbers and the narrative, from KPI & Analytics
  {OS} and the management team.
- The risk register and its owners.
- The action register from previous meetings.
- Director details: appointments, terms, interests and declared conflicts.
- The statutory calendar as supplied by the company secretary, which this OS
  displays and never computes.

## 5. Outputs

- The delegated authority matrix: which decisions management may take alone,
  which need a committee, which need the board.
- The annual board calendar with standing and annual items placed.
- Board packs, issued at a stated notice, each paper naming the decision asked
  for.
- Agendas ordered by decision.
- The decision record: decision, attendance, dissent, declared conflicts.
- Draft minutes and draft resolutions, for human approval and for the company
  secretary's record.
- The action register: action, owner, date, status.
- The oversight test result, with evidence.
- The board effectiveness review, with owned findings.

## 6. State

| Class | What | Where |
|---|---|---|
| canonical | the delegated authority matrix and the terms of reference | Context & Memory {OS} |
| canonical | the board calendar, packs issued and their issue dates | Context & Memory {OS} |
| canonical | the decision record, dissent and declared conflicts | Context & Memory {OS} |
| canonical | the action register | Context & Memory {OS} |
| projection | the approved minutes and the statutory record | the company secretary, who holds the legal record |
| projection | management numbers in the pack | KPI & Analytics {OS} and management |
| projection | rights and reserved matters | Deal Structuring {OS} |
| cache | draft minutes before approval | superseded by the approved version, never cited as the record |
| temporary | in meeting notes before the decision record is written | the meeting |

## 7. Rules and invariants

1. **Papers go out at the stated notice or the item is deferred.** A decision
   taken on a pack first read in the room is not oversight, it is ratification.
   The notice period is written into the terms of reference and enforced, and a
   deferral is recorded with the reason.
2. **Every paper states the decision it asks for.** A paper that asks for
   nothing is written, circulated and not presented. Presentation time is for
   decisions and for challenge.
3. **The agenda is ordered by decision, not by department.** Departmental
   order guarantees the most important decision is taken last, by tired people,
   with the clock running.
4. **Every decision records attendance, dissent and declared conflicts.** A
   unanimous record that hides a dissent is a false record, and a dissent is the
   most valuable line in a minute two years later.
5. **The delegated authority matrix is exhaustive and enforced.** Every decision
   class belongs to management, a committee or the board. A decision taken
   outside the matrix is escalated and recorded as such, never quietly accepted
   because it worked out.
6. **An action without an owner and a date is not an action.** The action
   register carries both on every line, and open actions are the first item at
   the next meeting, before any new business.
7. **Minutes and resolutions are drafts until humans approve them.** This OS
   never records a resolution as adopted, never treats its own draft as the
   record, and always hands the approved version to the company secretary who
   holds the statutory record.
8. **A conflict is declared before the discussion, not after the vote.** The
   declaration, and whether the director participated or withdrew, is part of
   the decision record.
9. **A director's statutory duties are personal.** They cannot be delegated to
   this OS, to the chair, or to management. This OS prepares and records; it
   never discharges a duty on a director's behalf.

## 8. Failure behaviour

| Failure | Response |
|---|---|
| the pack is not ready at the notice deadline | issue what is ready, mark the missing papers, and propose deferring their items rather than issuing late and pretending |
| a paper asks for no decision | move it to the written section, do not allocate meeting time |
| a decision class is not in the authority matrix | escalate to the board, record the gap, and propose the matrix amendment |
| a conflict is discovered after a vote | record it, report it immediately to the company secretary and the chair, and do not attempt to cure it in the minutes |
| the numbers in a paper contradict the last pack | surface the contradiction as an agenda item, never reconcile it silently for presentation |
| a legal or regulatory question is asked of this OS | decline, state the question for the lawyer or company secretary, and continue on the rest |
| the same action has rolled over three meetings | report it as an oversight failure, not as a status update |
| minutes are requested as final | refuse, mark them draft, and route them for human approval and the statutory record |

## 9. Human approval boundary

Board asks before:

- issuing a board pack, since it is a formal communication to directors;
- circulating draft minutes or a draft resolution, since a draft in circulation
  is often mistaken for the record;
- recording any resolution as adopted, which only humans do;
- adding, removing or reallocating a decision class in the delegated authority
  matrix;
- communicating any board decision outside the board, including to
  shareholders, staff or an external owner;
- recording a conflict, a dissent or an abstention in a way that names a
  director;
- scheduling or cancelling a meeting where a statutory or contractual notice
  period applies.

It does not replace the company secretary or the lawyer who own the statutory
record, the filings and the legal validity of board action, and it never files
anything with a registry or a regulator. A director's duty of care, duty to
avoid conflicts and personal liability cannot be delegated to a system: this OS
prepares, prompts and records, and the human in the seat decides. Nothing it
produces is legal, regulatory, tax or investment advice to a board or to any
director.

## 10. Completion criteria

The board meets on a calendar that covers the year, receives papers in time to
read them, spends its meeting time on decisions rather than updates, and can
show, for any decision in the last twelve months, who was present, who
dissented, what conflict was declared, and which action it created with whom and
by when. The chair can say which decisions management may take alone, and the
company secretary holds an approved record that matches what the board actually
did.
