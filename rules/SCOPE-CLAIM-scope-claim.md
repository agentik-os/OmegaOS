# SCOPE-CLAIM — File-lock scope claims prevent concurrent edits

**Category:** Safety
**Added:** 2026-05-26

## Rule

Workers declare files_owned on spawn. A new worker is rejected if its files overlap with an active claim. Claims auto-release on done_clean.

## Origin

Two workers editing the same file produced merge conflicts.
