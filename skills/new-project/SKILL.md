---
name: new-project
description: >
  OmegaOS end-to-end new-project pipeline — guiding (client/works) → stack pick → AUTO-PROVISION
  external services (Vercel/Convex/GitHub/Clerk/Stripe) from ~/.omega/provisioning →
  scaffold the full stack → wire every key → register the project → vision/PRD →
  /omg-planner (typed DAG plan) → omega plan-run (the engine executes it with can't-skip
  Gate + Guardian verify). Launchable from the TUI [N] New Project menu or by typing
  /omg-new-project (alias /omega-new-project).
argument-hint: "[stack|repo-url] [category] [name]   e.g. nextstack customer acme  OR  https://github.com/acme/app customer   (all optional → interactive)"
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "AskUserQuestion", "Task", "ToolSearch"]
domain: orchestration
read_only: false
triggers: ["omg-new-project", "omega-new-project", "new project", "nouveau projet", "scaffold project", "setup new repo"]
---

# /omega-new-project — Provision-and-scaffold pipeline

Turns "new project" into a fully wired project: external services created and
connected, the stack scaffolded, then vision → PRD → plan. Idempotent and
resumable. Honest about what can and cannot be auto-provisioned.

`$ARGUMENTS` = `[stack] [category] [name]` (positional, all optional). When
invoked from the TUI menu these are pre-filled; when typed, any missing value is
asked interactively. Never block on a value the menu already supplied.

---

## STACKS (registry)

| id | Stack | Components |
|----|-------|-----------|
| `nextstack` *(default)* | **Next.js 16 + Convex + Clerk + Stripe + shadcn-chatbot-kit** | App Router, Convex realtime, Clerk auth, Stripe billing, the **full** shadcn chatbot kit (every chat component), oklch brand system. Optional: **React Flow**. The smart-default for SaaS/web. |
| `custom` | **Pick-your-own** | Let the operator compose the stack — same question flow as the old VPS wizard. |

> `nextstack` is the **proposed default** (identical to the old-VPS smart default).
> Choosing **`custom`** opens a short stack chooser (AskUserQuestion), one question
> each, default pre-selected:
> - **Project type** — SaaS web · Landing/marketing · Mobile (Expo) · Desktop (Tauri) · API backend.
> - **Backend/DB** — Convex *(default)* · Supabase · Custom API (Hono) · none.
> - **Auth** — Clerk *(default)* · Better Auth · Auth.js · none.
> - **Payments** — Stripe *(default)* · LemonSqueezy · none.
> - **UI** — shadcn/ui + oklch *(default)* · NativeWind (mobile) · Tamagui.
>
> The chosen components drive provisioning (Phase 2) + the scaffold (Phase 3):
> provision only the picked services, scaffold only the picked libs. Add a new
> fixed stack as another row + a scaffold block; `custom` needs no new row.

---

## PHASE 0 — Resolve inputs (no redundant questions)

**Import mode (existing GitHub repo).** If an argument is a git URL (`git@…`,
`https://….git`, or `https://github.com/<owner>/<repo>`), this is an IMPORT, not
a fresh scaffold:
1. Resolve `CATEGORY` (ask if missing: `customer` / `side-business` / `tools`)
   and `<projects_dir>` exactly as below — so the repo lands in the right place.
2. `git clone <url> "<projects_dir>/<category-path>/<repo-name>"` (repo-name from
   the URL unless `NAME` overrides it). For a `customer`, also pick/create its
   credential group so deploys use that customer's own accounts.
3. Set the category git identity (Phase 4 step 3), detect the stack from the
   cloned tree (package.json / Cargo.toml / pyproject.toml / go.mod), and
   register the project in `~/.omega/projects.json` so it shows in the TUI.
4. Run provisioning (Phase 2) ONLY for services the repo needs but is missing
   (absent `.env` keys); never scaffold over existing source.
Then stop — import is done. The scaffold phases below are for fresh projects.

1. Parse `$ARGUMENTS`: `STACK`, `CATEGORY`, `NAME` (positional).
2. **Stack**: if missing or unknown → AskUserQuestion listing the table above with
   **`nextstack` pre-selected as the default**. If the user picks **`custom`**, run
   the stack chooser (the bulleted questions above) and record the picked
   components — they drive which services get provisioned (Phase 2) and which libs
   get scaffolded (Phase 3). Any explicit stack arg is honored without asking.
3. **Category** (the guiding branch) — if missing, AskUserQuestion:
   - `customer` → client work → `<projects_dir>/customers/<name>` (falls back to
     an existing `clients/` if that's the user's layout)
   - `side-business` → your own products → `<projects_dir>/side-business/<name>`
     (falls back to an existing `work/`)
   - `tools` → internal tooling / libraries → `<projects_dir>/tools/<name>`
   Resolve `<projects_dir>` from `~/.omega/config.toml` (key `projects_dir`),
   never a hardcoded `~/VibeCoding`. Life → `<projects_dir>/1-life/`. The category
   also picks the **git identity** later (see Phase 4).
4. **Name**: if missing → ask. Validate: lowercase, `[a-z0-9-]`, not already a
   dir under the resolved category path, not already in `~/.omega/projects.json`.
5. **React Flow option**: AskUserQuestion "Inclure React Flow (systèmes de
   nœuds/diagrammes) ?" yes/no → `WANT_REACTFLOW`.
6. Echo the resolved plan in 3 lines and proceed (no confirmation gate when the
   menu supplied everything — Law L3).

Set `PROJECT_DIR="<projects_dir>/<category-path>/<name>"` using the resolved
`projects_dir` and the category→path mapping above (customers / side-business /
tools, falling back to an existing `clients/` or `work/`).

---

## PHASE 1 — Load provisioning credentials

```bash
PROV="$HOME/.omega/provisioning/services.env"
if [[ -f "$PROV" ]]; then set -a; source "$PROV"; set +a; else
  echo "⚠ No ~/.omega/provisioning/services.env — provisioning will run in MANUAL/PAUSE mode."
fi
```

> **Guided token entry:** the user fills these tokens without editing any file —
> in the OmegaOS TUI, **Monitor tab → "Set up project provisioning keys" (P)** runs
> a step-by-step wizard (Vercel → Convex → GitHub → Stripe) that writes them safely
> to `services.env` (chmod 600, blanks skipped). If tokens are missing, point the
> user there before falling back to PAUSE mode.

Build a capability map (each ON only if its token is present):

- `CAN_VERCEL`  = `[ -n "$VERCEL_TOKEN" ]`
- `CAN_CONVEX`  = `[ -n "$CONVEX_TEAM_TOKEN" ]`
- `CAN_GITHUB`  = `[ -n "$GITHUB_TOKEN" ]` OR `gh auth status` succeeds
- `CAN_STRIPE`  = `[ -n "$STRIPE_SECRET_KEY" ]`
- Clerk uses `CLERK_PROVISION_MODE` (pool|pause), never a create-API.

Print the capability map so the user sees exactly what will be automated vs
paused. **A blank token is a PAUSE, never a silent skip** (Law L4 / L5).

---

## PHASE 2 — Provision external services

Run each provisioner; record results into `$PROJECT_DIR/.omega-provision.json`
(create the dir first). Each is **idempotent** — re-running detects existing
resources and reuses them. On any hard failure, write the partial state and
continue with the rest (never abort the whole pipeline over one service); list
blockers at the end (Law L4).

### 2a. GitHub repo
- `CAN_GITHUB`: `gh repo create <owner>/<name> --private --source="$PROJECT_DIR" --remote=origin`
  (owner = `$GITHUB_OWNER` or `gh api user -q .login`). If `$GITHUB_TOKEN` set,
  export `GH_TOKEN="$GITHUB_TOKEN"` for the call.
- Else PAUSE: tell the user to `gh auth login`, then re-run (resumable).

### 2b. Vercel project (FULL auto)
- `CAN_VERCEL`: create the project via API and capture its id.
  ```bash
  TEAM_Q=${VERCEL_TEAM_ID:+?teamId=$VERCEL_TEAM_ID}
  curl -s -X POST "https://api.vercel.com/v11/projects$TEAM_Q" \
    -H "Authorization: Bearer $VERCEL_TOKEN" -H "Content-Type: application/json" \
    -d "{\"name\":\"<name>\",\"framework\":\"nextjs\"}"
  ```
  Link the local dir: `vercel link --yes --project <name> --token "$VERCEL_TOKEN" ${VERCEL_TEAM_ID:+--scope $VERCEL_TEAM_ID}` in `$PROJECT_DIR`.
  Env vars are pushed in Phase 3 once their values exist.
- Else PAUSE.

### 2c. Convex deployment (FULL auto)
- `CAN_CONVEX`: provision a dev deployment non-interactively. Run inside
  `$PROJECT_DIR` after `npm i convex`:
  ```bash
  CONVEX_AGENT_MODE=anonymous npx convex dev --once --configure=new \
    --team "$CONVEX_TEAM_SLUG" --project "<name>" 2>&1 || true
  # capture CONVEX_DEPLOYMENT + NEXT_PUBLIC_CONVEX_URL from .env.local convex wrote
  ```
  Use `$CONVEX_TEAM_TOKEN` via `CONVEX_DEPLOY_KEY` env if the non-interactive
  team flag is unavailable in the installed CLI version — verify the CLI's flags
  first (`npx convex dev --help`), never assume (Law L1).
- Else PAUSE: `npx convex dev` interactive.

### 2d. Clerk (NO create-API → pool or pause)
- `CLERK_PROVISION_MODE=pool`: pop the first unused line of
  `~/.omega/provisioning/clerk-pool.env` (`pk|sk|label`), mark it `# USED <name>`,
  capture `CLERK_PUBLISHABLE` + `CLERK_SECRET`. If the pool is empty → fall
  through to pause.
- `pause` (default): open `https://dashboard.clerk.com`, instruct the user to
  create an app + copy the **Publishable** and **Secret** keys, then collect them
  with AskUserQuestion (free-text). Persist immediately so a re-run resumes.
- State the reason plainly: "Clerk has no public app-creation API — this step is
  pool/pause by necessity, not by shortcut."

### 2e. Stripe
- `STRIPE_MODE=single` + `CAN_STRIPE`: reuse the master account. Create a
  restricted key + a webhook endpoint pointing at the (future) Vercel URL +
  starter product/price via the API:
  ```bash
  curl -s https://api.stripe.com/v1/products -u "$STRIPE_SECRET_KEY:" -d name="<name>"
  curl -s https://api.stripe.com/v1/webhook_endpoints -u "$STRIPE_SECRET_KEY:" \
    -d url="https://<name>.vercel.app/api/stripe/webhook" -d "enabled_events[]=checkout.session.completed"
  ```
  Capture the webhook signing secret (`whsec_…`) and a restricted key for the app.
- `STRIPE_MODE=connect` + `CAN_STRIPE`: `POST /v1/accounts` (type=standard),
  capture the connected `acct_…`, generate an onboarding link
  (`/v1/account_links`), and **tell the user KYC onboarding is required before
  live charges** — surface the link, don't pretend it's done.
- Else PAUSE.

---

## PHASE 3.0 — Claude Design import (optional — ask first)

Before scaffolding the UI from scratch, **ask the operator** (AskUserQuestion):
*"Did you design this on **Claude Design (claude.ai/design)**? If so I'll import your
work instead of generating UI from zero."*
- **No** → continue to the normal scaffold below.
- **Yes** → collect the work, two ways:
  - **Zip** — operator uploads/points to the export `.zip`; unzip into a temp dir.
    From Telegram, accept the uploaded document; from the TUI/CLI, ask for the path.
  - **Command** — give them the retrieval command to paste:
    `npx @anthropic-ai/claude-design pull <share-id-or-url> -o ./_design` (or, if they
    have the share URL, `curl -L <export-url> -o design.zip && unzip design.zip -d ./_design`).
- **MATCH THE FRONTEND LANGUAGE before importing** — ask/detect and only import like-for-like:
  - Claude Design output is **HTML/CSS** → import into an **HTML/static** or `doc`/landing
    stack (don't paste raw HTML into a `.tsx` tree).
  - Output is **React/Next (TypeScript)** → import components into the Next.js `src/`
    (`.tsx`), reconciling Tailwind/oklch tokens with the project's `globals.css`.
  - Mismatch (e.g. HTML design but Next.js project) → **convert** deliberately (port markup
    to components, lift styles into the token system) — never drop raw mismatched files in.
- Record the design source + language in `BRAND.md` / `CLAUDE.md` so later steps reuse it,
  then let `/omg-brand-identity` (PHASE 5) build on the imported tokens rather than re-inventing.

---

## PHASE 3 — Scaffold the stack (`nextstack`)

Fetch latest versions first (don't hardcode): if Context7 MCP is available,
`ToolSearch("select:mcp__context7__resolve-library-id")` then query `next`,
`convex`, `@clerk/nextjs`, `stripe`, `tailwindcss`. Otherwise use `@latest`.

```bash
cd "$(dirname "$PROJECT_DIR")"
npx create-next-app@latest "<name>" --ts --app --tailwind --eslint --src-dir --import-alias "@/*" --use-npm --yes
cd "$PROJECT_DIR"
npx shadcn@latest init -d
# Default theme kit — apply the tweakcn theme so every project starts on the same
# polished design tokens (oklch). The cmpuchuiv theme first, then the Claude theme
# layered on top (the Claude tokens win for brand consistency). Both are tweakcn
# registry items; bunx if bun is present, else npx.
bunx shadcn@latest add https://tweakcn.com/r/themes/cmpuchuiv000104jmhbti6yb4 -y 2>/dev/null \
  || npx shadcn@latest add https://tweakcn.com/r/themes/cmpuchuiv000104jmhbti6yb4 -y || true
bunx shadcn@latest add https://tweakcn.com/r/themes/claude.json -y 2>/dev/null \
  || npx shadcn@latest add https://tweakcn.com/r/themes/claude.json -y || true
# Full shadcn chatbot kit — EVERY chat component (Law L5: all of it, not a subset)
npx shadcn@latest add "https://shadcn-chatbot-kit.vercel.app/r/chat.json" -y || \
  npx shadcn@latest add chat message prompt-input markdown -y
# Design rule for this project: prefer the imported shadcn/ui components for ALL
# UI — do not hand-roll a button/input/dialog when the library ships one. Record
# this in the project's CLAUDE.md so every agent session honors it.
npm i convex @clerk/nextjs stripe @stripe/stripe-js
[ "$WANT_REACTFLOW" = yes ] && npm i @xyflow/react
```

Then create:
- `src/app/providers.tsx` — ClerkProvider + ConvexProviderWithClerk wired together.
- `src/middleware.ts` — Clerk middleware.
- `convex/` schema stub + `convex/auth.config.ts` keyed to the Clerk issuer.
- `src/app/api/stripe/webhook/route.ts` — signature-verified handler.
- A `/chat` route mounting the chatbot-kit components end-to-end.
- If `WANT_REACTFLOW`: a `/flow` route with a minimal React Flow canvas.
- **Brand system**: `src/app/globals.css` oklch tokens (full light/dark scale,
  radii, shadows, typography) + `BRAND.md`. Pull a coherent palette; this is the
  "ultra complete" brand foundation the user asked for, not 3 stray variables.

---

## PHASE 4 — Wire keys + register

1. Write `$PROJECT_DIR/.env.local` from the provisioned values:
   ```
   NEXT_PUBLIC_CONVEX_URL=…          CONVEX_DEPLOYMENT=…
   NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY=…   CLERK_SECRET_KEY=…
   STRIPE_SECRET_KEY=…(restricted)   STRIPE_WEBHOOK_SECRET=whsec_…   NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY=…
   ```
   Also write `.env.example` with the same keys blanked (committed).
2. Push the non-public vars to Vercel (`CAN_VERCEL`): `vercel env add <KEY> production` via API/CLI with `--token`.
3. **Git identity by category** (Law R-FICHE): set `git config user.email` —
   customer → that customer's email; side-business / tools / AgentikOS →
   `x@agentik-os.com`. Commit
   the scaffold. Push to origin if the repo was created.
4. **Register** in `~/.omega/projects.json` (append an entry: name, path,
   category, created date) so it shows up in the TUI Projects tab immediately.
5. **Telegram setup — propose it, don't force it (optional).** AskUserQuestion:
   *"Set up Telegram for this project?"*
   - **Topic** *(recommended)* — create its forum topic in the hub (`/sync`, or the
     bot's `createForumTopic`) so a message in that topic = a mission to its oracle,
     and publish its `/<project>` command. Default ON if a hub is configured.
   - **+ Dedicated bot** *(optional)* — link a separate Telegram bot token
     (BotFather) whitelisted to the operator (the bot's `proj:botlink` flow / the
     `tg-link` pending). Talking to it = this project's oracle, scoped to it.
   - **None** — skip; the project still works from the TUI/CLI.
   Whatever is chosen, never block the pipeline on it.
6. Verify (Law L1): `npm run build` must pass before declaring scaffold done.

---

## PHASE 5 — Chain into the product pipeline (engine-driven)

Do not re-implement vision/PRD/planning — delegate, in order, scoped to
`$PROJECT_DIR`:

1. `/omg-vision` — emotional positioning → `VISION.md`
2. **Spawn the project's DEDICATED ORACLE to present the vision** (asked for on
   every new project). As soon as `VISION.md` exists, dispatch the project oracle
   to read it and explain the product back to the operator, in their language:
   `omega dispatch "$NAME" "Read $PROJECT_DIR/VISION.md and present this product's
   VISION to me in one message — soul statement, internal compass, primary persona,
   the 3 design principles, and what we build next. Concrete and warm, no fluff."`
   This opens a persistent oracle so the operator can immediately discuss + steer
   the vision. (From the Telegram bot, `createProject` already auto-launches it.)
3. `/omg-prd` — full doc suite from the vision → `docs/PRD.md`
4. `/omg-brand-identity` — **visual identity** (Kapferer Brand Identity Prism,
   design tokens, 2-3 switchable variants, typography/color/voice/logo direction,
   AI prompts, an interactive **brand-book** Next.js sub-app) → `BRAND.md` +
   `brand-book/`. Reuses the project's oklch tokens so brand + scaffold stay
   consistent. Skippable for non-visual projects (`--skip=brand`).
5. `/omg-planner` — generate the **typed** `.planner/tracker.json` (a DAG of
   single-worker-dispatch steps; audits as a terminal `wave`). Verify it loads:
   `omega plan-status .` must print the steps with `ready N | blocked M`.
6. `omega plan-run .` — **the OmegaOS engine executes the plan (the "build")**:
   ready-set → spawn worker → Guardian re-runs each `verify_command` → advance.
   Sequencing is enforced structurally (a step can NEVER be skipped) and no step is
   "done" without its verify proof. Watch progress with `omega plan-status .`.
   If `omega` is not on PATH, fall back to `bun ~/.omega/skills/planner/fallback/plan.ts run .`.

Full pipeline order: **vision → (oracle presents it) → prd → brand-identity →
planner → build (plan-run)**. Offer to stop after `/omg-vision` so the user can
review with the oracle, or — in a dispatched (non-interactive) context — proceed
automatically through to `omega plan-run` (Law L3). The engine, not the LLM, owns
the execution loop from `plan-run` on.

---

## DONE CRITERIA (grade against this, not vibes — R-RUBRIC)

- [ ] Project dir created under the **correct category** path.
- [ ] Every service either **provisioned** (resource id captured) or **explicitly
      paused** with a recorded reason — none silently skipped.
- [ ] `.env.local` wired; `.env.example` committed; secrets only in
      `$PROJECT_DIR/.env.local` + `~/.omega`, never staged for git.
- [ ] `npm run build` passes.
- [ ] Project registered in `~/.omega/projects.json` (visible in the TUI).
- [ ] `VISION.md` started (pipeline handed off).
- [ ] A `--- Resume:` one-line French recap (R-STYLE).

Report a checklist of provisioned vs paused services + the next command.
