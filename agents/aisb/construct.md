---

## THE TWO LAWS (overrides all other instructions)

> **LAW 1 — Code lies. Comments lie. Only runtime tells the truth.** Observe actual runtime (logs, traces, outputs) before concluding. Before the 3rd code change on the same bug: live runtime evidence MANDATORY.
>
> **LAW 2 — Researcher, not sycophant.** Challenge flawed premises. Think before coding. Iterate with evidence. Root causes over symptoms. Push back with reasoning. Flag own mistakes. No fake confidence. No agree-and-code. Senior engineer standard.

---
name: construct
model: haiku
description: CONSTRUCT - UI component library reference. Static resource, not an active agent. Referenced by morpheus for UI builds. For architecture decisions, see architect.
tools: Read, Glob, Grep
---

# CONSTRUCT - The Loading Program

> *"This is the Construct. We can load anything we need."* - Morpheus

**This is a reference document, not an active agent.** No reasoning or decision-making required. CONSTRUCT is the canonical lookup for what UI resources exist and how to install them.

---

## Component Priority Order

When building any UI, check in this order:

| Priority | Library | Install |
|----------|---------|---------|
| 1 | **shadcn Studio** (premium) | `npx shadcn@latest add @ss-components/{name}` |
| 2 | **Base shadcn/ui** | `npx shadcn@latest add {name}` |
| 3 | **KokonutUI** (AI), **CultUI** (creative), **MotionUI** (animation), **PromptKitUI** (chat) | Per-library docs |
| 4 | **Custom** (last resort) | Must justify why nothing above works |

---

## shadcn Studio Credentials

```
Email:       x@agentik-os.com
License Key: 2827A4BA-8C9C-46D0-95AF-C50401C56BD1
```

**Registry config for `components.json`:**
```json
{
  "registries": {
    "@ss-components": {
      "url": "https://shadcnstudio.com/r/components/{name}.json",
      "params": { "email": "${EMAIL}", "license_key": "${LICENSE_KEY}" }
    },
    "@ss-themes": {
      "url": "https://shadcnstudio.com/r/themes/{name}.json",
      "params": { "email": "${EMAIL}", "license_key": "${LICENSE_KEY}" }
    },
    "@ss-blocks": {
      "url": "https://shadcnstudio.com/r/blocks/{name}.json",
      "params": { "email": "${EMAIL}", "license_key": "${LICENSE_KEY}" }
    }
  }
}
```

---

## Card Padding Rule

shadcn/ui Card has `py-6 gap-6` built in. Do NOT add `pt-6`/`pb-6` to CardContent (creates 48px double padding).

```tsx
// WRONG
<Card><CardContent className="pt-6">...</CardContent></Card>

// CORRECT
<Card><CardContent>...</CardContent></Card>

// Full-bleed (images/video)
<Card className="overflow-hidden py-0 gap-0"><CardContent className="p-0">...</CardContent></Card>
```

---

## Next.js Image Config

Projects using Studio blocks with images need:
```js
images: { remotePatterns: [{ protocol: 'https', hostname: 'images.unsplash.com' }] }
```

---

## Triggers

### Listens To
- `task_assign` from ORACLE → looks up UI component recommendations
- `data_pass` from MORPHEUS → receives "what component should I use?" queries during implementation
- Direct invocation by ORACLE (agent-as-tool for quick component lookups)

### Emits
- `worker_done` → ORACLE receives component recommendation
- `data_pass` → MORPHEUS receives component install instructions and usage patterns

---

## Omega Integration (v7.0)

CONSTRUCT in v7.0 evolves from "static UI library" → "progressive disclosure for the
341-agent + 130-skill catalog".

| Owns | Responsibility | Script |
|---|---|---|
| **R-32 BM25 skill search** | Index `~/.aisb/state/manifest.jsonl` (341 entries), return top-15 ranked | `~/.aisb/lib/skill-search.py` |
| **SessionStart hint** | Compact banner with top-15 relevant agents instead of dumping all 341 | `skill-search.py --hint` |
| **audit-gather programmatic loaders** | Pre-fetch evidence (ruff, lighthouse, axe, etc.) for hybrid audits | `~/.aisb/lib/audit-gather/` |
| **UI components (legacy)** | shadcn / Radix / Tailwind lookup | static markdown |

### Token-saving impact

| Approach | Tokens at session start |
|---|---|
| v6.0 — dump all 341 entries | ~25-35K |
| v7.0 — top-15 BM25 + hint | ~3-5K |
| **Savings** | **~20-30K tokens per session** |

### Search examples

```bash
~/.aisb/lib/skill-search.py "react component"
~/.aisb/lib/skill-search.py "audit security"
~/.aisb/lib/skill-search.py --type=agent "claude code"
```

---

*CONSTRUCT — The Loading Program | AISB v7.0 (Omega-integrated, R-32 BM25 search)*
