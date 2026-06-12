#!/usr/bin/env bun
/**
 * OmegaOS Deposit bot — a private inbox so the operator can send photos/notes
 * from their phone that an agent (Claude in a terminal session) can READ.
 *
 * Every photo is downloaded to $OMEGA_DIR/inbox/ (timestamped), captions are
 * saved as sidecar .txt and indexed in inbox/index.jsonl. The bot is locked to
 * ONE chat (R-TGSEC): either the chat_id written into deposit.toml at setup, or
 * — when chat_id is 0/unset — the first chat whose message contains the one-time
 * pair_code printed by inbox-bot-up (a blind first-sender lock would let anyone
 * who finds the bot's username own the operator's inbox, an agent-readable
 * prompt-injection path). Everything from any other chat is dropped + logged.
 *
 * Config: $OMEGA_DIR/deposit.toml  (mirrors telegram.toml)
 *     bot_token = "123:ABC"
 *     chat_id   = 0            # 0 = lock via pair_code
 *     pair_code = "123456"     # one-time pairing code (only when chat_id is 0)
 *     enabled   = true
 * Token may also come from $OMEGA_DEPOSIT_TOKEN / $INBOX_BOT_TOKEN (env wins
 * over the file so the systemd/launchd unit can inject it without touching disk).
 *
 * Setup:  omega-inbox-bot-up <BOT_TOKEN> [YOUR_TELEGRAM_USER_ID]
 */
import { existsSync, readFileSync, writeFileSync, mkdirSync, appendFileSync, renameSync } from "fs";
import { homedir } from "os";

const OMEGA_DIR = process.env.OMEGA_DIR || `${homedir()}/.omega`;
const CONF = `${OMEGA_DIR}/deposit.toml`;
const STATE = `${OMEGA_DIR}/inbox-bot/state.json`;
const MEDIA = `${OMEGA_DIR}/inbox`;          // photos land here — the agent reads this dir
const INDEX = `${MEDIA}/index.jsonl`;

// Minimal TOML scalar reader (same approach the Rust side uses for telegram.toml):
// we only need bot_token / chat_id / pair_code / enabled, so a line regex beats a TOML dep.
function readConf(): { bot_token?: string; chat_id?: number; pair_code?: string; enabled?: boolean } {
  try {
    const t = readFileSync(CONF, "utf8");
    const tok = t.match(/^\s*bot_token\s*=\s*"([^"]*)"/m)?.[1];
    const cid = t.match(/^\s*chat_id\s*=\s*(-?\d+)/m)?.[1];
    const pc = t.match(/^\s*pair_code\s*=\s*"([^"]*)"/m)?.[1];
    const en = t.match(/^\s*enabled\s*=\s*(true|false)/m)?.[1];
    return { bot_token: tok, chat_id: cid != null ? Number(cid) : undefined, pair_code: pc, enabled: en ? en === "true" : undefined };
  } catch { return {}; }
}

const conf = readConf();
const TOKEN = process.env.OMEGA_DEPOSIT_TOKEN || process.env.INBOX_BOT_TOKEN || conf.bot_token || "";
// Declarations BEFORE the launch branch: main() runs synchronously from the
// else-arm below, and a `const` declared after it is in the temporal dead
// zone — tg()'s `${API}` would throw, be swallowed by its try/catch, and
// masquerade as "token rejected by Telegram".
const API = `https://api.telegram.org/bot${TOKEN}`;
const FILE_API = `https://api.telegram.org/file/bot${TOKEN}`;
mkdirSync(MEDIA, { recursive: true });
mkdirSync(`${OMEGA_DIR}/inbox-bot`, { recursive: true });
if (!TOKEN) {
  console.log(`[deposit] no token yet — waiting. Connect: omega-inbox-bot-up <BOT_TOKEN> [USER_ID]  (writes ${CONF})`);
  // Idle instead of crash-looping under systemd Restart=always until a token exists.
  setInterval(() => {}, 1 << 30);
} else if (conf.enabled === false) {
  console.log("[deposit] disabled (enabled=false in deposit.toml) — idling.");
  setInterval(() => {}, 1 << 30);
} else {
  main().catch((e) => { console.error("deposit fatal:", e?.message || e); process.exit(1); });
}

type State = { chat_id?: number };
const loadState = (): State => { try { return JSON.parse(readFileSync(STATE, "utf8")); } catch { return {}; } };
const saveState = (s: State) => { try { writeFileSync(STATE, JSON.stringify(s, null, 2)); } catch {} };
// Allow-list: explicit chat_id from deposit.toml wins; else the persisted self-lock.
let allowed: number | undefined = conf.chat_id && conf.chat_id !== 0 ? conf.chat_id : loadState().chat_id;

async function tg(method: string, body: any): Promise<any> {
  try {
    const r = await fetch(`${API}/${method}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(body) });
    return await r.json();
  } catch (e) { console.error(`tg ${method}:`, e); return { ok: false }; }
}

function stamp(): string {
  const d = new Date();
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}_${p(d.getHours())}${p(d.getMinutes())}${p(d.getSeconds())}`;
}

async function download(fileId: string, ext: string): Promise<string | null> {
  const f = await tg("getFile", { file_id: fileId });
  if (!f.ok) return null;
  const res = await fetch(`${FILE_API}/${f.result.file_path}`);
  if (!res.ok) return null;
  const out = `${MEDIA}/${stamp()}_${fileId.slice(-6)}.${ext}`;
  writeFileSync(out, Buffer.from(await res.arrayBuffer()));
  return out;
}

const note = (rec: any) => { try { appendFileSync(INDEX, JSON.stringify(rec) + "\n"); } catch {} };

async function handle(msg: any) {
  const chat = msg.chat?.id as number;
  if (allowed == null) {
    // NEVER blind-lock to the first sender: anyone who discovers the bot's username
    // before the operator's first DM would own the inbox — and agents Read + act on
    // ~/.omega/inbox/, so that is a prompt-injection path (R-TGSEC). Pair only when
    // the message carries the one-time code printed by inbox-bot-up.
    const code = (conf.pair_code || "").trim();
    const text = (msg.caption || msg.text || "").trim();
    if (!code || !text.includes(code)) {
      console.error(`[deposit] UNPAIRED message from chat ${chat} (${msg.chat?.username || msg.chat?.first_name || "?"}) DROPPED — ${code ? "pairing code mismatch" : "no chat_id/pair_code in deposit.toml; rerun omega-inbox-bot-up"}`);
      return;
    }
    allowed = chat; saveState({ chat_id: chat });
    console.log(`[deposit] paired — locked to chat ${chat} (${msg.chat?.first_name || msg.chat?.username || "?"})`);
    await tg("sendMessage", { chat_id: chat, text: "✅ Appairé — dépôt verrouillé sur ce chat. Envoie photos / notes — l'agent les lit dans ~/.omega/inbox/." });
    return; // the pairing message itself is not a note
  }
  if (chat !== allowed) { console.log(`[deposit] drop message from non-allowed chat ${chat}`); return; }

  const caption = (msg.caption || msg.text || "").trim();

  if (Array.isArray(msg.photo) && msg.photo.length) {
    const p = await download(msg.photo[msg.photo.length - 1].file_id, "jpg");
    if (p) {
      if (caption) writeFileSync(p.replace(/\.jpg$/, ".txt"), caption);
      note({ ts: stamp(), kind: "photo", path: p, caption });
      console.log(`[deposit] photo ${p}${caption ? ` — "${caption}"` : ""}`);
      await tg("sendMessage", { chat_id: chat, text: `📥 reçu : ${p.split("/").pop()}${caption ? `\n📝 ${caption}` : ""}` });
    } else await tg("sendMessage", { chat_id: chat, text: "⚠️ échec du téléchargement, réessaie." });
    return;
  }

  if (msg.document) {
    // EVERY document type — this is a deposit box. The old image-only mime
    // filter SILENTLY dropped anything else (the operator's App Store .p8
    // key vanished without even an error reply). Keep the original filename
    // (sanitized) so keys/configs stay recognizable; the agent sorts them
    // from ~/.omega/inbox/ into their final home.
    const orig = (msg.document.file_name || "document").replace(/[^A-Za-z0-9._-]/g, "_").slice(0, 80);
    const ext = orig.includes(".") ? orig.split(".").pop()!.toLowerCase() : "bin";
    const p = await download(msg.document.file_id, ext);
    if (p) {
      const named = p.replace(/[^/]+$/, `${stamp()}_${orig}`);
      try { renameSync(p, named); } catch {}
      const fin = existsSync(named) ? named : p;
      if (caption) writeFileSync(`${fin}.txt`, caption);
      note({ ts: stamp(), kind: "document", path: fin, caption });
      console.log(`[deposit] doc ${fin}`);
      await tg("sendMessage", { chat_id: chat, text: `📥 reçu : ${fin.split("/").pop()}${caption ? `\n📝 ${caption}` : ""}` });
    } else await tg("sendMessage", { chat_id: chat, text: "⚠️ échec du téléchargement, réessaie." });
    return;
  }

  if (caption) {
    const p = `${MEDIA}/${stamp()}_note.txt`;
    writeFileSync(p, caption);
    note({ ts: stamp(), kind: "note", path: p, caption });
    console.log(`[deposit] note: ${caption}`);
    await tg("sendMessage", { chat_id: chat, text: "📝 note enregistrée." });
  }
}

async function main() {
  const me = await tg("getMe", {});
  if (!me.ok) { console.error("[deposit] token rejected by Telegram (getMe). Fix it: omega-inbox-bot-up <TOKEN>"); process.exit(1); }
  console.log(`[deposit] @${me.result?.username} — inbox=${MEDIA} — locked=${allowed ?? "(awaiting pairing code)"}`);
  let offset = 0;
  for (;;) {
    const r = await tg("getUpdates", { offset, timeout: 50, allowed_updates: ["message"] });
    if (r.ok && Array.isArray(r.result)) {
      for (const u of r.result) {
        offset = u.update_id + 1;
        if (u.message) { try { await handle(u.message); } catch (e: any) { console.error("handle:", e?.message || e); } }
      }
    } else if (!r.ok) { await Bun.sleep(2000); }
  }
}
