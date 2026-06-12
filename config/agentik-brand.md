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
  linework (NEVER solid-dark, NEVER brunette), confident gaze, fitted black outfit, AI cofounder influencer".
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
