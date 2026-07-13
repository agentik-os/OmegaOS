---
name: duo
description: "Binome Claude (stratège) ⇄ Codex/Sol (coder). Claude planifie, Codex challenge la stratégie puis implémente, Claude relit le diff et rend un verdict — avec bascule automatique sur Claude si le quota Codex est épuisé. Use when the user types /duo, /omg-duo, or says: build/implement/refactor/fix with the binome, code this with Codex, delegate to Codex/Sol, pair Claude and Codex, or FR 'code ça avec Codex', 'délègue à Sol', 'binome Claude Codex', 'fais coder Codex et relis'. NOT for pure questions, reads, or one-liners (Claude handles those directly — no need to burn the Codex quota)."
---

# /duo — le binome Claude ⇄ Codex

Claude reste **stratège et arbitre**. Codex (modèle `gpt-5.6-sol`) est le **coder**.
Le muscle déterministe est le binaire `omega-duo` : Claude ne lance **jamais** `codex`
en direct, il passe toujours par le bridge, qui gère la détection de quota et la
bascule automatique sur Claude.

## Quand l'utiliser

- L'opérateur tape `/duo <tâche>` ou dit explicitement « code ça avec Codex / Sol / le binome ».
- La tâche **écrit du code** : implémenter, modifier, refactorer, corriger un bug.

**Ne PAS** déclencher pour une question, une lecture, un one-liner : Claude répond seul.
Router du trivial vers Codex gaspille son quota (routage explicite, R-BUDGET).

## Le bridge

Résolu dans l'ordre : `omega-duo` sur le PATH, sinon `~/.omega/skills/duo/bin/omega-duo`,
sinon `~/Station/SideBusiness/OmegaOS/tools/duo/bin/omega-duo`.

```
omega-duo run --task <file.md> --cwd <projet> --mode <plan|code|review> [--agent codex|claude]
  → une ligne JSON : { agent, ok, output, fell_back, reason, exit_code, log }
```

- `mode plan` / `mode review` → Codex en **lecture seule** (il critique, il n'édite pas).
- `mode code` → Codex en **full-auto** (il édite les fichiers).
- `fell_back: true` → le quota Codex était épuisé, **Claude** a fait le travail. Tu DOIS
  le dire à l'opérateur dans ta réponse (bascule visible, pas silencieuse).

Les tâches passent par **fichiers** (jamais en argv : un gros prompt en argument est
tronqué). Écris-les dans un dossier scratch, p.ex. `agentic/duo/<slug>/`.

## La boucle (obligatoire, dans l'ordre)

Crée une TODO par étape.

### 1. Plan (Claude)
Écris `plan.md` : objectif, critères de succès **vérifiables** (commande de test/build),
fichiers touchés, approche. C'est ta stratégie — sois précis.

### 2. Critique (Codex, lecture seule)
Écris `critique-task.md` : « Voici un plan d'implémentation dans ce repo. Challenge-le :
angles morts, fichiers qui bloquent, meilleure approche. NE code rien, réponds en texte. »
puis colle le plan. Lance :
```
omega-duo run --task agentic/duo/<slug>/critique-task.md --cwd <projet> --mode plan
```
Lis la critique dans `output`. Codex connaît le code : prends ses objections au sérieux.

### 3. Plan v2 (Claude)
Intègre la critique (ou justifie en 1 ligne pourquoi tu l'écartes — L2, pas de
suivisme aveugle). Réécris `plan.md`.

### 4. Implémentation (Codex, full-auto)
Écris `code-task.md` : le plan v2 + « Implémente exactement ça dans ce repo. Respecte le
style existant. Ne touche qu'aux fichiers listés. » Lance :
```
omega-duo run --task agentic/duo/<slug>/code-task.md --cwd <projet> --mode code
```

### 5. Revue (Claude) — le verdict est à TOI
Lis le **diff réel** (`git -C <projet> diff`), pas le récit de Codex (R-VERIFY : le
« done » d'un délégué est un input, jamais le verdict). Exécute la commande de succès du
plan (L1 : le runtime est la seule vérité). Rends **PASS** ou **FIX** :
- **PASS** → résume ce qui a changé, cite le diff/le test vert.
- **FIX** → écris `fix-task.md` (problème précis + fichier:ligne) et relance l'étape 4.

### Plafond (R-LOOP)
**Maximum 3 tours de FIX sur le même échec.** Au 3e échec, **STOP** : n'insiste pas,
escalade à l'opérateur avec l'état exact (ce qui marche, ce qui bloque, le dernier diff,
le log du bridge). Reboucler une 4e fois est du thrash, pas du progrès.

## Fallback quota (déjà géré par le bridge)

Tu n'as **rien** à parser toi-même. Si `fell_back: true` revient, dis-le simplement :
> ⚠️ Quota Codex épuisé — j'ai codé moi-même (Claude) le reste.

Codex est alors marqué indisponible pour la session (`omega-duo status` le montre) et les
étapes suivantes basculent direct sur Claude, sans retaper la limite. `omega-duo reset`
le réarme si besoin.

## Périmètre

- **Gemini** n'est pas câblé ici (redondant avec Codex, non utilisé). Le bridge a une
  table d'agents : l'ajouter plus tard = une entrée, pas une refonte.
- Codex tourne en sandbox (`read-only` pour plan/review, full-auto pour code) : il
  n'exfiltre pas et n'édite pas hors mode `code`.

## Rappels

- Réponds en français (R-STYLE) ; le code, les commits, les identifiants en anglais.
- Un livrable (lien/fichier) part aussi sur Telegram (R-TGDELIVER).
- Termine par : `--- **Résumé:** …` (qui a codé quoi, verdict, quota).
