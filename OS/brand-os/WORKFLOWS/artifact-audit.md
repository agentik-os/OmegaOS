# Artifact audit

Judge one concrete artifact against the brand system, rule by rule, and return
a verdict that names the offending element rather than a score.

## Trigger

Any artifact is about to be released under the identity: a landing page, a
deck, a post, a screen, an ad, an email, a partner asset. Also triggered when
somebody claims an artifact is on brand and nobody has checked.

## Steps

1. **Requester** submits the artifact and states the surface it is going out
   on, because the same asset can pass on one surface and fail on another.
2. **Brand {OS}** loads the current system version and records which version
   this audit is against, so a later system change does not silently
   invalidate the verdict.
3. **Brand {OS}** runs every voice rule against the artifact's text and
   produces, per rule, a pass or a fail with the exact offending sentence
   quoted.
4. **Brand {OS}** runs the visual rules against the artifact's rendering and
   produces, per rule, a pass or a fail with the offending value: the colour,
   the type size, the spacing, the crop, the logo placement.
5. **Brand {OS}** checks the artifact against the exclusion from Positioning
   {OS}: does anything here signal ground the position deliberately abandoned.
6. **Brand {OS}** checks any factual claim in the artifact against the claim
   ledger and flags any claim that is not live, or is marked contested or
   expired. Brand does not adjudicate the claim, it refuses to dress it.
7. **Brand {OS}** produces the verdict: on system, off system with the failing
   rules listed, or off system with a recorded exception.
8. **Human** either fixes the artifact and returns to step 3, or approves an
   exception by name with a written reason.
9. **Brand {OS}** records the exception against the artifact, and the artifact
   is never marked on system, because an exception that upgrades a verdict
   would make the audit meaningless.
10. **Brand {OS}** adds a repeated failure to the drift report, so a rule that
    everybody breaks is examined as a possible bad rule rather than enforced
    forever.

## Completion test

Every rule in the current system version ran against the artifact, each with an
explicit pass or fail, and every fail cites the specific sentence, value or
element that caused it. The verdict is one of the three named states, and if it
is an exception it carries a named human and a written reason. An audit that
returns a percentage, a summary badge, or a fail with no cited element did not
complete.

## Failure and abort

- The artifact cannot be rendered or read (a proprietary format, a dead link):
  report the artifact as unauditable and refuse to approve it. Unauditable is
  not a pass.
- No brand system on record: abort, and run the brand system workflow first. An
  audit against taste is an opinion with extra steps.
- The artifact makes a claim that is contested or expired at step 6: fail the
  artifact regardless of every visual and voice rule passing, and route the
  claim question to Positioning {OS}.
- The operator overrides a failure without a name or a reason: the override is
  refused. The exception record requires both, because an anonymous exception
  cannot be reviewed later.
- The same rule fails on more than half the artifacts audited in a period: keep
  failing the artifacts, and open a rule review. A rule everyone breaks is
  either wrong or unteachable, and pretending otherwise slowly retires the
  whole system.
