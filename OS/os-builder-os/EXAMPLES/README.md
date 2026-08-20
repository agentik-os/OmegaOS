# OS Builder {OS}: Examples

Worked examples showing this OS on a real situation, from opening move to
finished artifact. Each one runs the actual pipeline, including the phases that
produce nothing and the gates that say no.

| Example | Shows | Ends at |
|---|---|---|
| [ai-maturity-os.md](ai-maturity-os.md) | a full build, all fifteen phases, from a one-sentence request to a released unit | `RELEASE` |
| [refused-a-request.md](refused-a-request.md) | the viability tree declining to build an OS, and delivering the right smaller thing instead | `USE A LIGHTER ARTIFACT` |

## How to read these

The two examples are chosen as a pair because they are the two outcomes that
actually occur. Most requests that arrive at OS Builder should not become an
OS, and an example set that only shows successful builds teaches the wrong
reflex. The refusal is not the failure case. It is the common case.

**On the command output shown.** The transcripts reproduce the shape these
tools print, so you can recognise a real run and spot a fabricated one. They
are illustrations, not captured logs: run the commands against your own build
for the real numbers, and never paste an example's numbers into a report.

**On the slugs used.** Neither `ai-maturity-os` nor anything in the refusal
example is a registered unit of the suite. They are walk-throughs. The
registration steps are shown in full precisely because that is the step most
often done wrong, but running them for these examples would add units nobody
asked for.
