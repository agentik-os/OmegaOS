# AGENTIK — BRAND SSOT (visuel · voix · musique)

> LE standard de marque Agentik. S'applique à TOUT : Nova (studio, NOVA OS,
> projet Nova), la marketing machine, et les visuels de Gareth. Validé 2026-06-12.
> Source vivante : ~/Station/Nova/presence/avatar/BRANDING-AGENTIK.md (réfs + exemples).

## VISUEL — prompt maître (remplacer <SCENE>)
```
High-contrast black and white comic illustration, BOLD graphic Sin City style — strong solid
blacks, clean bright whites, dramatic striking contrast and silhouette, crisp confident ink with
light halftone, BUT clean and graphic, never muddy or overworked. Modern minimalist setting —
sleek contemporary concrete-and-glass architecture OR a clean natural landscape — with strong
negative space and a FEW characterful details. <SUBJECT> — <SCENE>. Bold graphic editorial shot,
monochrome, striking, premium. NOT muddy grey, NOT washed-out, NOT cluttered, NEVER two people.
```
- **Pour Nova** : `<SUBJECT>` = "ONE single woman only — Nova: long flowing BLONDE hair in light/white
  linework (NEVER solid-dark, NEVER brunette), confident gaze, fitted black outfit, Agentik cofounder, born in the binary matrix".
  Réf image obligatoire : `~/Station/Nova/presence/avatar/nova-face-3angles.jpg` (visage) ou
  `nova-fullbody-turnaround.jpg` (corps). RÈGLES DURES : Nova BLONDE toujours · UNE seule Nova.
- **Pour Gareth / marque générique** : `<SUBJECT>` = le sujet voulu, même style.
- Outil : Higgsfield CLI `higgsfield generate create nano_banana_2 --prompt "..." --image <ref> --wait`
  (plan ultra ; PAS l'API directe). Réf d'or validée : `nova-nature-hill.png`.

## VOIX (vidéos, voix off)
La voix de Nova = ElevenLabs, voice_id `WeAAwKYcS06VmXw086yZ` (« Nova »), via omega-ttsd ou
`POST api.elevenlabs.io/v1/text-to-speech/<id>`.

## MUSIQUE
ElevenLabs Music `POST /v1/music {prompt, music_length_ms}` (clé au coffre). Style maison :
**darkwave / synthwave / électro années 90**, sombre, analogique, cinématique.

## VIDÉO
Higgsfield `cinematic_studio_video --start-image <img>` (anime une image) → `ffmpeg` (musique) →
Remotion `~/Station/Nova/video/` (montage 1-min multi-plans + voix off Nova).

## ⚠️ RÈGLE VIDÉO — pas de lèvres non-synchro (opérateur 2026-06-12)
On ne montre JAMAIS Nova en train de mouther des mots si les lèvres ne sont pas synchro avec la
voix. Par défaut : Nova fait des ACTIONS / b-roll (bouche fermée), la voix off ElevenLabs passe
PAR-DESSUS. Pour un vrai talking-head lip-syncé, il faut HeyGen (clé HEYGEN_API_KEY actuellement
VIDE — à fournir) ou un modèle audio-driven ; tant qu'on ne l'a pas, prompt « mouth closed, does
NOT talk ». Réf : reel (actions+VO) = bon ; UGC v1 (parlait) = corrigé en bouche fermée.


## 🎛️ STACK MODÈLES (non-Google — validé 2026-06-12)
- **Image** : `gpt_image_2` (OpenAI GPT Image 2) — cinématographique, premium, tient le perso + le blond. (nano_banana = Google → évité.)
- **Vidéo** : `seedance_2_0` (Seedance 2.0) en défaut, `kling3_0` (Kling 3.0) en alternative — vrai mouvement cinéma. **JAMAIS Veo/Google.** L'audio Veo/Seedance est strippé (on garde la voix Nova + musique).
- **Lip-sync (talking-head)** : HeyGen ✅ TESTÉ — upload talking_photo + /v2/video/generate (clé au coffre + Composio ACTIVE) — pour quand Nova doit VRAIMENT parler face caméra, lèvres synchro. Sinon : action + voix off, bouche fermée.
- **Voix** : ElevenLabs `PB6BdkFkZLbI39GHdnbQ` (Nova, EN native London/Paris, FR aussi). **Musique** : ElevenLabs Music darkwave/synthwave 90s.
- Viser le CINÉMATOGRAPHIQUE : lumière dramatique, cadrage soigné, "award-winning", beau, premium.

## 🎛️ Catalogue complet des modèles
Toutes les capacités image/vidéo/audio/lip-sync/3D pilotables : `~/.omega/branding/HIGGSFIELD-CAPABILITIES.md`. Le studio Nova, la marketing machine et NOVA OS peuvent TOUT utiliser (image gpt_image_2/seedream/flux/recraft, vidéo seedance/kling/wan, lip-sync omnihuman, upscale topaz, virality predictor brain_activity…). Non-Google, cinématographique.
