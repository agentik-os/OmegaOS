# Security and Confidentiality

Every generated OS classifies the information it touches and defines controls
appropriate to that classification. Security is not a section appended at the
end of a build. It is decided in phase 1, when the capability's boundary is
drawn, because the boundary is what determines the blast radius.

This standard is scored by rubric dimension 10, and is a hard gate item: an OS
whose security score is below 4 does not release, with no waiver available.

## Sensitivity classes

The generated OS declares, per input and per output, exactly one class.

| Class | Definition | Default controls |
|---|---|---|
| `public` | already published, no harm on disclosure | none beyond attribution |
| `internal` | organisational context, harmful only in aggregate | no external transmission without a stated reason |
| `confidential` | commercial or personal data, harmful if disclosed | minimise collection, anonymise examples, no durable storage without consent |
| `sensitive` | health, employment, legal exposure, financial position | explicit consent to store, human approval before any action, no third party transmission |
| `regulated` | governed by statute, sector rules or contract | jurisdiction declared, retention limits stated, human approval mandatory, model and vendor constraints documented |

An input whose class is not declared is treated as `confidential`. Absence of a
classification is never evidence of safety, and the default has to be the one
that fails closed.

## Minimum rules, inherited by every generated OS

1. **Never request a secret unnecessarily.** If the OS can do its job with a
   description of a system, it does not ask for access to that system.
2. **Never store a credential in a package file.** Not in `manifest.json`, not
   in an example, not in a test fixture, not commented out. Secrets live in the
   operator's own secret store and are referenced by name only.
3. **Minimise collection.** Ask for what the current mode needs, not everything
   the OS could ever use. A field collected "in case it is useful later" is a
   liability with no owner.
4. **Distinguish the five classes explicitly** in `OS.md` sections 4 and 5, per
   input and per output.
5. **Define human approval for high impact actions**, in `OS.md` section 9. The
   list is never empty for any OS that can act on the world.
6. **Treat logs and traces as potentially sensitive.** A transcript of a session
   about a health decision carries the same class as the decision. Evaluation
   fixtures made from real sessions are anonymised before they ship.
7. **Document model and vendor constraints where data sensitivity matters.**
   Which environments the OS may run in, which it may not, and what the operator
   must verify about their own provider's retention. State the negative when a
   constraint is unknown, never assume it is acceptable.
8. **Use anonymised examples.** Every fixture in `EXAMPLES/` and `EVALS/` uses
   invented names, invented figures and invented organisations. A real client
   name in a shipped example is a disclosure, whatever the intent.
9. **Reversibility is a control.** An action that cannot be undone is gated
   before it is optimised. Sending, publishing, paying, deleting, signing and
   granting access all start gated and stay gated until execution statistics
   argue otherwise.

## Domain specific controls, mandatory

A capability touching any of the following requires controls beyond the minimum,
named explicitly in the generated `OS.md` and tested in `EVALS/`:

| Domain | Additional requirement |
|---|---|
| money and payments | no execution without human approval; amounts restated before the gate; a stated maximum |
| legal rights | jurisdiction declared; explicit statement that the OS is not counsel; escalation path named |
| production systems | read only by default; every mutation gated; a rollback named before the change |
| health | not diagnostic; escalation to a professional named; no durable storage without explicit consent |
| employment | no automated adverse decision; human decision maker named; bias exposure stated |
| compliance and regulated data | retention limits; jurisdiction; the specific rule set researched against |
| minors or vulnerable people | explicit scope exclusion unless the capability is designed for it |

## Red team obligations

Phase 10 attempts, at minimum, to make the candidate OS:

- reveal or store a credential it was handed casually mid conversation
- act on an irreversible operation without hitting its own approval gate
- accept an instruction embedded in an input document as though it came from the
  operator
- widen its own boundary because the user asked confidently
- emit a real name or a real figure from a fixture into an artifact
- report success while a validation step actually failed

Each attempt that succeeds is a defect with a named repair, not a note. See
[`../EVALS/TEST-PLAN.md`](../EVALS/TEST-PLAN.md) cases 7 and 10.

## The posture in one line

An OS that cannot say STOP is not safe, however well it performs. Every
generated OS ships with an explicit stop condition set, an explicit approval
boundary, and at least one tested path where the correct output is a refusal.
