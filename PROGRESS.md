```markdown
# OmegaOS — PROGRESS

> Recap humain du projet. Source de vérité doctrine = `crates/omega-core/src/rules.rs` (compilé). État machine des agents = les `.json` dans `~/.omega/state` (hors repo, gitignored).
> **Dernière synchro : 2026-07-02**

## État d'un coup d'œil
OmegaOS est un *agentic terminal OS* : plan de contrôle Rust pour piloter une flotte d'agents de code IA en parallèle sous un rulebook typé compilé dans le binaire. Workspace Rust (3 crates), TUI `ratatui`, bridge Telegram, tourne sur `rmux` (pas de tmux). Version workspace `0.1.6` (npm `omega-os@1.5.4`), branche `main`.

Aujourd'hui : ajout de la règle **R-MODEL** (bon modèle + reasoning-effort pour la tâche) dans la doctrine SSOT, propagée dans `doctor.rs` / `rules.rs` / `oracle.md` / agent AISB, puis audit `/codeaudit` archivé (verdict **98/100 PASS**). Deux dossiers marketing apparaissent en untracked (`marketing/`, `tools/marketing-machine/`).

## Tableau d'état

| Domaine | Statut | Note |
|---|---|---|
| Stack Rust (3 crates) | ✅ stable | `omega-core` / `omega-cli` / `omega-tui`, edition 2021, resolver 2 |
| Doctrine typée (Lois + Règles) | ✅ actif | R-MODEL ajoutée aujourd'hui → **voir Dérive** (RULES.md dit encore 26 Règles) |
| Doctor / registre de règles | ✅ à jour | `doctor.rs` propage R-MODEL |
| Install parity (LAW 0) | ⚠️ à vérifier | relancer `./scripts/verify-install.sh` après R-MODEL |
| Infra rmux (pin `4455da0`) | ✅ | aucune dépendance tmux |
| Audits | ✅ | verdict R-MODEL archivé `agentic/audits/codeaudit-r-model-2095d9f.md` |
| Marketing / tooling | 🟡 untracked | `marketing/`, `tools/marketing-machine/` non commités |
| Git | 🟡 arbre sale | 2 dossiers untracked à trier/commiter ou ignorer |

## Fait récemment (git)
| Commit | Résumé |
|---|---|
| `31f5800` | audit : archive du verdict `/codeaudit` pour R-MODEL (commit `2095d9f`) — **98/100 PASS** |
| `2095d9f` | rules : ajout **R-MODEL** — bon modèle & reasoning-effort selon la tâche (`rules.rs`, `doctor.rs`, `oracle.md`, CLAUDE.md AISB, nouveau `rules/R-MODEL-*.md`) |

Untracked (non commités) : `marketing/`, `tools/marketing-machine/`.

## Prochaines étapes
- [ ] Mettre à jour **RULES.md** : le SSOT compte désormais **27 Règles** avec R-MODEL, RULES.md dit encore « 26 Règles » (voir Dérive).
- [ ] Lancer `./scripts/verify-install.sh` pour confirmer LAW 0 (install parity) après l'ajout de R-MODEL.
- [ ] Décider du sort de `marketing/` et `tools/marketing-machine/` : commiter, ranger, ou `.gitignore` — ne pas laisser l'arbre sale.
- [ ] Vérifier qu'aucun secret ne traîne dans les nouveaux dossiers marketing avant tout commit.
- [ ] Repasser un `omega rules list` pour confirmer l'affichage courant de l'ensemble (27 règles).

## Journal
- 2026-07-02 — Ajout de la règle **R-MODEL** (bon modèle + reasoning-effort pour la tâche) dans la doctrine typée et propagation (doctor/rules/oracle/AISB) ; audit `/codeaudit` du commit associé archivé avec verdict **98/100 PASS**. Apparition de deux dossiers marketing untracked.

## ⚠️ Dérive vs RULES
- **Compte de règles** : la SSOT `crates/omega-core/src/rules.rs` embarque désormais **R-MODEL** en plus (commit `2095d9f`), portant l'ensemble à **27 Règles nommées**. `RULES.md` (section *Identité* et *Intouchable*) affirme encore « 6 Lois L0–L5 + **26** Règles ». → Mettre RULES.md à jour (26 → 27) pour rester synchronisée avec le binaire compilé et `~/.omega/state/oracle-OmegaOS*`.

---
> Note : l'état machine des agents reste les fichiers `.json` dans `~/.omega/state` (hors repo, jamais commité).
```
