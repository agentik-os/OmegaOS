# OmegaOS — État d'avancement (recap)

> Recap **humain**, dérivé de `git log` + `Cargo.toml` + `CHANGELOG.md` + `~/.omega/state/oracle-OmegaOS*`.
> L'**état machine** que lisent les agents reste les `.json` dans `~/.omega/state/` — ce fichier ne les remplace pas, il les rend lisibles. Régénérable.
> Dernière synchro : 2026-06-07

## En un coup d'œil
- **Phase** : pre-1.0, tournant en daily-use. Version workspace `0.1.5`, package npm `omega-os@1.5.3` publié (tarball registry vérifié byte-identique au tree testé).
- **Repo** : `github.com/agentik-os/OmegaOS` @ `c1e0c75` (branche `main`) — arbre **propre** (`git status --porcelain` = 0), en sync avec `origin/main`.
- **Dernières missions oracle** : campagnes de fix successives (fix5 → fix8) côté TUI/CLI/installer/tg-bot, plus la mission « theme doctrine » (WCAG-AA dans le moteur de thèmes). Toutes `done_clean`, install-parity vérifiée.

## Crates / composants — état
| Composant | Rôle | État |
|---|---|---|
| `omega-core` | orchestration, rules registry, doctor, patrol, file-scope locking (32 modules) | actif, suite de tests verte (~347 tests aux derniers runs) |
| `omega-cli` | binaire `omega`, 40+ commandes (`clap`) | actif |
| `omega-tui` | session manager `ratatui`, 7 onglets, 17 thèmes | actif, contrat de contraste WCAG-AA test-enforced |
| Doctrine (`rules.rs`) | 6 Lois + 20 Règles typées, injectées par `agent_context_block(scope)` | stable, SSOT compilé |
| Quality Arsenal | ~20–23 audits Gestalt-Popper, gate adversarial 2-of-3 | actif, auto-sélection par oracle |
| Bridge Telegram | tg-bot (systemd Linux / launchd macOS), wizard d'install | actif ; setup canonique via env `OMEGA_TG_TOKEN` |
| `install.sh` / `verify-install.sh` | bootstrap from-source + gate install-parity | passe (INSTALL PARITY OK) |
| rmux | infra multiplexeur (pin `rev 4455da0`) | dépendance git, fix paste 10k intégré |

## Fait récemment (git)
- `c1e0c75` — fix8-echoes : derniers échos runtime enseignent la forme env-prefix du setup Telegram.
- `e6b5bb0` — doctor ne false-warn plus « duplicate pollers » sur Mac headless.
- `f81db04` / `ae64483` — `OMEGA_TG_TOKEN` (env) devient la commande canonique de setup Telegram, token jamais en argv.
- `9443f8a` — agent-bot systemd units sans dépendance login-shell.
- `aa7e8ca` — release v0.1.5 (fix rmux paste 10k via PasteFilter stateful).
- Antérieur : campagnes fix6/fix7 (15 findings `/code-review all xhigh` → 6 workers chacune), moteur de thèmes WCAG-AA + `docs/THEMES.md`, CI GitHub Actions `-D warnings`, README multilingue (fr/ru/zh) + docs contributeur.

## Prochaines étapes
- Résidus acceptés (INFO/LOW, backlog mémoire `fix*-residual`) : `omega-tg-up` garde son contrat token-positionnel (commande distincte), une mention « legacy » labellisée dans README, spawns bash internes au bot.
- Limites non levées (host headless) : live-verify terminal kitty-protocol (Alt+Esc), arm live du delete projet (couverts par tests unitaires uniquement) — à valider sur un host adéquat.
- Marche vers 1.0 (semver dès 1.0 ; `main` seule ligne supportée d'ici là). Mainteneur humain : à préciser.

## Docs clés
- Doctrine / instructions agents : `OMEGA.md` + `~/.omega/rules/` + `crates/omega-core/src/rules.rs` (SSOT)
- Produit : `README.md` (+ `README.fr.md` / `.ru.md` / `.zh.md`)
- Architecture : `docs/ARCHITECTURE.md`, `docs/GETTING-STARTED.md`, `MAP.md`
- Install / sécurité : `install.sh`, `scripts/verify-install.sh`, `SECURITY.md`, `CLAUDE.md`
- Historique : `CHANGELOG.md`
- Règles repo : `RULES.md`
