# R-TGDELIVER — Livrables (liens et fichiers) toujours poussés sur Telegram

**Kind:** Rule
**Category:** Reporting
**Added:** 2026-07-09

## Rule

Chaque fois qu'un livrable pour l'operateur est un LIEN (URL live, deploiement Vercel, artifact, dashboard, page publique, URL de telechargement) ou un FICHIER (PDF, ZIP, audio/mp3, image, rapport), le pousser AUSSI sur Telegram automatiquement, dans le meme tour, sans qu'il ait a le demander. L'operateur lit et tape ses liens depuis son telephone via Telegram : un lien ou un fichier laisse uniquement dans le terminal ou sur un store qu'il doit ouvrir a la main est un livrable rate. Envoyer via le bot Omega (`omega send`, ou l'API Bot `sendMessage` / `sendDocument`), UNIQUEMENT vers le chat allow-liste de l'operateur (R-TGSEC). Message court et propre : ce que c'est + l'URL tappable (disable_web_page_preview pour les listes). Pour un vrai fichier, envoyer un lien public (Vercel) ou tailnet, ou le fichier lui-meme via sendDocument s'il est petit. Ne PAS spammer les chemins de scratch internes ni les artefacts intermediaires : la regle vise les livrables user-facing. Un lien tailnet seul ne suffit pas si l'operateur n'a pas Tailscale sous la main : privilegier une URL publique quand c'est un livrable a consommer sur mobile.

## Origin

L'operateur consomme ses livrables depuis son telephone via Telegram. Des liens (dashboard Vercel, ZIP de PDF, echantillon audio) et des fichiers laisses seulement dans le terminal ou sur le tailnet (qui exige Tailscale) etaient rates ou penibles a atteindre. Il a demande explicitement que TOUT lien et TOUT fichier livrable atterrisse toujours sur Telegram, automatiquement.
