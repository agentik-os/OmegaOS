# OmegaOS — État d'avancement (recap)

> Recap **humain**, dérivé de `git log` + `Cargo.toml` + `CHANGELOG.md` + `~/.omega/state/oracle-OmegaOS*`.
> L'**état machine** que lisent les agents reste les `.json` dans `~/.omega/state/` — ce fichier ne les remplace pas, il les rend lisibles. Régénérable.
> Dernière synchro : 2026-06-07

## En un coup d'œil
- **Phase** : pre-1.0, en daily-use. Version workspace `0.1.5`, package npm `omega-os@1.5.3` (inchangées aujourd'hui — pas de commit de release).
- **Repo** : `github.com/agentik-os/OmegaOS` @ `e53f1e5` (branche `main`) — arbre de travail **NON propre** : WIP « inbox-bot » non commité (`scripts/inbox-bot-up.sh`, `telegram-bot/inbox-bot.ts` non suivis) + `scripts/verify-install.sh` modifié.
- **Travail du jour** : vague « atlas » côté bridge Telegram (hub de nettoyage + purge RAM) et trois skills de maintenance (`cleanup`, `project-tidy`, `ramflush`) wirés dans `install.sh` ; recompte de l'arsenal d'audits 16→23 ; gros chantier de rangement docs/ + agentic/.

## Crates / composants — état
| Composant | Rôle | État |
|---|---|---|
| `omega-core` | orchestration, rules registry, doctor, patrol, file-scope locking (32 modules) | actif, suite de tests verte (~347 tests aux derniers runs) |
| `omega-cli` | binaire `omega`, 40+ commandes (`clap`) | actif |
| `omega-tui` | session manager `ratatui`, 7 onglets, 17 thèmes | actif, contrat de contraste WCAG-AA test-enforced |
| Doctrine (`rules.rs`) | 6 Lois + 21 Règles typées (ajout **R-SKILLPUB** : tout nouveau skill est publié à la librairie), injectées par `agent_context_block(scope)` | stable, SSOT compilé |
| Quality Arsenal | **23** audits Gestalt-Popper (recompté 16→23), gate adversarial 2-of-3, invariant #9 « no performative phases » | actif, auto-sélection par oracle |
| Bridge Telegram | tg-bot (systemd Linux / launchd macOS), wizard d'install, **hub de nettoyage « atlas » (Clean)** : purge RAM via helper root, Kill-all déplacé dans le hub | actif ; setup canonique via env `OMEGA_TG_TOKEN` |
| Skills maintenance | `cleanup`, `project-tidy`, `ramflush` (SKILL.md + scripts) | actifs, wirés dans `install.sh`, publiés à la librairie (R-SKILLPUB) |
| `install.sh` / `verify-install.sh` | bootstrap from-source + gate install-parity | à revérifier (verify-install.sh modifié, non commité) |
| rmux | infra multiplexeur (pin `rev 4455da0`) | dépendance git, fix paste 10k intégré |

## Fait récemment (git)
- `e53f1e5` — fix(atlas) : la purge RAM libère réellement la RAM via un helper root (bun ne peut pas exécuter sudo).
- `c5cd9eb` — feat(skills) : ajout `ramflush` (purge RAM) + branchement dans le hub Clean d'atlas + règle **R-SKILLPUB**.
- `4e4e8ec` — fix(atlas) : « Kill all » retiré du menu principal (vit désormais dans le hub Clean).
- `d4c67f3` — feat(atlas) : hub de nettoyage dans le bot de commande Telegram.
- `c022301` — docs(audits) : arsenal recompté 16→23 + ajout de l'invariant #9 (pas de phases performatives).
- `01d00f6` — chore(tidy+docs) : rangement docs/ + agentic/, fix docs périmées, suppression de dotfolders parasites ; `MAP.md`→`docs/MAP.md`, `RESET-RECOVERY.md`→`docs/`.
- `1e5ce51` — feat(skills) : ajout des skills de maintenance `cleanup` + `project-tidy` + wiring install.

## Prochaines étapes
- **Finir le WIP inbox-bot** non commité (`telegram-bot/inbox-bot.ts`, `scripts/inbox-bot-up.sh`) + statuer sur `scripts/verify-install.sh` modifié, puis commit propre.
- **Re-passer le gate** : `./scripts/verify-install.sh` doit repasser (install-parity, git clean, remote en sync) avant tout « done » — l'arbre est actuellement sale.
- Mettre `RULES.md` en cohérence avec le code (voir « Dérive vs RULES » : compte de règles 20→21, bash hors bootstrap).
- Résidus acceptés (INFO/LOW, backlog `fix*-residual`) : `omega-tg-up` garde son contrat token-positionnel, mention « legacy » labellisée dans README, spawns bash internes au bot.
- Limites non levées (host headless) : live-verify terminal kitty-protocol (Alt+Esc), arm live du delete projet — à valider sur un host adéquat.
- Marche vers 1.0 (semver dès 1.0 ; `main` seule ligne supportée d'ici là). Mainteneur humain : à préciser.

## ⚠️ Dérive vs RULES
- **Compte de règles** : `RULES.md` (Identité) annonce « 6 Lois + **20** Règles compilées dans le binaire », mais le commit `c5cd9eb` ajoute **R-SKILLPUB** à `crates/omega-core/src/rules.rs` → le SSOT compile désormais **21** Règles. `RULES.md` n'a pas été mis à jour.
- **Bash hors bootstrap** : `RULES.md` (Stack) stipule « Bash : à un seul endroit, le bootstrap d'install. » Or le jour a ajouté plusieurs scripts bash hors install : `skills/ramflush/scripts/ram-flush.sh`, `skills/cleanup/scripts/*.sh` (8 scripts), `skills/project-tidy/scripts/tidy-*.sh`, plus le WIP `scripts/inbox-bot-up.sh`. La règle « un seul endroit » doit soit être amendée (exception « scripts de skills »), soit le placement revu.

## Docs clés
- Doctrine / instructions agents : `OMEGA.md` + `~/.omega/rules/` + `crates/omega-core/src/rules.rs` (SSOT)
- Produit : `README.md` (+ `README.fr.md` / `.ru.md` / `.zh.md`)
- Architecture : `docs/ARCHITECTURE.md`, `docs/GETTING-STARTED.md`, `docs/MAP.md`
- Install / sécurité : `install.sh`, `scripts/verify-install.sh`, `SECURITY.md`, `CLAUDE.md`
- Historique : `CHANGELOG.md`
- Règles repo : `RULES.md`

## Journal
- 2026-06-07 — Vague « atlas » : hub de nettoyage dans le bot Telegram (purge RAM via helper root, Kill-all déplacé dans le hub) + trois skills de maintenance (`cleanup`, `project-tidy`, `ramflush`) wirés dans `install.sh` et règle R-SKILLPUB ajoutée. Recompte de l'arsenal d'audits (16→23, invariant #9) et gros rangement docs/+agentic/. WIP inbox-bot encore non commité.

> Note : l'**état machine** que lisent les agents reste les fichiers `.json` dans `~/.omega/state/` ; ce PROGRESS.md ne les remplace pas, il les rend lisibles.
