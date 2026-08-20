# OS Builder {OS}: Setup

The Runtime asks for the minimum context needed to be useful now, not
everything this OS could ever use. Four answers are enough to start a build;
the rest can be deepened later.

## Required

| Input | Why it is required | Default if you say nothing |
|---|---|---|
| **Suite root** | the absolute path to the directory holding `OS/`, so `_registry.json`, `_tools/scaffold.py` and `_tools/verify.py` are reachable. Without it there is no registry to check for duplicates and no grader to pass | the OmegaOS checkout this OS was installed from |
| **Write scope** | the one slug directory this build may write into. OS Builder writes inside that slug and nowhere else | none: the build stops until a slug is agreed |
| **Release approver** | the named human who approves a slug registration and a release. `OS.md` section 9 lists what only they may decide | the operator running the session |
| **Default security sensitivity** | public, internal, confidential, sensitive or regulated. It sets the controls a generated OS must carry and decides whether a domain review is needed before the build starts | internal |

`python3` with the standard library is the only runtime requirement. Both tools
are stdlib only: no virtualenv, no install step, no network.

## Optional

| Input | What it improves |
|---|---|
| **Source material** | documents, an existing prompt, a legacy pack, a transcript, a workflow that already works on paper. Given at intake, it becomes the source base instead of a research stage from zero |
| **Target group** | which of the 9 suite groups the new unit belongs to. Supplying it early makes the duplicate check and the handoff edges sharper |
| **Build mode** | full build (every stage) or fast build (low-risk capabilities only). Fast never waives evidence, security or the quality gate |
| **Research depth** | how far to go before the source base is considered closed, and what counts as a defensible source in this domain |
| **Target environments** | which of Claude, Codex, ChatGPT and Gemini the generated OS must support, which decides how much the four `ADAPTERS/` files have to carry |
| **Established operator context** | anything already recorded in Context & Memory {OS}, so a build does not re-ask what was answered elsewhere |

## Environment differences that matter

OS Builder writes many files and runs two graders. That is not equally possible
everywhere, and the difference is declared in `manifest.json` under `targets`:

- **Claude and Codex:** full build. Files are written directly and
  `scaffold.py` and `verify.py` are run in place.
- **ChatGPT and Gemini:** design and author only. The content is produced in
  the session, and the operator runs `scaffold.py` and `verify.py` and pastes
  the output back. Nothing is claimed as verified until that output is seen.

An unsupported capability is reported, never silently worked around.

## Configure

```bash
agentik configure os-builder-os
```

## Verify

```bash
agentik doctor os-builder-os
python3 OS/_tools/verify.py os-builder-os
```

`doctor` reports which required inputs are present, which adapters support this
OS in your current environment, and what falls back. `verify.py` grades this
unit against the same 23 file contract it holds every unit it builds to, which
is the only honest way for a factory to prove it works.
