# Workflow: SOP maintenance

Produces either a new version with a change note, or a dated confirmation that
the current version still holds.

## Trigger

The review date arrives, or the work changes: a tool is replaced, a step is
simplified by Operations {OS}, a control is added by Review & Governance {OS},
an automation takes over part of the path, or the person who runs it changes.

## Inputs

- The published SOP and its version history.
- What changed in the work, and when.
- Reports from whoever has been running it, including where they deviate.
- Change feeds from Operations {OS}, Automation {OS} and Review & Governance
  {OS}.

## Steps

1. **Ask the current runner what they actually do.** Deviation from the SOP is
   normal and informative. Either the SOP is wrong or the deviation is a defect;
   both need a decision.
2. **Check the change feeds.** A step that Operations {OS} removed or that
   Automation {OS} now performs makes part of this document actively wrong,
   which is worse than out of date.
3. **Verify the prerequisites.** Tools, access and permissions drift faster than
   steps, and a wrong prerequisite blocks a run at minute one.
4. **Verify the quality bar** still matches what the consumer of the output
   expects.
5. **Decide the outcome:** confirmed, amended, rewritten, or retired.
6. **If amended, version it.** Increment the version, write the change note with
   what changed and why, and keep every prior version.
7. **If a step that touches money, safety, compliance or a client commitment
   changed,** get human approval and route to Review & Governance {OS}.
8. **If the work has been automated,** retire the human SOP or reduce it to the
   exception path, and say clearly which parts a human still owns.
9. **Retest if the change is substantial.** A rewritten decision branch deserves
   a novice test as much as the original did.
10. **Set the next review date** and confirm the owner still accepts the role.
    Owners change jobs, and an SOP owned by someone who left is an orphan.

## Completion test

- The SOP is marked confirmed, amended, rewritten or retired, with a date.
- The current runner's actual practice was compared against the document.
- Prerequisites and the quality bar were verified, not assumed.
- Any amendment carries a version increment and a change note.
- Steps touching money, safety, compliance or client commitments were approved
  by a human.
- The next review date is set and the owner still accepts.

## Failure paths

| Situation | Response |
|---|---|
| the owner has left | mark the SOP orphaned and unverified, and route ownership to Review & Governance {OS} |
| everyone deviates from the SOP in the same way | the SOP is wrong; adopt the deviation after checking it against the quality bar |
| the work changed so much the SOP is unrecognisable | do not patch it; return to Operations {OS}, then capture and test again from scratch |
| nobody has run it since the last review | question whether the procedure is still needed at all before spending time maintaining it |
