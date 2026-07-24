---
name: duo
description: "Binome Claude (stratège) ⇄ Codex/Sol (coder). Claude planifie, Codex challenge la stratégie puis implémente, Claude relit le diff et rend un verdict — avec bascule automatique sur Claude si le quota Codex est épuisé. Use when the user types /duo, /omg-duo, or says: build/implement/refactor/fix with the binome, code this with Codex, delegate to Codex/Sol, pair Claude and Codex, or FR 'code ça avec Codex', 'délègue à Sol', 'binome Claude Codex', 'fais coder Codex et relis'. NOT for pure questions, reads, or one-liners (Claude handles those directly — no need to burn the Codex quota)."
---

# /duo — le binome Claude ⇄ Codex

Claude reste **stratège et arbitre**. Codex (modèle `gpt-5.6-sol`) est le **coder**.
Le muscle déterministe est le binaire `omega-duo` : Claude ne lance **jamais** `codex`
en direct. Le bridge vérifie l'authentification réelle, la lecture du worktree, le
sandbox, les mutations interdites et la détection de quota avant de rapporter le résultat.

## Quand l'utiliser

- L'opérateur tape `/duo <tâche>` ou dit explicitement « code ça avec Codex / Sol / le binome ».
- La tâche **écrit du code** : implémenter, modifier, refactorer, corriger un bug.

**Ne PAS** déclencher pour une question, une lecture, un one-liner : Claude répond seul.
Router du trivial vers Codex gaspille son quota (routage explicite, R-BUDGET).

## Le bridge

Résolu dans l'ordre : `omega-duo` sur le PATH, sinon
`<OMEGA_DIR>/skills/duo/bin/omega-duo`, sinon
`~/Station/SideBusiness/OmegaOS/tools/duo/bin/omega-duo`. Le répertoire canonique est
`$OMEGA_DIR` s'il est défini, sinon `~/OmegaOS/System` s'il existe, sinon `~/.omega`.

```
omega-duo run --task <file.md> --cwd <projet> --mode <plan|code|review> [--agent codex|claude]
  → une ligne JSON : { agent, ok, output, fell_back, reason, exit_code, log,
                       sandbox_degraded,
                       capabilities: { shell_exec, worktree_read } }
```

- `mode plan` / `mode review` → l'essai zéro utilise toujours le mode natif
  en lecture seule (`Codex --sandbox read-only` ou `Claude --permission-mode plan`)
  et prouve la lecture du dépôt.
- `mode code` → Codex en **full-auto** (il édite les fichiers).
- `fell_back: true` → le quota Codex était épuisé, **Claude** a fait le travail. Tu DOIS
  le dire à l'opérateur. Le résultat est `single_model`, jamais une validation indépendante.
- `sandbox_degraded: true` → un essai natif exécuté n'a pas pu prouver la lecture du
  dépôt. Alors seulement, le bridge a utilisé un accès externe gardé et vérifié le
  worktree avant/après. Tu DOIS le signaler.
- `capabilities.shell_exec` et `capabilities.worktree_read` doivent être `true`. Sinon,
  ou si `ok` vaut `false`, **STOP** : ne lis pas `output` comme une critique ou revue valide.
- `reason: "codex-unauthenticated"` → session Codex inutilisable. **STOP**, demande une
  réparation de l'authentification. Ce cas ne bascule jamais sur Claude et ne consomme pas
  le drapeau de quota.

L'appelant fournit un **fichier**, puis le bridge transmet son contenu aux agents sur
stdin, jamais en argv. Écris les tâches dans un dossier scratch, p.ex.
`agentic/duo/<slug>/`.

L'état d'un enfant en cours reste observation-only. Pour annuler un login Codex,
utilise `omega codex-login-abort --pid <n>` avec le PID retourné par
`codex-login`. Le chemin vérifie l'identité et les argv exacts, attend le sentinel
de sortie, puis restaure seulement une topologie réellement perturbée. Si la phase
Telegram possède le bouton Cancel, elle doit être repointée vers cette commande.

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
Vérifie d'abord `ok`, `sandbox_degraded` et les deux `capabilities`. Lis ensuite la
critique dans `output`. Une critique sans lecture prouvée du worktree n'est pas une critique.

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
le réarme si besoin. Cette bascule ne satisfait pas un contrôle obligatoire à deux modèles.

Une erreur `codex-unauthenticated` est différente : pas de fallback, pas de marquage quota,
pas de résultat vert. Répare l'authentification Codex avant de reprendre la boucle. Le
probe live `omega doctor --deep` est une action explicite qui peut consommer du quota.

## Périmètre

- **Gemini** n'est pas câblé ici (redondant avec Codex, non utilisé). Le bridge a une
  table d'agents : l'ajouter plus tard = une entrée, pas une refonte.
- Le mode natif en lecture seule est toujours l'essai zéro pour plan/review. Un `bwrap`
  absent ou impossible à lancer est une dépendance inconnue, jamais une preuve que le
  sandbox Codex est cassé, et ne déclenche pas le mode dégradé. Claude essaie de même
  `--permission-mode plan` avant tout accès gardé.
- Après l'échec démontré du probe natif, le bridge annonce `sandbox_degraded: true`,
  applique une garde worktree et échoue sur toute mutation observée. Cette garde couvre
  le worktree, l'index, les fichiers ignorés ou non suivis, les dossiers vides et les
  métadonnées Git complètes, y compris le git-dir. Le mode dégradé échoue fermé si
  aucun watcher live ne peut être établi. Elle ne garantit ni l'isolation réseau, ni
  les écritures hors dépôt.
- Un échec réseau/provider, une réponse 5xx, une réponse malformée ou un marker
  absent ne constitue pas une preuve de refus sandbox et ne déclenche jamais le
  bypass. Le mode natif compare l'état observable par Git, donc un artefact ignoré
  modifié par un writer externe ne rend pas une revue native invalide.

## Rappels

- Réponds en français (R-STYLE) ; le code, les commits, les identifiants en anglais.
- Un livrable (lien/fichier) part aussi sur Telegram (R-TGDELIVER).
- Termine par : `--- **Résumé:** …` (qui a codé quoi, verdict, quota).
