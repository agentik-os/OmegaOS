# Publication gate

The single path by which anything leaves this OS and reaches an audience.
Nothing publishes except through it.

## Trigger

Any asset reaches its scheduled slot, or the operator asks to publish something
now.

## Steps

1. **The OS assembles the release candidate**: the exact text, the exact
   assets, the surface, the scheduled time, and the asset's stated job.
2. **The OS re-checks provenance.** Every material record in the asset has a
   source and a timestamp. Any record still staged blocks the release and is
   named.
3. **The OS re-checks the epistemic labels.** Every material claim carries an
   E1 to E5 label, and no E4 or E5 claim is phrased as settled.
4. **The OS re-checks narrative provenance.** If the asset carries a story, the
   Storyteller {OS} release verdict is re-read at publication time, not at
   drafting time, because consent can be withdrawn between the two.
5. **The OS re-checks rights**: copyright, likeness, privacy, platform rules,
   music and image licences, advertising and affiliate disclosure,
   accessibility. Any unresolved item emits `content.rights.blocked`.
6. **The OS re-checks third-party naming.** Any named customer, partner or
   individual must have a recorded consent, from Network {OS} or from the
   story object. Absent consent, the OS offers anonymisation or omission.
7. **The OS presents the release candidate to a human** for approval of the
   exact text and assets that will ship. The approval is recorded with the
   content hash of what was approved.
8. **The OS publishes** only if the approved content matches the content about
   to ship, byte for byte. A difference aborts the publication.
9. **The OS records the publication**: surface, time, approver, content hash,
   and the asset's job.
10. **The OS opens the measurement window** for `/content-review`, against the
    stated job.

## Completion test

For every published asset there exists a publication record containing: the
surface, the publication time, the approver, the content hash of the approved
text, the asset's stated job, a rights clearance, and, where applicable, a
Storyteller release verdict re-read at publication time and a consent record for
every named third party.

Published content whose hash differs from the approved hash fails this test and
is treated as an incident, not a discrepancy.

## Failure and abort

- **Any staged record in the asset:** block, name the record, do not publish on
  a staged fact.
- **Consent withdrawn between drafting and publication:** abort. This is the
  specific reason the verdict is re-read at step 4 rather than trusted from the
  draft.
- **Rights unresolved:** emit `content.rights.blocked`, do not publish, name
  what would clear it. Overriding the block is a human decision with its own
  approval.
- **Approved hash does not match the shipping content:** abort the publication
  and report the difference. Never publish an unapproved variant, however small
  the edit.
- **Human approval refused:** the asset returns to the calendar unpublished,
  with the refusal recorded. The slot is left empty rather than backfilled with
  something generic.
- **Platform rejects the publication:** record the rejection verbatim, do not
  retry with altered content without a fresh approval.
