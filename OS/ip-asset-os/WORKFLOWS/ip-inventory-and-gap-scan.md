# Workflow: IP inventory and gap scan

Produces the asset register and, more importantly, the list of assets you cannot
prove you own.

## Trigger

Any of:

- First run of this OS.
- A diligence request, funding round, partnership or sale conversation begins.
- A new contributor (contractor, agency, employee, co-founder) has produced work.
- Quarterly, on the calendar.

## Steps

1. **Fix the perimeter.** Ask which jurisdictions the user operates, sells and
   would enforce in. Record them. Do not default: IP rights are territorial, and
   a defaulted jurisdiction produces a confident wrong answer.
2. **Pull the entity list.** Read `ownership.entity.registered` from Ownership
   {OS}. These are the only holders an asset may be attached to. If no entity is
   registered, note that assets will be held personally and say so explicitly.
3. **Sweep by category.** Walk each category and list what exists:
   marks and logos in use; product and feature names; copyright works (writing,
   video, audio, design, courses); code repositories; datasets and trained
   models; domains; registered designs; patents and applications; processes held
   secret; durable physical assets. Use what Brand {OS} and Context & Memory
   {OS} already hold so the user is not asked twice.
4. **Create a row per asset.** Type, description, creation date, holder of
   record, jurisdictions of use. An asset attached to an entity that Ownership
   {OS} does not know is held out of the register until the entity is confirmed.
5. **Run the title check on every row.** For each asset, ask who made it, when,
   under what agreement, and name the document that proves the answer:
   assignment deed, employment invention clause, contractor IP term, purchase
   agreement, registration certificate.
6. **Mark the status honestly.** `proven` only where the document exists and the
   user has confirmed it. `unproven` where it does not, with the missing document
   and its likely holder named. `disputed` where two claims collide. The OS does
   not upgrade a status because the user is confident.
7. **Flag encumbrances.** For codebases and models, list the open-source licence
   obligations carried by what ships. For anything commissioned, note whether the
   commission transferred rights or only a licence to use.
8. **Read the protection posture.** For each asset: registered or not, where,
   under which number, with which status and renewal date, and the date that
   status was last verified. Anything unverified is stored `unverified`.
9. **Rank the gaps.** Order by exposure: an unproven asset that is central to the
   business and already licensed out is a different problem from an unproven blog
   post. Name the top gaps and the professional act each needs.
10. **Emit.** `ipasset.registered` per asset added. Send the ranked gap list to
    Execution {OS} as tasks with named human owners, and prepare a counsel brief
    for the gaps that need a lawyer. Ask before instructing anyone.

## Completion test

Every asset the user can name has a row; every row has a title status of
`proven`, `unproven` or `disputed`; every `unproven` row names the missing
document and who holds it; every registration status carries the date it was
read; and the ranked gap list has been reviewed by the user. A register with no
`unproven` rows on a first run is a failed run, not a clean one.
