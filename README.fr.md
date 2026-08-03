# OmegaOS

Un plan de contrôle en terminal pour piloter en parallèle une flotte d'agents de codage IA, où chaque agent obéit au même corpus de règles typé.

[English](README.md) | Français | [Русский](README.ru.md) | [中文](README.zh.md) | [Español](README.es-ES.md)

> Le [README anglais](README.md) est la version canonique et la plus à jour ; cette traduction peut avoir un temps de retard.

[![CI](https://github.com/agentik-os/OmegaOS/actions/workflows/ci.yml/badge.svg)](https://github.com/agentik-os/OmegaOS/actions/workflows/ci.yml) ![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg) ![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)

OmegaOS n'est pas une bibliothèque qu'on importe. On l'installe sur une machine Linux. On récupère la commande `omega`, une TUI pour surveiller et tuer les sessions, et une couche d'orchestration qui distribue le travail aux agents. Il y a aussi un pont Telegram, si l'on veut piloter le tout depuis son téléphone.

Le runtime d'agent par défaut est OpenAI Codex. Claude Code, Gemini, Pi, Hermes et GLM restent disponibles comme choix explicites. Chaque agent reçoit un contexte compact, typé et adapté à son rôle, compilé depuis la même doctrine.

Version courante : voir [CHANGELOG.md](CHANGELOG.md) (`omega -V` sur une machine installée). Je m'en sers tous les jours ; attendez-vous à des aspérités.

## La doctrine

Il existe un registre typé de 7 Lois et de 47 Règles opérationnelles nommées. `omega rules list` affiche l'ensemble courant. Son compilateur vit dans `crates/omega-core/src/rules.rs` et impose un budget OmegaOS de 24 Ko.

**Les Lois sont inviolables.** Elles s'imposent à chaque agent et priment sur toute règle et toute tâche. Il y en a sept :

- **L0 — Livrer la vérité.** Un changement n'est terminé que lorsqu'une recompilation propre le reproduit et qu'il est poussé. En dessous, c'est un brouillon.
- **L1 — Le runtime est la seule vérité.** Le code et les commentaires énoncent l'intention. Seule l'exécution révèle la réalité. En cas de désaccord, c'est le runtime qui tranche.
- **L2 — Chercheur, pas flagorneur.** Quand une prémisse est bancale, on la conteste avec des arguments avant d'agir. Pas de fausse assurance. « Ça devrait marcher » sans preuve, c'est un mensonge.
- **L3 — Décider et avancer.** Un agent dépêché est autonome. Il ne s'arrête jamais pour demander « est-ce que je continue ? » Il décide, exécute, et rend compte ensuite.
- **L4 — Terminé veut dire 100 %, vérifié.** 92 %, ce n'est pas terminé. On énumère les tâches, on finit chacune, on vérifie chacune face au runtime.
- **L5 — La qualité avant la vitesse.** Pas de variante allégée, simplifiée ou expédiée d'un vrai protocole. Un 403 ou un 401, c'est un abandon, pas un succès.
- **L6 - Finir la mission.** Énumérer, exécuter, vérifier et rapporter chaque livrable demandé. Un plan ou une phase partielle n'est pas un arrêt valide.

**Les Règles sont opérationnelles.** Nommées (R-SCOPE, R-VERIFY, R-CITE…), réparties entre Universal, QualityGate, Orchestration, Reporting et Safety. Chaque Règle est cadrée selon les rôles auxquels elle s'applique : Master, Oracle, Worker. On n'encombre pas un worker de règles d'orchestration sur lesquelles il ne peut rien, et un oracle n'hérite pas de la discipline de verrouillage de fichiers du worker. Même registre, tranches différentes.

### L'entonnoir

C'est là qu'est le mécanisme. `rules::compile_rule_context_for_provider` combine le noyau compact des Lois, le contrat du rôle, les Règles pertinentes pour la mission et la mécanique du fournisseur. Il refuse tout contexte OmegaOS dépassant 24 Ko au lieu de le tronquer.

Un worker situé trois niveaux plus bas dans l'arbre porte les sept mêmes Lois que le Master tout en haut. Personne ne peut engendrer un enfant qui laisse discrètement tomber L5 pour aller plus vite, parce que le prompt de l'enfant est assemblé à partir du même registre par la même fonction.

Comme la doctrine n'est que du texte, elle fonctionne à l'identique que le backend soit Claude, GPT, Gemini, ou quelque chose que vous ajouterez plus tard.

Pour tout voir :

```
omega rules list
```

![omega rules list — les Lois et les Règles, affichées par OmegaOS](assets/omega-rules.svg)

## Architecture

Quatre niveaux, de haut en bas.

**Niveau 1 — Interface humaine.** La TUI, la CLI (plus de 40 commandes) et le pont Telegram pilotent tous la même couche en dessous.

**Niveau 2 — AISB Master.** Un agent persistant qui reste actif, redémarre tout seul s'il meurt, et reprend sa propre conversation avec `--continue`. Il embarque 14 templates d'agents nommés d'après des personnages de Matrix (Oracle, Morpheus, Seraph, Keymaker, Smith, Niobe, Architect, Merovingian, Neo, Zion, Link, Construct, Pythia, Council). Le Master est un répartiteur. Il se contente de classer le travail et de l'aiguiller vers les oracles.

**Niveau 3 — Oracle.** Un par projet. Il classe la demande, planifie, dépêche les workers, et passe le contrôle qualité à la fin. Un oracle orchestre. Il n'édite pas lui-même le code du projet.

**Niveau 4 — Workers.** Éphémères. Ils tournent en parallèle, et chacun est cadré à ses propres fichiers par une revendication de verrou : un seul rédacteur par fichier, garanti par des verrous de fichiers consultatifs (fs2). Le verrou est réel, ce n'est pas une convention. Un worker signale qu'il a fini en écrivant un `done.json` dont le statut vaut `done_clean`, `pending` ou `failed` ; sans ce statut, ce n'est pas terminé.

## Comment se déroule une mission

Une requête arrive par l'une de trois voies : la TUI, la CLI `omega`, ou le pont Telegram. Quel que soit son point de départ, elle atterrit sur l'AISB Master. Le Master la lit, la classe, et l'aiguille vers l'oracle responsable du projet concerné. Il ne touche à aucun fichier. Tout son rôle se résume à décider où part le travail.

L'Oracle prend ensuite le relais. Il est responsable d'un seul projet : il planifie la mission, la découpe en tâches, et dépêche un worker par tâche. Quand les workers rendent leur compte, il passe un contrôle qualité, puis fait remonter le rapport dans la chaîne. Ce qu'il ne fait jamais, c'est éditer le code du projet. Le correcteur et le rédacteur sont des agents distincts, si bien que la note n'est pas une auto-évaluation. Comme le correcteur n'a pas écrit le code, sa note est indépendante.

Les workers ont une durée de vie courte et s'exécutent en parallèle. Avant d'écrire dans un fichier, un worker se réserve ce fichier par un verrou consultatif (via `fs2`) : deux workers ne peuvent donc physiquement pas écrire le même fichier en même temps. On obtient ainsi un seul rédacteur par fichier, garanti par le verrou plutôt que par convention. Un worker accomplit sa tâche, confronte le résultat au comportement réel à l'exécution, et écrit un `done.json` portant le statut `done_clean`, `pending` ou `failed`. L'Oracle lit ce fichier, en accuse réception, et clôt la session. Tant que le statut n'est pas `done_clean`, la mission n'est pas terminée.

Un worker n'est pas obligé de mâcher ses sous-tâches une à une. Il peut exécuter un Workflow dans son propre processus : engendrer des sous-agents en parallèle, vérifier leurs sorties, et les fondre en une seule réponse. La revue de code en use, tout comme la recherche, les audits et le travail de design. C'est en général moins coûteux, et le résultat est meilleur que de dépêcher un nouveau worker pour chaque sous-tâche.

La vérification est délibérément contradictoire : un worker qui rapporte « terminé » ne clôt pas le contrôle ; son affirmation doit encore être vérifiée. Chaque affirmation passe par des agents indépendants, et ne survit que si une majorité (deux sur trois) s'accordent. Chaque constat est confronté aux autres agents avant d'être accepté. Les audits du Quality Arsenal se branchent précisément ici, au niveau du contrôle.

Tout cela repose sur l'entonnoir doctrinal décrit plus haut : chaque agent, à chaque niveau, reçoit les Lois et Règles cadrées à son rôle au moment même où il est dépêché. Un worker situé trois niveaux plus bas reçoit les mêmes règles L0–L5 que le Master.

Cette section du README en est elle-même un exemple. Elle est née d'un Workflow. Un agent en a rédigé le brouillon, des relecteurs indépendants l'ont passé au crible en traquant la prose générée par IA, un autre agent l'a révisé d'après leurs signalements, et des locuteurs natifs en ont assuré la traduction. Ainsi, aucune partie de ce texte n'est issue d'une seule passe non relue.

## Stack

C'est un workspace Rust avec trois crates :

- `omega-core` — l'orchestration, le registre de règles, le doctor, la timeline, le nettoyage, la patrouille, le verrouillage par portée de fichiers.
- `omega-cli` — le binaire `omega`, bâti sur `clap`.
- `omega-tui` — le gestionnaire de sessions, bâti sur `ratatui`.

En dessous, ça repose sur [rmux](https://github.com/agentik-os/rmux), un multiplexeur de terminal en Rust : un daemon, un SDK typé, et la gestion des PTY. rmux est une bibliothèque Rust typée, donc OmegaOS l'appelle directement au lieu de lancer tmux en sous-processus et d'en parser la sortie texte. Il n'y a aucune dépendance à tmux nulle part.

Bun et TypeScript se chargent du rendu des rapports PDF, via Next.js et Playwright. Bash n'apparaît qu'à un seul endroit : le bootstrap d'installation.

## Installation

Il vous faut une machine Linux et une toolchain Rust. L'installeur compile `rmux` et `omega` depuis les sources, donc le premier lancement n'est pas instantané.

```
git clone https://github.com/agentik-os/OmegaOS
cd OmegaOS
./install.sh
```

Une fois terminé, lancez le doctor.

## Utilisation

`omega doctor`, c'est la première chose à lancer, et celle qu'on relance dès que quelque chose cloche. Il contrôle toute la stack :

```
OmegaOS doctor

  [+] binary           omega 0.1.5
  [+] rmux daemon      connected, 6 live session(s)
  [+] rmux socket      /tmp/rmux-1000/default
  [+] doctrine         7 Laws + 47 Rules
  [+] agent CLI        codex available
  [+] state dir        /home/vibe/.omega/state
  [+] telegram service omega-tg-bot active
  [+] hooks            track + verify present, registered in settings.json
  [+] secrets dir      /home/vibe/.omega present
  [+] memory           249088MB available
  [+] codex auth       Codex login valid
  [+] telegram poller  1 poller
```

Les lignes `[!]` sont des avertissements, pas des erreurs — chacune embarque la commande de réparation, et `omega doctor --fix` répare les pannes mécaniques.

La surface de commandes, réduite à celles que vous utiliserez vraiment :

```
omega menu          Launch the TUI session manager
omega doctor        One-shot health check of the whole stack
omega rules list    List the Laws and Rules
omega dispatch      Dispatch a mission to an oracle
omega orchestrate   Run a full mission end-to-end (classify, plan, dispatch, monitor, gate)
omega spawn-worker  Spawn a worker under the current oracle
omega team          Spawn a team of agents in split panes
omega done          Signal task completion (called by workers)
omega timeline      Replay an oracle's dispatch-to-done history
omega resurrect     Re-spawn a crashed oracle from its persisted state
omega cleanup       Nuclear cleanup of stray sessions and stale state
omega backup        Back up the irreproducible ~/.omega state to a single tgz
omega telegram      Manage the Telegram bridge
omega pdf           Generate a PDF report
```

`omega menu` ouvre la TUI. Le daemon rmux possède chaque PTY, et les sessions portent un rôle : Master, Oracle, Worker, Home (vos propres shells interactifs) ou System (les daemons comme le pont Telegram). La TUI les liste avec leur progression en direct et permet de les tuer, de les verrouiller et de les renommer. Il y a un kill-all, un nettoyage radical pour quand l'état devient périmé, et le doctor intégré.

La boucle du quotidien est réduite. `omega orchestrate` mène une mission complète de bout en bout. `omega timeline` rejoue ce qu'a fait un oracle, dispatch par dispatch. Et quand un oracle plante, `omega resurrect` le ramène depuis son état persisté.

## Arsenal qualité

Il y a environ deux douzaines d'audits forensiques sous `skills/audits/` — `codeaudit`, `secaudit`, `perfaudit`, `a11yaudit`, `uiuxaudit`, `flowaudit`, `seoaudit`, `apiaudit`, et d'autres. Chacun déroule un protocole Gestalt-Popper : une porte de clarté en entrée, puis une falsification active — l'audit cherche à casser la chose plutôt qu'à la confirmer — puis une attention décuplée sur le point unique le plus important au lieu de la répartir uniformément. Quand un oracle termine une mission, il sélectionne tout seul les audits adaptés à ce qui vient de changer, comme ça vous n'avez pas à vous rappeler lesquels lancer.

## Limites

Autant que vous les sachiez avant de vous lancer.

- **Linux d'abord.** Développé sur un VPS sans tête. Pas de Windows. macOS n'est pas testé mais devrait globalement marcher, puisque ce n'est que du Rust et rmux.
- La TUI suppose un terminal 256 couleurs. Sur un terminal 16 couleurs, ce sera moche.
- Le runtime d'agent par défaut est OpenAI Codex. La CLI `codex` doit être connectée. Claude Code, Gemini, Pi, Hermes et GLM restent des alternatives explicites.
- **Une seule machine.** Le daemon rmux est local. Il n'y a pas d'orchestration multi-hôtes.
- C'est du 0.1.x. Je m'en sers tous les jours, mais vous tomberez sur des aspérités que je n'ai pas encore rencontrées.

## Remerciements

OmegaOS s'appuie sur le travail de beaucoup d'autres gens :

La plus grosse dette, c'est [rmux](https://github.com/agentik-os/rmux), le multiplexeur de terminal en Rust sur lequel tourne tout ce qui est ici.

Le reste de la stack Rust :

- [ratatui](https://github.com/ratatui/ratatui) et [crossterm](https://github.com/crossterm-rs/crossterm) — la TUI.
- [tokio](https://github.com/tokio-rs/tokio) — le runtime asynchrone.
- [clap](https://github.com/clap-rs/clap) et `clap_complete` — la CLI et les complétions de shell.
- [serde](https://github.com/serde-rs/serde) avec `serde_json`, `serde_yaml` et `toml` — la config et l'état.
- [anyhow](https://github.com/dtolnay/anyhow) et [thiserror](https://github.com/dtolnay/thiserror) — la gestion d'erreurs.
- `chrono` (horodatages), `dirs` (chemins), `fs2` (les verrous de fichiers consultatifs derrière les revendications de portée), `regex`, `tempfile`, `tracing` avec `tracing-subscriber` (logs), et `reqwest` (le HTTP pour Telegram et les PDF).

[Claude Code](https://www.anthropic.com) d'Anthropic est le runtime d'agent.

## Licence

Sous double licence, au choix [MIT](LICENSE-MIT) ou [Apache-2.0](LICENSE-APACHE), à votre convenance. Convention Rust classique. Prenez celle que vous préférez.
