@@@META
{
  "template": "whitepaper",
  "theme": "agentik",
  "eyebrow": "ARCHITECTURE · OMEGA v3 · 2026",
  "title": "Omega v3 — Architecture d'un Agentic OS Programmatique",
  "subtitle": "Du harnais bash + tmux à un moteur d'orchestration typé, multi-LLM, auto-améliorant et distribuable.",
  "author": "Claude Opus 4.7 — pour Gareth Moison",
  "date": "22 mai 2026",
  "docId": "OMEGA-V3-ARCH-01",
  "brand": "AGENTIK",
  "abstract": "Omega est aujourd'hui une plateforme d'ingénierie logicielle multi-agent qui fonctionne — mais dont la fiabilité repose sur près de 200 scripts bash, des sessions Claude Code pilotées dans tmux, et des règles markdown réinterprétées par un LLM à chaque tour. Ce document propose Omega v3 : une refonte qui conserve intégralement l'intelligence du système — le modèle d'orchestration à quatre niveaux, les dix-sept audits du Quality Arsenal, les Trois Lois — et remplace le harnais par un véritable moteur programmatique, typé, écrit en TypeScript sur Bun. Omega v3 est un Agentic OS : agnostique au fournisseur LLM via une source de vérité unique et des adaptateurs ; rangé sous une arborescence stricte dans un dossier maître unique ; doté d'un sous-système multi-RAG, de huit Factories qui régénèrent le système lui-même, et d'un installeur bootstrap qui déploie l'ensemble sur un VPS ou un MacBook neuf en une seule commande. Ce rapport définit l'architecture cible, le plan de migration sans rupture, et la feuille de route.",
  "toc": [
    { "index": "01", "title": "Résumé exécutif & verdict", "page": 4 },
    { "index": "02", "title": "Anatomie de l'Omega actuel", "page": 6 },
    { "index": "03", "title": "L'Agentic OS — vue en couches", "page": 9 },
    { "index": "04", "title": "Le système de fichiers & le SST", "page": 12 },
    { "index": "05", "title": "L'abstraction provider", "page": 16 },
    { "index": "06", "title": "Le moteur d'orchestration", "page": 19 },
    { "index": "07", "title": "Le prompt engineering automatisé", "page": 24 },
    { "index": "08", "title": "Les 8 Factories", "page": 27 },
    { "index": "09", "title": "Le sous-système multi-RAG", "page": 31 },
    { "index": "10", "title": "Agentic-Tools — le dossier Applications", "page": 35 },
    { "index": "11", "title": "Exécution goal-driven & tests live", "page": 38 },
    { "index": "12", "title": "Observabilité & sûreté", "page": 41 },
    { "index": "13", "title": "Le Bootstrap installer", "page": 44 },
    { "index": "14", "title": "Plan de migration — améliorer sans casser", "page": 48 },
    { "index": "15", "title": "Décision Python/Bun & feuille de route", "page": 53 }
  ]
}
@@@SECTION
{"index":"01","eyebrow":"Verdict","title":"Résumé exécutif & verdict","lead":"Omega fonctionne. Mais sa fiabilité tient à du bash. On garde l'intelligence, on remplace le harnais."}
Omega aujourd'hui, c'est trois systèmes assemblés : un bot Telegram en Python — le seul vrai programme du stack — des instances de Claude Code pilotées dans des sessions tmux, et près de 200 scripts bash qui jouent le rôle de machine à états, de bus de messages, de planificateur et de moteur de règles.

Il faut être juste : l'**intelligence** de ce système est excellente. La classification d'intention, la planification, l'écriture de code, les dix-sept audits forensiques du Quality Arsenal — c'est de la vraie valeur, accumulée sur des mois. Le problème n'est pas là.

Le problème, c'est le **harnais** qui fait tourner cette intelligence. tmux est détourné en superviseur de process. Le filesystem est détourné en base de données distribuée. Le markdown est détourné en langage de programmation. Trois détournements structurels — décortiqués en section 2 — qui obligent le système à porter quatre couches de « Safety Mesh » juste pour compenser sa propre fragilité.

### Ce qu'on garde, ce qu'on remplace

| On GARDE — l'intelligence | On REMPLACE — le harnais |
|---|---|
| Le modèle d'orchestration à 4 niveaux (AISB, Oracle, Manager, Workers) | tmux + collage de prompt, remplacés par des appels typés au Agent SDK |
| Les 17 audits du Quality Arsenal | ~200 scripts bash, remplacés par un moteur typé (~15-20 modules) |
| Les Trois Lois + les principes Karpathy | `.done.json` + flock + fs-polling, remplacés par un datastore transactionnel |
| La discipline omega-v2 (ne jamais tuer un travail en cours) | 55 crons, remplacés par un planificateur typé et observable |
| Le backend Convex, le modèle zéro-credential | 4 couches de Safety Mesh, remplacées par la supervision native du moteur |

### Le verdict

Rebâtir le harnais — pas l'intelligence — sous la forme d'un **Agentic OS** : un système d'exploitation d'orchestration, programmatique, agnostique au LLM, écrit en **TypeScript sur Bun**.

Les quatre gains que tu as demandés, chacun rattaché à un mécanisme concret :

- **Fiabilité** — un état transactionnel typé remplace 200 scripts qui se coordonnent par fichiers. Les courses de polling disparaissent.
- **Maintenabilité** — ~200 scripts deviennent ~15-20 modules typés, testés, compilés. Une seule politique de retry/timeout au lieu de N implémentations divergentes.
- **Distribution** — Bun compile le moteur en un binaire autonome ; le bootstrap dépose un fichier sur n'importe quelle machine.
- **Observabilité** — l'état EST la base de données ; observer devient une requête, plus un scraping de panes tmux.

Portée : ton VPS devient l'**instance #1**, et le bootstrap distribuable est le produit OmegaGitHub. Une seule architecture, deux déploiements — la distinction « OmegaVPS / OmegaGitHub » cesse d'être deux systèmes à maintenir.

> Omega v3 garde le cerveau, jette les béquilles.
@@@SECTION
{"index":"02","eyebrow":"Diagnostic","title":"Anatomie de l'Omega actuel","lead":"Trois détournements structurels : tmux en superviseur, le filesystem en base de données, le markdown en langage."}
Avant de critiquer, reconnaissons ce qui marche. Le modèle à 4 niveaux est **sain** : il sépare proprement l'intention, la décomposition, l'exécution et la vérification. Le Quality Arsenal est de l'**expertise métier**, pas de l'infrastructure. omega-v2 a apporté une **discipline réelle** — `safe-kill.sh` refuse de tuer un travail en cours, chaque session porte un `todo.json` / `progress.json` / `mission.json`. Et le produit existe déjà : 3 060 fichiers packagés, une desktop app Tauri, un modèle d'installation « zéro credential ».

Tout cela se garde. Le diagnostic qui suit porte uniquement sur le **harnais**.

### Détournement 1 — tmux comme superviseur de process

`dispatch-to-session.sh` (44 Ko) crée une session tmux, lance le CLI `claude`, **attend l'apparition du glyphe `❯`**, colle le prompt via `tmux load-buffer` puis `paste-buffer`, appuie sur Entrée, et **réessaie jusqu'à 3 fois** si la soumission échoue. Ses propres codes de sortie le trahissent : `3` = soumission échouée, `4` = session disparue, `5` = claude n'a jamais démarré. Ce sont les échecs de transport d'un *hack* : on simule un humain qui tape dans un terminal.

Conséquence directe : la Safety Mesh L1 (brief-replay — on écrit `brief-<session>.txt` *avant* de coller, au cas où un crash surviendrait entre le collage et la vérification) et L3 (shadow manager, **14 signaux de stall**) n'existent **que** pour surveiller ce hack. Sur 166 scripts de lib, la famille patrol / watchdog / reaper / shadow / stall pèse ~18 jobs cron et des dizaines de scripts. La moitié du système surveille l'autre moitié.

### Détournement 2 — le filesystem comme machine à états distribuée

L'état d'une mission est éparpillé sur le disque : `.done.json`, le dossier `~/.aisb/state/`, des verrous flock, `inotifywait` couplé à du polling toutes les 60 s, le registre `oracles/*.json`, plus `progress.json`, `todo.json`, `mission.json`, `brief-*.txt`, `dispatch-queue.jsonl`. Des dizaines de scripts lisent et mutent ces fichiers **sans transaction, sans schéma, sans point de requête unique**.

Le plan de migration native du 14 mai l'admet lui-même : « des boucles de fs-poll qui font la course sous charge » et « une détection de fin de worker heuristique ». Le symptôme le plus parlant : un worker peut être déclaré `done_clean` puis **rétrogradé** en `pending` — parce que la vérité n'est stockée nulle part ; elle est reconstruite, à chaque tick, par des scripts qui ne sont pas d'accord entre eux.

### Détournement 3 — le markdown comme langage de programmation

Environ **22 000 tokens de règles** sont chargés à *chaque* requête. Les skills, les specs d'agents, les commandes sont de la configuration interprétée de façon **non-déterministe** par un LLM. Changer une policy, c'est réécrire du markdown que le LLM relira et réinterprétera au tour suivant — sans garantie qu'il l'applique pareil. Il n'y a ni compilation, ni vérification de types, ni test. 281 agents et 151 commandes : un patrimoine immense, mais ingouvernable comme on gouverne du code.

### Le coût cumulé

- 55 crons représentent environ **14 400 démarrages de process par jour**.
- Chaque Claude Code en tmux consomme **plusieurs gigaoctets de RAM**.
- Le système a besoin de **4 couches de Safety Mesh** parce que ses fondations ne sont pas fiables — la sûreté ne s'ajoute pas à l'architecture, elle la *compense*.

La conclusion est nette : l'intelligence d'Omega mérite d'être préservée intégralement. Le harnais doit être reconstruit. Tout le reste de ce document décrit ce harnais.
@@@SECTION
{"index":"03","eyebrow":"Architecture","title":"L'Agentic OS — vue en couches","lead":"Séparer le harnais déterministe — du code typé — de l'intelligence non-déterministe — des appels LLM. Le SST est le pont."}
Le principe fondateur d'Omega v3 tient en une distinction. Il y a **deux mondes**, et aujourd'hui ils sont mélangés dans du bash et du markdown.

- Le **harnais déterministe** : état, dispatch, cycle de vie, planification, verrous, observabilité. Ça, c'est du *code* — typé, testé, compilé. Le comportement est prévisible.
- L'**intelligence non-déterministe** : classifier, planifier, écrire du code, auditer. Ça, c'est des *appels LLM* — prompté, gradé, vérifié. Le comportement est statistique.
- Le **SST** (Single Source of Truth) est le **pont** : les règles, prompts et skills que le harnais injecte dans les appels LLM.

Omega v3 sépare nettement ces deux mondes. Le harnais cesse d'être du bash ; il devient un programme. L'intelligence cesse d'être pilotée par collage tmux ; elle devient une fonction qu'on appelle.

### Le modèle en couches

```
        ┌──────────────────────────────────────────────┐
   L1   │   TOI / un collègue   —  Telegram, CLI         │
        └────────────────────────┬─────────────────────┘
        ┌────────────────────────▼─────────────────────┐
   N1   │   AISB     intake · triage · routing · mémoire │
   N2   │   ORACLE   planification par projet · gate     │
   N3   │   MANAGER  coordination de mission · DAG       │
   N4   │   WORKERS  exécution bornée · boucle goal      │
        └────────────────────────┬─────────────────────┘
        ┌────────────────────────▼─────────────────────┐
   L0   │   OMEGA CORE   (le moteur programmatique)      │
        │   machine à états · bus typé · scheduler ·     │
        │   abstraction provider · observabilité         │
        └──────┬───────────────┬──────────────┬─────────┘
        ┌──────▼─────┐  ┌──────▼──────┐  ┌────▼─────────┐
        │  SST       │  │  RAG        │  │  8 FACTORIES │
        │  (génome)  │  │  (mémoire)  │  │  (évolution) │
        └────────────┘  └─────────────┘  └──────────────┘
```

- **L0 — Omega Core** : le substrat. Un programme, pas un agent. Il contient la machine à états, le bus de messages typé, le scheduler, l'abstraction provider et l'observabilité. C'est précisément ce qui était dispersé dans 200 scripts.
- **N1 — AISB** : reçoit l'intention (Telegram ou CLI), fait le triage, route vers le bon Oracle, gère la mémoire et les notifications.
- **N2 — Oracle** : par projet. Il planifie, produit le DAG de mission, et tient le quality gate.
- **N3 — Manager** : par mission. Il décompose en tâches, ordonnance, assigne les workers, et vérifie que **tous** ont fini avant de remonter le résultat.
- **N4 — Workers** : exécutent **une** tâche bornée, en boucle goal jusqu'à 100 % de validité, tests live inclus.
- **Transverse** : le SST (le génome — règles, prompts, skills), le RAG (la mémoire), les 8 Factories (l'évolution).

### Le glissement, en une phrase

Un worker n'est plus une session tmux dans laquelle on colle du texte. C'est un appel `runAgent()` au Agent SDK qui retourne un flux d'événements typés. **Tout le reste de cette architecture découle de cette seule phrase.**

Ton modèle à 4 niveaux n'est pas modifié — N1 à N4 sont exactement AISB, Oracle, Manager et Workers. Ce qui change, c'est le *transport* (un appel de fonction au lieu d'un collage tmux) et le *substrat* (L0, un vrai moteur, au lieu d'un tas de bash).
@@@SECTION
{"index":"04","eyebrow":"Fondations","title":"Le système de fichiers & le SST","lead":"Un dossier maître Omega/, quatre domaines Agentic-*, et un SST canonique que tout LLM consomme via un adaptateur."}
Tout commence par le rangement. Tu l'as dit clairement : installer des outils sur Ubuntu, « c'est le bordel » — tout atterrit à la racine, éparpillé. Omega v3 impose une arborescence stricte, sous un dossier maître unique.

### L'arbre maître

```
~/Omega/                          # dossier maître — tout l'Agentic OS
│
├── Agentic-Orchestration/        # LE MOTEUR
│   ├── engine/                   #   core programmatique (Bun/TS)
│   ├── missions/                 #   missions live + archivées
│   ├── state/                    #   omega.db (SQLite) — état unique
│   └── observability/            #   logs, traces, métriques
│
├── Agentic-AI/                   # PROVIDERS + CONNAISSANCE
│   ├── sst/                      #   SOURCE DE VÉRITÉ UNIQUE (neutre)
│   │   ├── rules/                #     règles communes à tous les LLM
│   │   ├── prompts/              #     templates de prompt par niveau
│   │   ├── skills/               #     skills canoniques
│   │   ├── commands/             #     commandes canoniques
│   │   ├── agents/               #     définitions d'agents & workers
│   │   ├── hooks/                #     hooks canoniques
│   │   └── automations/          #     routines planifiées
│   ├── providers/                #   un sous-dossier par fournisseur
│   │   ├── claude-code/          #     install + adaptateur
│   │   ├── glm/                  #     install + adaptateur
│   │   ├── openai/               #     install + adaptateur
│   │   └── deepseek/             #     install + adaptateur
│   ├── adapters/                 #   compilateurs SST vers provider
│   └── rag/                      #   index & stores RAG
│
├── Agentic-Tools/                # LE DOSSIER « APPLICATIONS »
│   ├── <tool>/                   #   chaque outil externe, isolé
│   └── registry.json             #   manifeste des outils installés
│
├── Agentic-Coding/               # TOUS LES PROJETS
│   ├── clients/                  #   projets clients
│   ├── work/                     #   projets internes
│   └── internal/                 #   outils & libs maison
```

Et la plomberie, à la racine de `Omega/` :

```
├── bootstrap/                    # L'INSTALLEUR (produit OmegaGitHub)
│   ├── install                   #   point d'entrée unique
│   ├── profiles/                 #   vps · macbook · minimal
│   └── manifest.yaml             #   quoi installer, quelles versions
│
├── var/                          # TOUT L'ÉPHÉMÈRE — un seul endroit
│   ├── cache/
│   ├── tmp/
│   └── logs/
│
├── .secrets/                     # credentials (jamais versionnés)
└── omega.config.ts               # configuration unique, typée
```

### Le SST — le cœur de l'agnosticisme LLM

`Agentic-AI/sst/` est **canonique et neutre**. Les règles, prompts, skills, commandes, agents, hooks et automatisations y sont écrits **une seule fois**, dans un format indépendant de tout fournisseur. C'est le génome du système.

Aucun dossier provider ne duplique le SST. Chaque provider possède un **adaptateur** qui *compile* le SST vers son format natif.

```
                 ┌──────────────────────────┐
                 │   Agentic-AI/sst/        │  ← tu écris ICI, une fois
                 │   rules · prompts ·       │
                 │   skills · commands ·     │
                 │   agents · hooks          │
                 └────────────┬─────────────┘
                              │  omega sync
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
  ┌───────────────┐  ┌────────────────┐  ┌────────────────┐
  │ claude-code   │  │ glm            │  │ openai         │
  │ adapter       │  │ adapter        │  │ adapter        │
  │ → ~/.claude/  │  │ → format GLM   │  │ → format OpenAI│
  │  settings.json│  │                │  │                │
  │  skills/*.md  │  │                │  │                │
  └───────────────┘  └────────────────┘  └────────────────┘
```

`omega sync` régénère toutes les projections provider depuis le SST. L'adaptateur Claude Code, par exemple, traduit `sst/hooks/` en blocs `hooks` de `settings.json`, et `sst/skills/` en `~/.claude/skills/*.md`. Un nouveau LLM demain ? Tu écris **un seul adaptateur** ; le SST est réutilisé tel quel. C'est exactement ça, « peu importe quelle IA j'utiliserai ».

### La discipline de nommage

Les noms de dossier sont en minuscules, soudés par un trait d'union, **jamais d'espace**. « Agentic Coding » avec une espace casserait chaque script shell, chaque chemin, chaque glob — et même le moteur v3 aura des points de contact shell (git, build, outils externes). Le préfixe `Agentic-` est conservé sur les 4 domaines, pour la lisibilité et pour porter ton intention, mais il est soudé.

### Le « bordel Ubuntu », résolu

Quatre domaines, zéro ambiguïté. Si c'est jetable → `var/` (cache, tmp, logs, un seul endroit). Si c'est un outil externe → `Agentic-Tools/` (section 10). Si c'est de la connaissance ou un provider → `Agentic-AI/`. Si c'est un projet → `Agentic-Coding/`. Si c'est le moteur → `Agentic-Orchestration/`. Plus rien à la racine du home, plus rien dans `/opt`, plus de fichiers temporaires perdus.
@@@SECTION
{"index":"05","eyebrow":"Multi-LLM","title":"L'abstraction provider — un contrat, tous les LLM","lead":"Le moteur ne parle qu'à une interface AgentProvider. Changer de LLM devient de la configuration, pas du code."}
Aujourd'hui, Omega est soudé à Claude Code. Le dispatch colle du texte dans le CLI `claude`. Vouloir GLM ou OpenAI demain imposerait de tout réécrire. Omega v3 brise ce soudage avec une seule interface, que **chaque** fournisseur implémente.

### Le contrat

```typescript
interface AgentProvider {
  readonly id: string;                  // "claude-code" | "glm" | "openai" | "deepseek"
  capabilities(): ProviderCapabilities;  // tool use, streaming, contexte, MCP...
  run(req: AgentRequest): AsyncIterable<AgentEvent>;
  cost(usage: Usage): number;
}

type AgentEvent =
  | { type: "thinking";    text: string }
  | { type: "text";        text: string }
  | { type: "tool_use";    name: string; input: unknown }
  | { type: "tool_result"; name: string; output: unknown }
  | { type: "done";        result: AgentResult }
  | { type: "error";       error: OmegaError };
```

Le moteur d'orchestration ne connaît **que** `AgentProvider`. Il ne sait pas — et n'a pas besoin de savoir — si, derrière, c'est Claude, GLM ou OpenAI.

### Les adaptateurs

- **ClaudeProvider** — enveloppe le **Claude Agent SDK**, c'est-à-dire « Claude Code en bibliothèque » : la même boucle d'agent, les sous-agents, MCP, les hooks, les permissions, la compaction de contexte — mais appelable depuis le code, avec des événements typés en streaming. C'est l'adaptateur de référence.
- **OpenAIProvider** — enveloppe l'OpenAI Agents SDK / l'API Responses.
- **GLMProvider** — enveloppe l'API Zhipu GLM.
- **DeepSeekProvider** — enveloppe l'API DeepSeek.

Chaque adaptateur fait **une seule chose** : traduire l'API native du fournisseur vers le schéma `AgentEvent` commun. Environ 200 à 400 lignes par adaptateur.

### Affectation par rôle

Comme tout passe par le même contrat, tu peux affecter un provider différent **par rôle ou par niveau**, en configuration :

```typescript
// omega.config.ts
export const providers = {
  aisb:    "glm",          // triage : un modèle rapide et économique suffit
  oracle:  "claude-code",  // planification : le meilleur raisonnement
  manager: "claude-code",
  worker:  "claude-code",  // écriture de code : Claude Opus
  audit:   "openai",       // un second avis, modèle différent
};
```

Deux bénéfices en cascade :

- **Robustesse** — un audit mené par un modèle *différent* de celui qui a écrit le code est une vraie falsification au sens de Popper (Seconde Loi). Le biais d'un modèle ne se valide pas lui-même.
- **Coût** — les niveaux à faible enjeu (triage, classification) tournent sur des modèles bon marché ; on réserve Claude Opus à l'écriture de code.

Ce que cette section rend vrai : « peu importe quelle IA j'utiliserai demain ». Un nouveau fournisseur = un nouvel adaptateur + une ligne de config. Zéro changement dans le moteur, zéro changement dans le SST.
@@@SECTION
{"index":"06","eyebrow":"Le moteur","title":"Le moteur d'orchestration","lead":"Les 4 niveaux deviennent des nœuds typés. La mission devient un objet. L'état devient une base de données transactionnelle."}
Le moteur — `Agentic-Orchestration/engine/` — est le L0 de l'architecture. Voici comment il fonctionne.

### Les 4 niveaux comme modules typés

Chaque niveau (AISB, Oracle, Manager, Worker) devient un module du moteur. Un niveau = une fonction qui prend un contexte typé, fait un appel `AgentProvider`, et produit un résultat typé. Plus de session tmux, plus de prompt collé.

### La Mission, objet de première classe

```typescript
type Mission = {
  id: string;
  project: string;
  intent: string;       // l'intention en langage naturel (venue de L1)
  status: "planning" | "running" | "auditing" | "shipped" | "failed";
  dag: TaskNode[];      // le graphe de tâches, décomposé par l'Oracle
  createdAt: string; updatedAt: string;
};

type TaskNode = {
  id: string;
  dependsOn: string[];               // les arêtes du DAG
  status: "pending" | "running" | "verifying" | "done" | "failed";
  assignee?: WorkerRef;
  scope: { filesOwned: string[] };   // verrou d'ownership — typé, pas du flock
  goal: GoalSpec;                    // critère de succès vérifiable
  evidence?: RuntimeEvidence;        // preuve runtime (Première Loi)
};
```

### Le cycle de vie d'une mission

```
 Telegram/CLI ─▶ AISB.classify() ─▶ Oracle.plan() ──▶ Mission{dag}
                                                          │
                       ┌──────────────────────────────────┘
                       ▼
              Manager.schedule(dag)
                       │  ordonnance par dépendances ; lance en
                       │  parallèle les tâches aux scopes disjoints
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
   Worker.run()   Worker.run()   Worker.run()   ← appels provider concurrents
        │              │              │
        └──────────────┼──────────────┘
                       ▼
              Manager.verifyAll()    ← tous les workers done_clean ?
                       ▼
              Oracle.qualityGate()   ← le close-gate (section 12)
                       ▼
              AISB.report()          ← synthèse vers Telegram (DM + topic)
```

### L'état : une vraie base de données

`Agentic-Orchestration/state/omega.db` — un SQLite embarqué (Bun possède un driver SQLite natif et rapide). **Une** source de vérité, typée, transactionnelle, requêtable. Elle remplace, à elle seule : `.done.json`, `oracles/*.json`, `progress.json`, `todo.json`, `mission.json`, `brief-*.txt`, `dispatch-queue.jsonl`, et les verrous flock.

Pour le produit multi-utilisateur, le même schéma se projette sur **Convex** (déjà présent dans `omega/convex-backend/`). SQLite pour l'instance VPS solo, Convex pour le SaaS : même schéma typé, deux backends.

### La fin d'un worker : un retour, pas un polling

L'appel `provider.run()` **retourne** quand l'agent a fini. L'événement `done` *est* le signal. Plus d'`inotifywait`, plus de polling 60 s, plus de détection heuristique « le worker semble inactif ». `.done.json` peut continuer d'être *écrit* (un export de compatibilité pour d'éventuels outils externes), mais il n'est plus *le mécanisme*.

### Concurrence & ressources

Le moteur est un seul process (ou un petit pool) qui fait tourner une boucle d'événements asynchrone et gère N appels d'agents concurrents. Plus de session tmux, plus un Claude Code complet par worker. La RAM passe de « plusieurs gigaoctets par session » à « un process moteur, plus des appels d'API ».

### Verrous & ownership

Le `scope.filesOwned` d'une tâche est un verrou typé, en mémoire ou en base — pas un `flock`. Le Manager ne lance en parallèle que des tâches aux scopes **disjoints** : exactement la règle de batch Linear actuelle, mais vérifiée par le moteur, pas espérée d'un script.

### La discipline omega-v2, préservée

« Ne jamais tuer un travail en cours » devient une **propriété de la machine à états**, pas une grille `safe-kill.sh`. Une tâche ne peut pas être à la fois `running` et `killed` : les transitions illégales sont interdites par le type. Le résultat d'omega-v2 (todo, progress, heartbeat) reste — exprimé comme des champs typés de `TaskNode`.

### Gestion d'échec unifiée

Retries, timeouts, circuit breakers, backpressure : **une seule** politique, typée, dans le moteur. Plus de logique de retry réinventée différemment dans 40 scripts — un défaut que le plan de migration native pointait déjà : « une logique retry/timeout hand-rolled qui diverge entre scripts ».
@@@SECTION
{"index":"07","eyebrow":"Prompts","title":"Le prompt engineering automatisé entre niveaux","lead":"Chaque hand-off génère son prompt depuis un template typé, le contexte de mission, le SST et la mémoire RAG. Composé, versionné, testé."}
Aujourd'hui, les prompts sont construits par `prompts.py` côté bot et `oracle-prompt.sh` (27 Ko !) côté lib — de la concaténation de chaînes en bash et en Python, plus une injection de « knowledge-pack » par un script qui `grep` des fichiers. C'est fragile, non testé, non versionné.

Omega v3 introduit la **Prompt Factory** (le premier des huit « éducateurs » — voir section 8). Chaque hand-off entre niveaux — AISB vers Oracle, Oracle vers Manager, Manager vers Worker — génère son prompt programmatiquement, à partir de quatre entrées :

```
   template typé (par niveau)
     +  contexte de mission (Mission, TaskNode)
     +  règles SST pertinentes (sélectionnées, pas les 22K tokens)
     +  contexte RAG (récupéré, gradé — section 9)
     +  leçons des missions passées (SMITH)
     ───────────────────────────────────────────
     =  un prompt composé, versionné, traçable
```

### Trois propriétés que la concaténation bash n'a pas

1. **Composé, pas concaténé.** Un template de prompt est du code : typé, avec des slots nommés, testable unitairement. On peut écrire un test qui vérifie qu'un prompt Worker contient toujours son `goal` et son `scope`.
2. **Versionné.** Chaque prompt émis porte la version du template qui l'a produit. Quand un résultat dérape, on remonte au template exact — une régression de prompt devient traçable, au lieu d'être invisible.
3. **Pertinent.** Au lieu de charger 22 000 tokens de règles à chaque tour, le moteur sélectionne les règles SST utiles à *cette* tâche précise. Moins de tokens, plus de signal, moins de bruit pour le modèle.

### Le « N4 prompt engineering automated »

La Prompt Factory ne fait pas que générer — elle **s'auto-règle**. Le moteur enregistre, pour chaque template, le taux de succès au premier essai et le taux de reprise (rework). Les templates qui mènent au rework sont signalés, réécrits — par la Factory elle-même, sous quality gate — puis re-mesurés.

Le prompt engineering devient une **boucle fermée pilotée par les outcomes**. C'est précisément ce que tu décrivais par « L4/N4 prompt engineering automated by Agentik AI systems » : ce ne sont plus des humains qui ajustent des prompts à la main, c'est le système qui mesure, propose et améliore ses propres prompts.
@@@SECTION
{"index":"08","eyebrow":"Auto-amélioration","title":"Les 8 Factories — le système qui se régénère","lead":"Tes 8 éducateurs : des méta-agents qui génèrent et maintiennent chaque catégorie de brique. Le SST est le génome ; les Factories sont l'évolution."}
Définissons d'abord le mot. Une **Factory** — ton « éducateur » — est un méta-agent qui *génère et maintient* une catégorie de composant du système, et qui écrit le résultat dans le SST. Le SST est le génome d'Omega ; les Factories sont la façon dont ce génome évolue. Une Factory = un agent (un appel provider) + du code typé qui valide et installe le résultat.

### Les huit

| # | Factory (« éducateur ») | Génère & maintient | Généralise aujourd'hui |
|---|---|---|---|
| 1 | **Prompt Factory** | les prompts inter-niveaux, auto-réglés sur les outcomes | `prompts.py`, `oracle-prompt.sh` |
| 2 | **Artifact Factory** | les livrables : rapports, PDF, templates, sorties | `pdfgen`, génération ad hoc |
| 3 | **Skill Factory** | de nouveaux skills, maintient le catalogue | `skill-creator`, `skill-validator` |
| 4 | **Agent Factory** | les définitions d'agents & de co-workers | les 281 specs écrites à la main |
| 5 | **Connection Factory** | serveurs MCP, intégrations API, connexions provider | la configuration MCP manuelle |
| 6 | **Automation Factory** | hooks + routines planifiées | les 55 crons écrits à la main |
| 7 | **Platform Watcher** | suit les releases Claude Code / GLM / OpenAI, met à jour les adaptateurs | Pythia (généralisé à tous les providers) |
| 8 | **Goal/Loop Factory** | définit & règle la boucle goal et les flux de test live | `/goal`, `/loop` opt-in |

Chaque Factory écrit dans le SST. Chaque écriture est ensuite projetée vers tous les providers par `omega sync`. Une amélioration produite une fois bénéficie immédiatement à Claude, GLM et OpenAI.

### Le moteur de l'auto-amélioration

SMITH — la boucle d'apprentissage déjà existante — devient le **signal de feedback** qui pilote les huit Factories.

```
   missions exécutées ─▶ outcomes mesurés
                         (succès, rework, coût, temps)
            │
            ▼
        SMITH analyse ─▶ « le template X cause du rework »
            │            « il manque un skill Y »
            ▼
   la Factory concernée génère le correctif
            │
            ▼
        quality gate ─▶ SST mis à jour ─▶ omega sync
            │
            ▼
     tous les providers reçoivent l'amélioration
```

### Gouvernance — jamais d'auto-modification aveugle

Une Factory **propose** ; elle n'impose pas. Toute écriture dans le SST passe par un quality gate (les audits du Quality Arsenal) et, selon un seuil de confiance configurable, soit par un accord humain (un bouton dans Telegram), soit par une adoption autonome.

C'est le modèle actuel de Pythia — « propose, n'applique jamais » — généralisé à toute l'auto-amélioration. Le SST est versionné en git : chaque changement produit par une Factory est diff-able et réversible. Le système peut évoluer seul, mais jamais sans laisser de trace ni sans filet.
@@@SECTION
{"index":"09","eyebrow":"Mémoire","title":"Le sous-système multi-RAG","lead":"Cinq stratégies de RAG, un routeur qui choisit la meilleure par requête. La récupération de contexte devient gradée, plus grepée."}
Aujourd'hui, l'injection de contexte se résume à `knowledge-pack-builder.sh` : un script bash qui `grep` des fichiers. Aucune notion de pertinence, aucune mémoire sémantique, aucun graphe.

Omega v3 installe un véritable sous-système RAG sous `Agentic-AI/rag/`, avec un **routeur** qui choisit la stratégie adaptée à chaque requête.

### Les cinq stratégies

| Stratégie | Quand l'utiliser | Mécanisme |
|---|---|---|
| **Hybrid RAG** | défaut — recherche de code & de doc | embeddings denses + BM25 sparse, fusion des scores |
| **Graph RAG** | « comment X est lié à Y », questions cross-projet, architecture | graphe entités/relations, parcours multi-saut |
| **Agentic RAG** | questions ouvertes, exploration | l'agent décide quoi récupérer, en plusieurs sauts |
| **Corrective RAG (CRAG)** | quand la justesse prime | auto-évalue les chunks ; si faible pertinence, re-récupère ou bascule en repli web |
| **Multimodal RAG** | screenshots, PDF, diagrammes, captures d'UI | embeddings multimodaux |

### Le routeur

```
   requête ─▶ classifieur (heuristique + petit appel LLM)
                 │
     ┌───────────┼───────────┬────────────┬─────────────┐
     ▼           ▼           ▼            ▼             ▼
  Hybrid      Graph       Agentic       CRAG       Multimodal
```

Le **CRAG** mérite une mention spéciale : il auto-note la qualité de ce qu'il récupère et re-récupère si c'est faible. C'est exactement l'obsession « check / recheck jusqu'à 100 % » que tu demandes — appliquée, ici, à la récupération de contexte.

### Stockage & intégration

- `Agentic-AI/rag/` héberge un vector store (Chroma, déjà en place dans le système actuel), un graph store et un doc store. Les embeddings se font par API (Voyage, OpenAI).
- La mémoire cross-projet existante (claude-mem, l'agent MEROVINGIAN) devient le **Graph RAG** : la connaissance qui relie plusieurs projets est, par nature, un graphe.
- Le RAG alimente le contexte de *chaque* niveau, via la Prompt Factory (section 7). Le contexte injecté est désormais **gradé par pertinence**, plus simplement grepé.

Le gain net : on remplace un `grep` de fichiers par une récupération dont la qualité est mesurée — et corrigée — avant d'atteindre le modèle.
@@@SECTION
{"index":"10","eyebrow":"Outils","title":"Agentic-Tools — le dossier « Applications »","lead":"Chaque outil externe dans son propre dossier, déclaré dans un registre. Fini les installations Ubuntu éparpillées à la racine."}
Le problème, tel que tu l'as décrit : sur Ubuntu, installer des outils open-source « c'est le bordel » — ça atterrit dans `/usr/local/bin`, `/opt`, `~/.local`, les globals npm, des dotfiles. Impossible de savoir ce qui est installé, ni de désinstaller proprement.

La solution : `Agentic-Tools/`, calqué sur le dossier `/Applications` de macOS. **Tout** outil installé depuis l'extérieur vit dans `Agentic-Tools/<outil>/` — isolé, autonome, repérable.

### Le registre

```json
// Agentic-Tools/registry.json
{
  "tools": [
    {
      "name": "rtk",
      "version": "1.4.0",
      "path": "Agentic-Tools/rtk/",
      "source": "github:org/rtk",
      "invoke": "Agentic-Tools/rtk/bin/rtk",
      "installedAt": "2026-05-22",
      "providers": ["claude-code", "glm"]
    }
  ]
}
```

### Le flux d'installation

```
   omega tool install <nom>
        │
        ├─ 1. télécharge & installe dans Agentic-Tools/<nom>/
        │      (jamais à la racine système)
        ├─ 2. enregistre l'entrée dans registry.json
        ├─ 3. expose l'outil aux providers via le SST
        │      (génère une commande/skill SST, puis omega sync)
        └─ 4. l'outil est immédiatement disponible pour tout LLM
```

Cycle de vie complet : `omega tool list / update / remove`. Désinstaller, c'est supprimer un dossier et une entrée du registre. Zéro pollution système, zéro résidu.

### Le bénéfice bootstrap

Le `registry.json` **est** le manifeste de ce qu'il faut réinstaller. Sur une machine neuve, le bootstrap lit le registre et réinstalle exactement le même jeu d'outils, aux mêmes versions. La couche Tools rend le système reproductible.

### Le rangement automatique

Tu l'avais demandé explicitement : « quand j'installe un nouvel outil, automatiquement il est mis dans le bon dossier ». C'est `omega tool install` qui l'impose — il n'existe aucun autre chemin d'installation. Le bon dossier n'est pas une convention qu'on espère respecter ; c'est le **seul** chemin que le système connaît. On ne peut pas mal ranger un outil parce qu'il n'y a qu'une seule place où le mettre.
@@@SECTION
{"index":"11","eyebrow":"Qualité","title":"Exécution goal-driven & tests live","lead":"Chaque worker boucle : exécute, vérifie (build + tests + flux live), grade, et recommence tant que ce n'est pas 100 %."}
Ton exigence : l'agent fait le travail, le check et le recheck jusqu'à ce que ce soit 100 % valide et fonctionnel, avec de vrais tests en live, de vrais tests de flow. Omega v3 en fait le **modèle d'exécution par défaut** de chaque worker.

### La boucle goal

```
        ┌──────────────────────────────────────────────┐
        ▼                                              │
   1. EXÉCUTE la tâche                                  │
        │                                              │
   2. VÉRIFIE                                           │
        ├─ build + typecheck                            │
        ├─ tests unitaires / intégration                │
        └─ TESTS LIVE — Playwright headless sur l'URL    │
           prod : golden path + cas limites             │
        │                                              │
   3. GRADE  (outcomes : multi-grader, passe Popper)    │
        │                                              │
   4. 100 % ? ──non──▶ analyse l'écart ▶ corrige ───────┘
        │
       oui ─▶ tâche done_clean (+ preuve runtime attachée)
```

La boucle est **bornée** : un plafond d'itérations et un plafond de coût en tokens empêchent les boucles infinies — exactement les garde-fous R-28 (cost accounting) déjà conçus dans le système actuel.

### Ce qui change par rapport à aujourd'hui

- L'infrastructure outcomes/grader (R-19 à R-30 : rubrique, multi-grader à consensus, passe adverse de Popper) **existe déjà**, mais elle est *opt-in*. Omega v3 en fait le défaut : aucun worker ne se déclare `done` sans avoir bouclé.
- Le moteur embarque un **Test Runner** de première classe, qui pilote Playwright en headless contre les URL **prod** — conforme à tes règles : URL prod, Playwright en CLI, jamais via MCP, jamais de dev server.
- La **Première Loi devient une contrainte d'architecture**. Une `TaskNode` ne peut pas passer à `done` sans un champ `evidence: RuntimeEvidence` — sortie de test, screenshots, status HTTP. « Le code ment, seul le runtime dit la vérité » n'est plus une consigne qu'on espère respectée ; c'est un champ obligatoire du type. Le compilateur lui-même refuse une tâche terminée sans preuve.

### Le réglage de la boucle

La **Goal/Loop Factory** (éducateur n°8) règle cette boucle : quels tests pour quel type de tâche, quels seuils de réussite, combien d'itérations avant d'escalader vers le Manager. Elle apprend des outcomes quels flux de vérification attrapent réellement les régressions — et lesquels ne servent qu'à brûler des tokens.
@@@SECTION
{"index":"12","eyebrow":"Confiance","title":"Observabilité & sûreté","lead":"L'état EST la base de données : observer, c'est interroger. Les 4 couches de Safety Mesh fondent en correctness native plus un vrai quality gate."}
### Observabilité

- **Logs structurés** — un seul format d'événement typé, à la place du `echo` éparpillé dans 200 scripts.
- **Traces** — chaque mission est un arbre de trace : mission, puis tâches, puis appels d'agents, puis appels d'outils. On peut rejouer n'importe quelle mission de bout en bout.
- **Métriques** — latence, coût, tokens, taux de succès au premier essai, taux de rework — par niveau et par provider.
- **Dashboard** — le dashboard existant (`agentik-monitor/dashboard`) est réorienté : il interroge `omega.db`. Comme l'état *est* la base, observer = exécuter une requête. Plus de reconstruction depuis `/tmp/aisb-sessions.json`, plus de scraping de panes tmux.

### La Safety Mesh, repensée

Les quatre couches actuelles existent surtout pour compenser le hack tmux. Leur *intention* est préservée ; leur *implémentation* fond.

| Couche actuelle | Rôle | Devenir en v3 |
|---|---|---|
| **L1 — Brief-replay** | persister le dispatch avant le collage | **Disparaît.** Pas de collage à perdre — la mission est une ligne en base, durable par construction. |
| **L2 — CPU Guard** | contrôle d'admission sous charge | **Devient** un vrai limiteur de concurrence avec backpressure, typé, dans le scheduler. |
| **L3 — Shadow Manager** (14 signaux de stall) | détecter les sessions bloquées | **Disparaît en grande partie.** Pas de tmux à bloquer ; le moteur a de vrais timeouts et heartbeats sur les appels d'agents. |
| **L4 — Mission Auditor** (close-gate) | quality gate au hand-off | **Conservé.** C'est de la vraie valeur. Devient une transition typée de la machine à états. |

### Surveillance programmatique ET agentique

Tu as demandé une surveillance « programmatique et agentique entre les couches ». Omega v3 a les deux :

1. **Supervision déterministe** — le moteur supervise ses propres agents : timeouts, heartbeats, limites de concurrence, circuit breakers. Du code, prévisible, sans appel LLM.
2. **Revue agentique** — un méta-agent « Observer » (parent de Pythia et de SMITH) relit périodiquement les traces et les métriques, pour repérer les anomalies qu'un simple seuil ne voit pas : un projet qui converge lentement, un template de prompt qui dérive, un provider qui se dégrade. Il *propose* — il ne corrige pas seul.

Le bilan : on passe de quatre couches de bash qui compensent une fondation fragile, à une **correctness native** plus un vrai quality gate plus un reviewer agentique. La sûreté ne compense plus l'architecture ; elle s'ajoute à une architecture déjà saine.
@@@SECTION
{"index":"13","eyebrow":"Distribution","title":"Le Bootstrap — un VPS ou un MacBook neuf, une commande","lead":"Le produit OmegaGitHub. Le moteur compilé en un binaire ; le bootstrap déploie tout l'arbre Omega/ et démarre le service."}
L'objectif : tu achètes un VPS neuf, ou tu sors un MacBook neuf — une commande — et tout l'Agentic OS est debout. C'est ton « First Bootstrap VPS installation » : l'architecture est *orientée bootstrap*. À partir de cette amorce, tout le reste se met en place.

### Ce que le bootstrap installe

```
   omega bootstrap --profile=vps
        │
        ├─ 1. détecte l'OS + le gestionnaire de paquets (Ubuntu / macOS)
        ├─ 2. crée l'arbre ~/Omega/ (4 domaines + bootstrap + var)
        ├─ 3. installe le binaire moteur (un seul fichier, compilé par Bun)
        ├─ 4. installe les providers déclarés (claude-code, glm, ...)
        ├─ 5. compile le SST vers chaque format provider (omega sync)
        ├─ 6. restaure les outils depuis registry.json
        ├─ 7. écrit omega.config.ts + .secrets/ (interactif ou via env)
        └─ 8. démarre le service moteur (systemd sur VPS, launchd sur Mac)
```

### Un binaire, pas 3 060 fichiers

Aujourd'hui le produit Omega, c'est 3 060 fichiers et trois installeurs bash — `setup` (759 lignes), `install.sh` (611 lignes), `bootstrap.sh` (386 lignes) — exécutés en 14 étapes. Avec Bun, le moteur **se compile en un binaire autonome**. Le bootstrap se réduit à : télécharger un binaire (ou cloner le repo), puis lancer `omega bootstrap`. Les 14 étapes bash s'effondrent, parce qu'il n'y a plus 3 060 fichiers à disperser sur le disque — il y a un binaire et un arbre.

### Les profils

- **vps** — headless, service systemd, bot Telegram, sans interface graphique.
- **macbook** — launchd, app desktop optionnelle (la desktop app Tauri existante est réutilisée).
- **minimal** — le moteur seul, pour du CI ou un conteneur.

### Les propriétés

- **Idempotent** — relancer le bootstrap répare la dérive de configuration, ne casse rien.
- **Cross-platform** — Ubuntu et macOS, par détection d'OS (la logique existe déjà dans `bootstrap.sh`).
- **Zéro credential embarqué** — le modèle actuel est conservé. L'utilisateur connecte ses propres comptes après l'installation. Les secrets vivent dans `~/Omega/.secrets/`, jamais dans l'arbre versionné.

### Une architecture, deux déploiements

Le repo bootstrap **est** le produit OmegaGitHub. Ton VPS est l'instance #1. Le MacBook d'un collègue sera l'instance #2 — même repo, même commande. Tu ne maintiens plus deux Omega (le système perso *versus* le produit) : tu maintiens **un** Agentic OS, et le bootstrap est la façon dont il s'instancie. C'est la résolution propre de la distinction « OmegaVPS / OmegaGitHub » : ce n'est plus deux systèmes, c'est un système et son installeur.
@@@SECTION
{"index":"14","eyebrow":"Migration","title":"Plan de migration — améliorer sans casser","lead":"Stratégie strangler-fig : le moteur v3 grandit à côté de l'Omega actuel, qui continue de tourner. Jamais de big-bang."}
Ta consigne était explicite : « il faut pas casser, il faut améliorer ». Donc : aucune réécriture big-bang. La méthode est le **strangler-fig** (le figuier étrangleur) — le nouveau moteur grandit à côté de l'ancien système, lui prend ses responsabilités une par une, et l'ancien chemin n'est retiré qu'après vérification.

### Les phases

| Phase | Ce qu'on fait | On retire (après dual-run vérifié) |
|---|---|---|
| **0 — Fondations** | Créer l'arbre `~/Omega/`, le squelette du moteur (Bun/TS), `omega.db`, l'interface `AgentProvider` + `ClaudeProvider`. Aucun changement de comportement. | rien |
| **1 — Étrangler le dispatch worker** | Les NOUVEAUX workers passent par le moteur (appels Agent SDK) au lieu de `dispatch-to-session.sh`. tmux reste en fallback. Dual-run, comparaison. | — |
| **2 — Étrangler l'état** | Missions & tâches dans `omega.db`. `.done.json` devient un export de compatibilité. | `oracles/*.json`, `progress.json`, verrous flock |
| **3 — Étrangler patrol/sûreté** | La supervision native du moteur remplace patrol / watchdog / reaper. | ~18 crons de patrol |
| **4 — Migrer le SST** | Règles, skills, commandes, agents vers `Agentic-AI/sst/`. L'adaptateur claude-code recompile vers `~/.claude/`. Comportement vérifié identique. | la dispersion markdown |
| **5 — Les Factories** | Construire les huit, en commençant par Prompt Factory + Platform Watcher (Pythia généralisé). | génération manuelle de prompts/crons |
| **6 — Multi-provider** | Ajouter les adaptateurs GLM / OpenAI / DeepSeek. | le soudage à Claude uniquement |
| **7 — Le bootstrap** | Packager le moteur, construire l'installeur, tester sur un VPS neuf. | les 3 installeurs bash + 3 060 fichiers |

La règle de chaque phase : dual-run, mesurer, *puis* retirer l'ancien chemin. Jamais l'inverse.

### Les zones sacrées

Conformément à tes instructions et au plan de migration native existant, ces zones ne sont touchées qu'en dernier, et uniquement en remplacement à l'identique :

- `account.py` et toute la facturation ;
- l'OAuth et la rotation multi-comptes (`claude-oauth.sh`) ;
- tous les fichiers `.env` et les credentials.

### Le bot Python

Il reste en service comme **passerelle Telegram** pendant toute la migration, et il est étranglé en dernier — soit porté en TypeScript, soit conservé comme une fine couche de passerelle qui parle au moteur via une API locale. Cette décision se prend en Phase 6 ou après, pas maintenant.

### Réversibilité

À chaque phase, l'ancien chemin reste fonctionnel jusqu'à la preuve du dual-run. Le SST est versionné en git. Aucune phase n'est irréversible. C'est exactement la prudence du plan de migration native du 14 mai — appliquée, cette fois, à une refonte plus profonde du harnais.
@@@SECTION
{"index":"15","eyebrow":"Décision","title":"Python ou Bun — la décision, et la feuille de route","lead":"Bun + TypeScript, un seul langage. Le binaire autonome décide pour la distribution ; la parité avec le Agent SDK décide pour la fiabilité."}
Tu as demandé un avis clair. Le voici, argumenté.

### La comparaison

| Critère | Python | Bun + TypeScript | Gagnant |
|---|---|---|---|
| Claude Agent SDK | SDK officiel, solide | SDK de première classe (Claude Code est lui-même en TS) | TS (léger) |
| Distribution | venv / `uv` à gérer | **compile en un binaire autonome** | **Bun (décisif)** |
| Produit existant | — | Convex backend en TS, desktop app Tauri | TS |
| Bot existant | en Python | — | Python (seul poids) |
| Écosystème RAG | plus riche | suffisant (orchestration + DB externes) | Python (léger) |
| Concurrence / streaming | asyncio, correct | boucle d'événements, runtime plus rapide | TS (léger) |

### Le verdict : Bun + TypeScript, un seul langage

- **La distribution tranche.** Tu veux pouvoir déposer Omega sur le VPS d'un collègue ou sur un MacBook neuf. Un binaire autonome compilé par Bun, c'est un fichier. Un projet Python, c'est un runtime, un venv, des versions à aligner. Pour un produit qu'on installe ailleurs, c'est décisif.
- **La parité tranche pour la fiabilité.** Le Claude Agent SDK et Claude Code sont écrits en TypeScript. Bâtir le moteur en TS maximise la fidélité de comportement avec l'outil qu'Omega pilote depuis le premier jour.
- **Le RAG ne justifie pas Python.** Le RAG d'Omega v3 est surtout de l'orchestration — le routeur, CRAG, le multi-saut — plus des services externes (vector DB, graph DB, embeddings par API). L'orchestration, c'est précisément ce que le moteur fait bien. Pas besoin d'un sidecar Python ; on ne l'ajoutera que si une bibliothèque précise se révèle incontournable — et par défaut, ce ne sera pas le cas.
- **Le bot Python** reste en passerelle pendant la migration, puis est étranglé (Phase 6+). C'est le seul poids Python, et il est temporaire.

Un seul langage = une seule toolchain, un seul setup de test, une seule surface. C'est l'option « simplicité d'abord » de Karpathy — et c'est la bonne.

### La feuille de route

Le rythme suit les sept phases de la section 14. Pas de fausse échéance — la qualité est la contrainte, pas le temps. Trois repères :

- **Socle (Phases 0-2)** — l'arbre, le moteur, le datastore, le ClaudeProvider, le dispatch worker étranglé. C'est le cœur ; c'est là que se gagnent la fiabilité et la maintenabilité.
- **Autonomie (Phases 3-5)** — supervision native, SST, les huit Factories. C'est là que se gagnent l'observabilité et l'auto-amélioration.
- **Portée (Phases 6-7)** — multi-provider et bootstrap. C'est là que se gagne la distribution.

Chaque phase est livrable et démontrable seule. Tu peux t'arrêter après le socle et avoir déjà un Omega bien plus fiable ; les phases suivantes ajoutent de la portée, pas des prérequis.

### La thèse, en une ligne

Omega v3 garde le cerveau — le modèle à 4 niveaux, le Quality Arsenal, les Trois Lois — et jette les béquilles : tmux, le bash, le markdown-comme-runtime. L'intelligence était déjà là. On lui donne enfin un corps qui ne tremble pas.
