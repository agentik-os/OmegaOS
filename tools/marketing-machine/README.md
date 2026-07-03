# omega-marketing-machine

Per-project **marketing machine** scaffolder. Gives every OmegaOS/Partner project an
**identical `marketing/` folder** (the parent folder of all GTM artifacts), then the
marketing skills fill it. Closes the loop from strategy → copy → visual identity (DA +
Higgsfield) → autonomous publishing via **zernio** (`tools/zernio/`).

Governing rules: **R-MARKETING** (GTM suite + dependency order), **R-VISUAL-ID**
(Higgsfield pair), **R-SKILLPUB** (this is the SSOT), **R-CITE**, **L0/L1**.

## The canonical `marketing/` tree

```
marketing/
  README.md                       # status board + pipeline for this project
  00-context/                     # product-marketing-context, market-research, competitors, personas
  01-strategy/                    # gtm, content-strategy, launch-strategy
  02-copy/                        # copywriting, ad-creative, social-content, cold-email
  03-visual-identity/             # DA.md (direction artistique)
    higgsfield/                   # system-prompt, preprompt, avatar-soul, shotlist
  04-publishing/                  # zernio.md (platforms + connect cmds) + calendar.json (queue)
```

`00→04` mirrors the marketing-machine dependency order (R-MARKETING). The two layers
beyond the classic GTM suite are **03-visual-identity** (DA → Higgsfield system/preprompt/
avatar so generation is on-brand and identity-faithful) and **04-publishing** (the zernio
queue that turns validated content into scheduled posts).

## Scaffold

```bash
tools/marketing-machine/scaffold.sh                       # all projects in projects.tsv
tools/marketing-machine/scaffold.sh Partners/foo Foo foo  # one ad-hoc project (dir name slug)
```

Idempotent + **non-destructive**: only writes a file if it does not already exist, so it
never clobbers content the marketing skills (or you) have written. Add a project by
appending one TAB-separated line to `projects.tsv`.

## Fill the machine

Per project, run the marketing skills in dependency order (they read `00-context/` first),
or fan out one strong agent per project that embodies the method. Each file carries
frontmatter naming the skill that produces it. French products (DentistryGPT, Gluten-Libre,
Verba, …) → outward copy in French (R-STYLE).

## Publish (zernio — the autonomous half)

One zernio profile per project (`profiles slug` = the project slug). Accounts connect via
OAuth (operator step — `omega-zernio connect <slug> <platform>`), never auto-connected.
After content is validated:

```bash
omega-zernio post <slug> --text "…" --platforms instagram,tiktok --media ./a.png --dry-run
omega-zernio post <slug> --text "…" --platforms instagram,tiktok --schedule 2026-07-01T09:00:00Z
```

The per-project `04-publishing/calendar.json` is the queue feeding these.

## Boundaries (external opt-in, R-VISUAL-ID / R-ENV)

- **Higgsfield** image/video generation = curl|sh CLI + paid web plan; API credits ≠ web
  subscription. The setup phase produces prompts/DA/avatar **specs only** — no live
  generation, not runtime-verifiable without operator credentials.
- **zernio** publishing requires the operator's OAuth-connected accounts. Key lives in
  `~/.omega/secrets/integrations.env` (`ZERNIO_API_KEY`), never in the repo.

## Growth kit

`growth/GROWTH-KIT.md` distills the growth mechanics (X teardown of @tibo_maker + @antinertia,
and the IG/YouTube/TikTok/LinkedIn multi-platform analysis) into a runnable, project-agnostic
playbook: the per-platform hook engine, the carousel system (IG + LinkedIn), the comment-to-DM
demo-and-capture loop, the build-in-public engine, named series formats, and per-platform
protocols. Every content skill and daily engine reads it before producing. No em-dash (R-NODASH).
