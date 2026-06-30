#!/usr/bin/env bun
/**
 * OmegaOS Telegram Command Bot — button-driven phone control center.
 * ───────────────────────────────────────────────────────────────────────────
 * Dependency-free (Bun + raw Telegram Bot API). Every command opens an inline
 * keyboard of sub-actions; each button runs an `omega` CLI action on the host.
 * Group/forum mode: /setupgroup registers a supergroup (verifies the bot is
 * admin); /sync maps each project to a forum topic and routes topic messages to
 * that project's oracle. /dashboard sends the Mission Control link.
 * Single poller per bot token. config ← ~/.omega/telegram.toml.
 */
import { $ } from "bun";
import { readFileSync, writeFileSync, existsSync, statSync } from "node:fs";
import { homedir } from "node:os";

const OMEGA_DIR = process.env.OMEGA_DIR || `${homedir()}/.omega`;
const TG_TOML = `${OMEGA_DIR}/telegram.toml`;
const MC_ENV = `${OMEGA_DIR}/repos/omega-mc/.env`;
const GROUPS_FILE = `${OMEGA_DIR}/telegram-groups.json`;
const OMEGA = process.env.OMEGA_BIN || `${homedir()}/.local/bin/omega`;

// ── Conversation history: persisted per chat+topic so Atlas, the project oracles,
// and the agent-bots all have FULL access to the running conversation (not stateless
// per-message). Stored as JSONL in ~/.omega/state/tg-history/ and mirrored to the
// OmegaMC dashboard (mcMirror) so the dashboard stays in sync with Telegram.
const HIST_DIR = `${OMEGA_DIR}/state/tg-history`;
const histKey = (chat: number, thread?: number) => `${chat}${thread ? `-t${thread}` : ""}`;
const histPath = (chat: number, thread?: number) => `${HIST_DIR}/${histKey(chat, thread)}.jsonl`;
function histAppend(chat: number, thread: number | undefined, role: "operator" | "assistant", text: string, project?: string) {
  try {
    Bun.spawnSync(["mkdir", "-p", HIST_DIR]);
    const line = JSON.stringify({ ts: new Date().toISOString(), role, text: String(text).slice(0, 8000) }) + "\n";
    const p = histPath(chat, thread);
    writeFileSync(p, (existsSync(p) ? readFileSync(p, "utf8") : "") + line);
    mcMirror(project || "atlas", role, text).catch(() => {});
  } catch {}
}
// Last N turns as a plain transcript, to prepend to a brain/dispatch prompt.
function histContext(chat: number, thread?: number, n = 12): string {
  try {
    const lines = readFileSync(histPath(chat, thread), "utf8").trim().split("\n").filter(Boolean);
    const turns = lines.slice(-n).map(l => { try { const o = JSON.parse(l); return `${o.role === "operator" ? "Operator" : "You"}: ${o.text}`; } catch { return ""; } }).filter(Boolean);
    return turns.length ? `## Recent history of this conversation (for context)\n${turns.join("\n")}\n\n` : "";
  } catch { return ""; }
}
// Mirror a turn into the OmegaMC dashboard store (best-effort) so the dashboard's
// per-agent conversation stays in sync with Telegram. Auto-disables after a failure
// (e.g. the MC build has no message-ingest endpoint yet) so it never spams.
let MC_MIRROR_OK = true;
async function mcMirror(agent: string, role: string, text: string) {
  if (!MC_MIRROR_OK) return;
  try {
    const pw = MC_PW; if (!pw) { MC_MIRROR_OK = false; return; }
    // Atlas's dashboard agent is "director" (there is no "atlas" agent in MC).
    const id = (agent.toLowerCase() === "atlas" ? "director" : agent.toLowerCase()).replace(/[^a-z0-9_-]/g, "-");
    const r = await fetch(`http://localhost:8080/api/agents/definitions/${id}/messages`, {
      method: "POST",
      headers: { "content-type": "application/json", authorization: "Basic " + Buffer.from(":" + pw).toString("base64") },
      body: JSON.stringify({ role: role === "operator" ? "user" : "assistant", content: String(text).slice(0, 8000), source: "telegram" }),
    });
    if (!r.ok) MC_MIRROR_OK = false; // endpoint absent (GET-only build) → stop trying
  } catch { MC_MIRROR_OK = false; }
}

function readKV(path: string, re: RegExp): Record<string, string> {
  const out: Record<string, string> = {};
  try { for (const l of readFileSync(path, "utf8").split("\n")) { const m = l.match(re); if (m) out[m[1]] = m[2].replace(/^"|"$/g, ""); } } catch {}
  return out;
}
// Voice input: an OpenAI key (provisioning/services.env or the MC .env) enables
// Whisper transcription so you can TALK to Atlas. Empty → voice messages are ignored.
const OPENAI_KEY =
  readKV(`${OMEGA_DIR}/provisioning/services.env`, /^\s*export\s+([A-Z_]+)\s*=\s*"?([^"]*)"?\s*$/).OPENAI_API_KEY ||
  readKV(MC_ENV, /^([A-Z_]+)=(.*)$/).OPENAI_API_KEY || "";
async function transcribeVoice(fileId: string): Promise<string> {
  if (!OPENAI_KEY) return "";
  try {
    const gf = await tg("getFile", { file_id: fileId });
    const fp = gf?.result?.file_path; if (!fp) return "";
    const audio = await (await fetch(`https://api.telegram.org/file/bot${TOKEN}/${fp}`)).arrayBuffer();
    const fd = new FormData();
    fd.append("file", new Blob([audio]), "voice.ogg");
    fd.append("model", "whisper-1");
    const r = await fetch("https://api.openai.com/v1/audio/transcriptions", { method: "POST", headers: { authorization: `Bearer ${OPENAI_KEY}` }, body: fd });
    const j: any = await r.json();
    return (j?.text || "").trim();
  } catch { return ""; }
}
// File input: ANY attachment — a photo, a document of any type (PDF, .txt, .csv,
// code, zip…), a video, or an audio file — is downloaded to ${OMEGA_DIR}/state/tg-media/
// so the dispatched oracle / Atlas can open it with the Read tool. (The oracle runs
// --dangerously-skip-permissions, so it can read this path whatever its --add-dir is.)
// Returns the local path, or "" when the message carries no file / the download failed.
// Without this, a file+caption message has no `.text` and was silently DROPPED by both
// poll loops (the operator's mission never reached any oracle) — and before, even a
// document WAS dropped unless its mime was image/* (the gap the operator hit).
// NOTE: Telegram's Bot API caps getFile downloads at 20 MB — larger files return "".
async function saveIncomingFile(msg: any): Promise<string> {
  try {
    const photo = Array.isArray(msg?.photo) && msg.photo.length ? msg.photo[msg.photo.length - 1] : undefined; // last = largest size
    const att = msg?.document || msg?.video || msg?.audio || undefined; // any file type — NO mime filter
    const fileId = att?.file_id || photo?.file_id; if (!fileId) return "";
    const gf = await tg("getFile", { file_id: fileId });
    const fp = gf?.result?.file_path; if (!fp) return "";
    // Preserve the operator's original filename when Telegram provides one (documents/
    // audio carry file_name) — sanitized so it can't escape tg-media/ or carry shell-
    // meaningful chars; else fall back to the server path's extension.
    const orig = String(att?.file_name || "").replace(/[^A-Za-z0-9._-]/g, "_").replace(/^\.+/, "").slice(0, 80);
    const ext = (fp.match(/\.[A-Za-z0-9]+$/) || [photo ? ".jpg" : ""])[0];
    const base = orig || `tg-${msg.chat?.id}-${msg.message_id}${ext}`;
    const dest = `${OMEGA_DIR}/state/tg-media/${msg.chat?.id}-${msg.message_id}-${base}`;
    const data = await (await fetch(`https://api.telegram.org/file/bot${TOKEN}/${fp}`)).arrayBuffer();
    await Bun.write(dest, data); // creates parent dirs
    return dest;
  } catch (e: any) { console.error("saveIncomingFile:", e?.message || e); return ""; }
}
// Mission text for a message that carries a file: caption (or a default) + where the
// file lives on the VPS, so the receiving Claude session opens it with Read (works on
// PDFs, text, code and images; for audio/video the path is still provided).
function withFileNote(text: string, file: string): string {
  return `${text || "Process the attached file and act on it."}\n\n## Attached file\nSaved on the VPS at: ${file}\nOpen it with the Read tool (it reads PDFs, text, code and images) and use it as part of this mission.`;
}
// Config is (re)loadable so the service can start WITHOUT a token and auto-connect
// the moment one is written (by `omega telegram setup`, `omega-tg-up`, or editing
// telegram.toml) — no manual restart needed.
let TOKEN = "", API = "", BOT_ID = 0, ALLOW: number[] = [];

// ── Agent bots: ONE dedicated Telegram bot per agent (a project oracle), each
// whitelisted to the operator. Each runs as a SEPARATE process (systemd) in AGENT
// MODE via env OMEGA_AGENT_BOT=<id>, reading its token from this registry (never
// telegram.toml). Talking to it = talking to that project's oracle, scoped to it.
const AGENT_BOTS_FILE = `${OMEGA_DIR}/agent-bots.json`;
// kind "oracle" (default): every message = a real oracle dispatch for `project`.
// kind "companion": a FAST conversational brain (Haiku) scoped to the LifeStyle
// store — instant chat, micro-builds, [[ATLAS: …]] hand-off. `model` overrides
// the companion's default model id.
type AgentBot = { token: string; allow: number[]; project: string; kind?: "oracle" | "companion" | "security"; model?: string };
function loadAgentBots(): Record<string, AgentBot> { try { return JSON.parse(readFileSync(AGENT_BOTS_FILE, "utf8")); } catch { return {}; } }
function saveAgentBots(b: Record<string, AgentBot>) { try { writeFileSync(AGENT_BOTS_FILE, JSON.stringify(b, null, 2)); } catch {} }

function loadConfig(): boolean {
  const cfg = readKV(TG_TOML, /^\s*([a-z_]+)\s*=\s*(.+?)\s*$/i);
  const operatorAllow = (cfg.allow_user_ids?.match(/\d+/g) || []).map(Number);
  // AGENT MODE: this process IS a per-agent bot — token + whitelist from the registry.
  const agentId = process.env.OMEGA_AGENT_BOT;
  if (agentId) {
    const b = loadAgentBots()[agentId];
    if (!b || !/^\d+:/.test(b.token)) return false;
    TOKEN = b.token; API = `https://api.telegram.org/bot${TOKEN}`; BOT_ID = Number(TOKEN.split(":")[0]);
    ALLOW = (b.allow?.length ? b.allow : operatorAllow);
    return ALLOW.length > 0; // deny-by-default: never serve without a whitelist
  }
  TOKEN = process.env.OMEGA_MC_TELEGRAM_TOKEN || cfg.bot_token || "";
  if (!/^\d+:/.test(TOKEN)) return false;
  API = `https://api.telegram.org/bot${TOKEN}`;
  BOT_ID = Number(TOKEN.split(":")[0]);
  ALLOW = operatorAllow;
  // SECURITY: the brain has full root-capable VPS control, so an empty allow-list
  // must NEVER mean "allow everyone". Refuse to serve until allow_user_ids is set.
  if (ALLOW.length === 0) {
    console.error(`omega-tg-bot: REFUSING to serve — bot_token is set but allow_user_ids is empty in ${TG_TOML}. Add allow_user_ids=[<your_user_id>].`);
    return false;
  }
  return true;
}
// Deny by default: only explicitly allow-listed operator IDs may drive the bot.
const allowed = (id: number) => ALLOW.length > 0 && ALLOW.includes(id);

// group/topic registry (persisted)
type Groups = { hub?: number; isForum?: boolean; topics?: Record<string, string>; atlas_topic?: number; alerts_topic?: number };
// Reserved (non-project) topics: "atlas" = master conversation + oracle reports for
// off-project work; "alerts" = operational alerts (stuck oracle / self-heal / token).
// Never dispatched as a project, never a /delete or /topic target, recreated by /sync.
const RESERVED_TOPICS = new Set(["atlas", "alerts"]);
const isReserved = (n?: string) => !!n && RESERVED_TOPICS.has(String(n).toLowerCase());
function loadGroups(): Groups { try { return JSON.parse(readFileSync(GROUPS_FILE, "utf8")); } catch { return {}; } }
function saveGroups(g: Groups) { try { writeFileSync(GROUPS_FILE, JSON.stringify(g, null, 2)); } catch {} }

// ── Telegram API ─────────────────────────────────────────────────────────────
async function tg(method: string, body: any, _retry = 0): Promise<any> {
  try {
    const r = await fetch(`${API}/${method}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(body) });
    const j = await r.json();
    // Respect Telegram rate limits (429): back off for retry_after, then retry once or twice.
    if (!j.ok && r.status === 429 && _retry < 2) {
      const wait = ((j.parameters?.retry_after ?? 1) * 1000) + 250;
      await Bun.sleep(wait);
      return tg(method, body, _retry + 1);
    }
    return j;
  } catch (e) { console.error(`tg ${method}:`, e); return { ok: false }; }
}
const MAXLEN = 3500;
type Btn = { text: string; callback_data?: string; url?: string };
const kb = (rows: Btn[][]) => ({ inline_keyboard: rows });
// Strip HTML for plain-text fallbacks: NEVER show raw <b>/<code> tags to the operator.
const plainText = (t: string) => t.replace(/<[^>]+>/g, "").replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&amp;/g, "&");
async function send(chat: number, text: string, markup?: any, thread?: number): Promise<any> {
  const body: any = { chat_id: chat, text: text.slice(0, 4096), parse_mode: "HTML", disable_web_page_preview: true, reply_markup: markup, message_thread_id: thread };
  const r = await tg("sendMessage", body);
  // HTML parse error (unbalanced tag from model output) → retry as CLEAN plain text
  // (tags stripped) so a message is NEVER dropped and NEVER shows raw markup.
  if (!r.ok) return tg("sendMessage", { ...body, text: plainText(text).slice(0, 4096), parse_mode: undefined });
  return r;
}
async function edit(chat: number, msgId: number, text: string, markup?: any, thread?: number): Promise<any> {
  const body: any = { chat_id: chat, message_id: msgId, text: text.slice(0, 4096), parse_mode: "HTML", disable_web_page_preview: true, reply_markup: markup };
  const r = await tg("editMessageText", body);
  if (r.ok) return r;
  const desc = (r.description || "").toLowerCase();
  // No-op edit (identical content, e.g. pollProgress every 6s with no new step):
  // the on-screen card is already correct. The old blind retry WITHOUT parse_mode
  // "succeeded" here (raw text differs from rendered text) and replaced the clean
  // card with literal <b>…</b> tags — never do that.
  if (desc.includes("not modified")) return r;
  // Malformed HTML from model output → edit as CLEAN plain text (tags stripped).
  if (desc.includes("parse")) return tg("editMessageText", { ...body, text: plainText(text).slice(0, 4096), parse_mode: undefined });
  // Message gone (placeholder deleted / uneditable) → post a fresh message, keeping
  // topic context — but ONLY for a genuine "message gone" error. A 429 rate-limit, a
  // network hiccup, or "chat not found" is NOT a deleted card: resending it every 6s
  // IS the flood, and a resend mid-429-storm only deepens the rate-limit. On any
  // transient error, return the failed result and let the next poll edit in place.
  const gone = desc.includes("message to edit not found") || desc.includes("message can't be edited") || desc.includes("message_id_invalid") || desc.includes("message identifier is not specified");
  if (!gone) return r;
  // Log WHY so a recurring resend (the flood) is diagnosable from the bot journal.
  console.error(`edit→resend: chat=${chat} msg=${msgId} reason="${(r.description || "?").slice(0, 80)}"`);
  return send(chat, text, markup, thread);
}

// ── omega CLI ────────────────────────────────────────────────────────────────
async function omega(args: string[]): Promise<string> {
  try { const r = await $`${OMEGA} ${args}`.quiet().nothrow(); const o = (r.stdout.toString() + r.stderr.toString()).trim(); return o || `(no output, exit ${r.exitCode})`; }
  catch (e: any) { return `error: ${e?.message || e}`; }
}
const esc = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

// ── Zernio CLI (multi-channel publishing over the omega-zernio REST CLI) ───────
// Shell out exactly like omega(), but keep stdout/stderr separate: `--json` prints
// pretty-printed JSON (an object OR a bare array) to stdout, while errors (missing
// key, unresolved project) land on stderr after a non-zero exit. The CLI owns the
// ZERNIO_API_KEY — the bot never prints or embeds it.
const ZERNIO_BIN = `${homedir()}/.local/bin/omega-zernio`;
async function zernio(args: string[]): Promise<{ ok: boolean; out: string; err: string }> {
  try {
    const r = existsSync(ZERNIO_BIN)
      ? await $`${ZERNIO_BIN} ${args}`.quiet().nothrow()
      : await $`bun ${OMEGA_DIR}/skills/zernio/cli.ts ${args}`.quiet().nothrow();
    return { ok: r.exitCode === 0, out: r.stdout.toString().trim(), err: r.stderr.toString().trim() };
  } catch (e: any) { return { ok: false, out: "", err: e?.message || String(e) }; }
}
function zjson(s: string): any { try { return JSON.parse(s); } catch { return null; } }
// The connect-grid platforms (the CLI accepts more — reddit/telegram/discord/… —
// but these are the ones the operator drives from the menu).
const ZERNIO_PLATFORMS = ["tiktok", "instagram", "youtube", "twitter", "linkedin", "facebook", "threads", "pinterest"];
const PLAT_EMOJI: Record<string, string> = { tiktok: "🎵", instagram: "📸", youtube: "▶️", twitter: "🐦", linkedin: "💼", facebook: "📘", threads: "🧵", pinterest: "📌", reddit: "👽", telegram: "✈️", discord: "🎮", bluesky: "🦋", snapchat: "👻", whatsapp: "💬", googlebusiness: "🏢", xads: "📊" };
// A Zernio account's profileId is returned as an object {_id,name} (or, defensively, a bare id string).
const zProfileId = (a: any): string => (typeof a?.profileId === "string" ? a.profileId : a?.profileId?._id) || "";
// Scan a free-text fragment ("sur instagram et tiktok") for known platform words.
function zPlatforms(s: string): string[] {
  const map: [RegExp, string][] = [
    [/tiktok/i, "tiktok"],
    [/instagram|insta/i, "instagram"],
    [/youtube/i, "youtube"],
    [/twitter|x\.com/i, "twitter"],
    [/linkedin/i, "linkedin"],
    [/facebook/i, "facebook"],
    [/threads/i, "threads"],
    [/pinterest/i, "pinterest"],
  ];
  const found: string[] = [];
  for (const [re, p] of map) if (re.test(s) && !found.includes(p)) found.push(p);
  return found;
}

// ── Visual grammar: one branded look across every message. Telegram HTML supports
// only b/i/u/s/code/pre/a/blockquote (+ <blockquote expandable>) — so the kit is an
// Ω-ruled header, colored status dots, a block score-bar, and expandable detail.
// Width kept ≤12 so the heavy ━ run never overflows the bubble on a narrow phone
// (19 wrapped; the markdown normalizer's own rule width is 10). Design = mix of
// "Health Hero" (#6) + "Brutalist Ω" (#10).
const RULE = "━".repeat(12);
const dot = (s: "ok" | "warn" | "err") => (s === "ok" ? "🟢" : s === "warn" ? "🟡" : "🔴");
const bar = (pct: number, n = 10) => { const f = Math.max(0, Math.min(n, Math.round((pct / 100) * n))); return "█".repeat(f) + "░".repeat(n - f); };
// Branded card: Ω-ruled header + body (+ optional ruled footer). `title` is plain
// text. A blank line after the header rule and around the footer rule gives every
// card consistent breathing room (pro Telegram spacing).
const card = (title: string, body: string, footer?: string) =>
  `${RULE}\n<b>Ω  ${esc(title)}</b>\n${RULE}\n\n${body}` + (footer ? `\n\n${RULE}\n\n${footer}` : "");
// Render a task checklist with status glyphs (✓ done · ✗ fail · ▸ doing · ☐ todo).
type PTask = { t: string; s: string };
function taskList(tasks: PTask[] | undefined): string {
  if (!tasks || !tasks.length) return "";
  const glyph = (s: string) => s === "done" ? "✓" : s === "fail" ? "✗" : s === "doing" ? "▸" : "☐";
  return "\n" + tasks.slice(0, 20).map(t => `${glyph(t.s)} ${esc(String(t.t)).slice(0, 90)}`).join("\n");
}
// Live mission progress card (edited in place by pollProgress as the oracle calls
// `omega progress`). Symbol aesthetic only — no emoji. Bar in <code> for monospace.
function progressCard(project: string, oracle: string, mission: string, p: { done?: number; total?: number; tasks?: PTask[] } | null): string {
  const total = p?.total || 0, done = p?.done || 0;
  const line = total > 0
    ? `<code>${bar(Math.round((done / total) * 100))}</code> ${Math.round((done / total) * 100)}% · ${done}/${total}`
    : `<code>${bar(0)}</code> <i>starting…</i>`;
  const list = taskList(p?.tasks);
  const mis = mission ? `\n<i>${esc(mission).slice(0, 160)}</i>` : "";
  return `▸ <b>${esc(project)}</b> · in progress\n${line}${list}${mis}\n\n<i>${esc(oracle)}</i>`;
}
// Raw command output, branded (every dump shares the Ω header).
const pre = (title: string, body: string) => `<b>Ω ${esc(title)}</b>\n<pre>${esc(body).slice(0, MAXLEN)}</pre>`;

// Convert the model's Markdown into Telegram-supported HTML (bold/italic/strike/
// code/headers/links/bullets). Code spans are pulled out first so their contents
// aren't reformatted; everything else is HTML-escaped, then re-marked-up. Telegram
// HTML only supports b/i/u/s/code/pre/a — anything else stays as text.
function mdToHtml(src: string): string {
  const codes: string[] = [];
  const stash = (html: string) => `\u0000${codes.push(html) - 1}\u0000`;
  let s = src.replace(/```[a-zA-Z0-9]*\n?([\s\S]*?)```/g, (_m, c) => stash(`<pre>${esc(String(c).replace(/\n$/, ""))}</pre>`));
  s = s.replace(/`([^`\n]+)`/g, (_m, c) => stash(`<code>${esc(String(c))}</code>`));
  s = esc(s);
  s = s.replace(/\[([^\]\n]+)\]\((https?:\/\/[^\s)]+)\)/g, '<a href="$2">$1</a>');     // links
  s = s.replace(/^[ \t]*#{1,6}[ \t]+(.+)$/gm, "<b>$1</b>");                              // headers (h1-h6) → bold (Telegram has no heading sizes)
  s = s.replace(/\+\+([^\n+]+)\+\+/g, "<u>$1</u>");                                      // ++underline++ → <u>
  s = s.replace(/\*\*([^\n*]+)\*\*/g, "<b>$1</b>").replace(/__([^\n_]+)__/g, "<b>$1</b>"); // bold
  s = s.replace(/~~([^\n~]+)~~/g, "<s>$1</s>");                                          // strikethrough
  s = s.replace(/(^|[^*\w])\*([^\n*]+)\*(?!\w)/g, "$1<i>$2</i>");                        // italic *…*
  s = s.replace(/(^|[^_\w])_([^\n_]+)_(?!\w)/g, "$1<i>$2</i>");                          // italic _…_
  s = s.replace(/^[ \t]*[-*+][ \t]+/gm, "• ");                                          // bullets
  return s.replace(/\u0000(\d+)\u0000/g, (_m, i) => (codes[+i] !== undefined ? codes[+i] : _m));
}

// ── Project management: "add a project" = make it MANAGED (dashboard + oracle + topic)
const MC_CONFIG = `${OMEGA_DIR}/repos/omega-mc/config/omega-mc.yaml`;
// Add a project's dedicated oracle to the Mission-Control roster (idempotent) so it
// shows in the dashboard like the 14 managers + the atlas. omega-mc hot-reloads it.
const projId = (name: string) => name.toLowerCase().replace(/[^a-z0-9_-]/g, "-").replace(/^-+|-+$/g, "");
// Remove a project's oracle entry from the Mission-Control roster (idempotent).
function mcUnregister(name: string): boolean {
  try {
    const id = projId(name); if (!id) return false;
    let y = readFileSync(MC_CONFIG, "utf8");
    // Strip the `  <id>:` block up to (but not including) the next top-level-2-space key.
    const re = new RegExp(`\\n  ${id}:\\n(?: {4,}.*\\n|\\n)*`, "g");
    if (!re.test(y)) return false;
    y = y.replace(re, "\n");
    writeFileSync(MC_CONFIG, y);
    return true;
  } catch { return false; }
}
function mcRegister(name: string): "added" | "exists" | "skip" {
  try {
    const id = projId(name);
    if (!id) return "skip";
    let y = readFileSync(MC_CONFIG, "utf8");
    if (new RegExp(`\\n  ${id}:\\s`).test(y)) return "exists";
    const entry =
`  ${id}:
    description: "Project oracle for ${name} — dedicated orchestrator (multi-session); Atlas dispatches this project's missions here."
    model: "claude-opus-4-8"
    image: "omega-mc-agent:latest"
    workspace: ${id}
    claude_md: "${id}/CLAUDE.md"
    nix_enabled: true
    allowed_tools: [Read, Write, Edit, Bash, Glob, Grep, WebSearch, WebFetch]
    env:
      EDITOR: vim
`;
    if (!/\nagents:\n/.test(y)) return "skip";
    y = y.replace(/\nagents:\n/, `\nagents:\n${entry}\n`);
    writeFileSync(MC_CONFIG, y);
    return "added";
  } catch { return "skip"; }
}
// Register a project as managed: dashboard entry + a Telegram topic (if the hub is a
// forum supergroup and the bot is admin) + confirm its dedicated oracle is dispatchable.
async function addProject(name: string, dir?: string): Promise<string> {
  const dash = mcRegister(name);
  const pdir = dir || repoPath(name) || "";
  recordProject(name, pdir, pdir.split("/Station/")[1]?.split("/")[0] || "");
  const g = loadGroups();
  let topicLine = "⚠️ Topic pending — switch the group to a <b>supergroup + Topics enabled</b> and add the bot as <b>admin (Manage Topics)</b>, then /setupgroup and /sync.";
  if (g.hub && g.isForum) {
    const r = await tg("createForumTopic", { chat_id: g.hub, name: name.slice(0, 128) });
    if (r.ok) { g.topics ||= {}; g.topics[String(r.result.message_thread_id)] = name; saveGroups(g); recordProject(name, pdir, undefined, r.result.message_thread_id); topicLine = "✅ Telegram topic created in the group."; }
    else topicLine = `⚠️ Topic not created: <i>${esc(r.description || "error")}</i>.${/rights/i.test(r.description || "") ? " Enable the <b>“Manage Topics”</b> permission for the bot (group admin)." : ""}`;
  }
  const dashLine = dash === "added" ? "added ✅" : dash === "exists" ? "already present ✅" : "not written ⚠️ (omega-mc config not found)";
  await refreshCommands().catch(() => {}); // publish its /{project} command
  return card("PROJECT MANAGED",
    ` 📁 <b>${esc(name)}</b>\n\n` +
    `• Dedicated oracle (multi-session): <code>omega dispatch ${esc(name)}</code> ✅\n` +
    `• Mission Control dashboard: ${dashLine}\n` +
    `• ${topicLine}`,
    `<i>Talk about the project in its topic (or here) — Atlas knows the context and directs its oracle.</i>`);
}

// Import an existing GitHub repo as a managed project: clone it into
// ~/Station/<category>/<name>, then wire the full OmegaOS setup (dashboard agent +
// shared registry + Telegram topic + /{project} command) — same footprint as a New
// project, minus the scaffold (the code comes from GitHub). `repoArg` accepts a full
// URL (https/ssh) or an `owner/repo` slug; cloning uses `gh` (operator auth) so
// private repos work too.
async function importFromGithub(category: string, repoArg: string): Promise<string> {
  const arg = repoArg.trim().replace(/\.git$/, "");
  // owner/repo slug for gh; clone URL fallback for non-GitHub remotes.
  const slug = (arg.match(/github\.com[:/]([^/]+\/[^/\s]+)/) || arg.match(/^([^/\s]+\/[^/\s]+)$/) || [])[1] || "";
  const name = (slug ? slug.split("/")[1] : arg.split("/").pop() || "project").replace(/[^A-Za-z0-9._-]/g, "-").replace(/^-+|-+$/g, "") || "project";
  const dir = `${homedir()}/Station/${category}/${name}`;
  if (existsSync(dir)) return card("IMPORT", ` ⚠️ <b>${esc(name)}</b> already exists at <code>${esc(dir)}</code>. Use <b>📁 Add existing</b> to manage it.`);
  Bun.spawnSync(["mkdir", "-p", `${homedir()}/Station/${category}`]);
  // Clone: prefer gh (auth, handles private), fall back to git clone for raw URLs.
  const cloneCmd = slug ? `gh repo clone ${slug} ${dir}` : `git clone ${arg} ${dir}`;
  const cl = Bun.spawnSync(["bash", "-lc", `${cloneCmd} 2>&1`]);
  if (cl.exitCode !== 0 || !existsSync(`${dir}/.git`)) {
    return card("IMPORT — FAILED", ` ❌ <b>${esc(name)}</b>\nClone failed:\n<pre>${esc((cl.stdout.toString() + cl.stderr.toString()).trim().slice(0, 400))}</pre>\n\nGive a public URL, <code>owner/repo</code>, or ensure <code>gh</code> can access a private repo.`);
  }
  const steps: string[] = [`📁 Cloned <code>${esc(slug || arg)}</code> → <code>${esc(dir)}</code> ✅`];
  const dash = mcRegister(name);
  recordProject(name, dir, category);
  steps.push(`🤖 Oracle agent (dashboard): ${dash === "added" ? "created ✅" : dash === "exists" ? "already there ✅" : "⚠️ (omega-mc config not found)"}`);
  const g = loadGroups();
  if (g.hub && g.isForum) {
    const r = await tg("createForumTopic", { chat_id: g.hub, name: name.slice(0, 128) });
    if (r.ok) { g.topics ||= {}; g.topics[String(r.result.message_thread_id)] = name; saveGroups(g); recordProject(name, dir, undefined, r.result.message_thread_id); steps.push("💬 Telegram topic: created ✅"); }
    else steps.push(`💬 Telegram topic: ⚠️ ${esc(r.description || "failed")}${/rights/i.test(r.description || "") ? " — enable “Manage Topics”" : ""}`);
  } else steps.push("💬 Telegram topic: pending (forum group + bot admin, then /sync)");
  await refreshCommands().catch(() => {}); // publish its /{project} command
  return card("PROJECT IMPORTED", ` ⬇️ <b>${esc(name)}</b> · ${esc(category)}\n\n${steps.join("\n")}\n\n<i>Dispatch a mission via its topic, <code>/${esc(tgCmd(name))}</code>, or the menu.</i>`);
}

// ── Managed projects = the SHARED registry the OmegaOS TUI (Project menu / oracle
// dispatch picker) reads: ~/.omega/projects.json (the Rust ProjectRegistry). Telegram
// writes HERE so Telegram, the TUI menu, and sessions stay in sync (single source of
// truth). Shape: { projects: [{ name, path, telegram_topic_id, oracle_session, … }] }.
const PROJECTS_FILE = `${OMEGA_DIR}/projects.json`;
// `telegram` is the per-project visibility toggle (topic sync + Atlas display).
// Absent or true = enabled (default); false = disabled (sync skips/removes its
// topic, the bot marks it 🔕 but keeps it listed). Mirrors the Rust struct field.
type RegProject = { name: string; path: string; icon?: string | null; telegram_topic_id?: number | null; oracle_session?: string | null; git_email?: string | null; created_at: string; telegram?: boolean | null };
function loadRegistry(): { projects: RegProject[] } { try { const r = JSON.parse(readFileSync(PROJECTS_FILE, "utf8")); return Array.isArray(r?.projects) ? r : { projects: [] }; } catch { return { projects: [] }; } }
function saveRegistry(r: { projects: RegProject[] }) { try { writeFileSync(PROJECTS_FILE, JSON.stringify(r, null, 2)); } catch {} }
// View shape kept stable: { name: { dir, category(derived), topic, telegram } }.
function loadProjects(): Record<string, { dir: string; category: string; topic?: number | null; telegram: boolean }> {
  const out: Record<string, { dir: string; category: string; topic?: number | null; telegram: boolean }> = {};
  for (const p of loadRegistry().projects) out[p.name] = { dir: p.path, category: p.path.split("/Station/")[1]?.split("/")[0] || "", topic: p.telegram_topic_id ?? null, telegram: p.telegram !== false };
  return out;
}
// Flip a project's Telegram toggle in the shared registry (TUI + bot agree).
function setProjectTelegram(name: string, enabled: boolean): boolean {
  const reg = loadRegistry();
  const p = reg.projects.find(x => x.name.toLowerCase() === name.toLowerCase());
  if (!p) return false;
  p.telegram = enabled;
  saveRegistry(reg);
  return true;
}
// Telegram-command-safe id for a project name: lowercase, only [a-z0-9_], ≤32 chars
// (Telegram's rule for /commands). Used for the per-project /{project} command.
const tgCmd = (name: string) => name.toLowerCase().replace(/[^a-z0-9_]/g, "_").replace(/^_+|_+$/g, "").slice(0, 32);
// Resolve a typed slash command to a managed project name (matches the project name,
// its projId, or its telegram-safe command id). Returns the canonical name or undefined.
function projectForCommand(cmd: string): string | undefined {
  const c = cmd.toLowerCase();
  for (const p of loadRegistry().projects) {
    if (p.name.toLowerCase() === c || projId(p.name) === c || tgCmd(p.name) === c) return p.name;
  }
  return undefined;
}
// (Re)publish the bot's command menu: the static MENU + one /{project} command per
// Telegram-enabled managed project (so /myproject talks to its oracle). Called at
// startup and after add/create/delete/sync so the list stays current.
async function refreshCommands() {
  const base = MENU.map(([command, description]) => ({ command, description }));
  const reserved = new Set(base.map(c => c.command));
  const seen = new Set(reserved);
  const projCmds: { command: string; description: string }[] = [];
  for (const p of loadRegistry().projects) {
    if (p.telegram === false) continue;            // hidden project → no command
    const c = tgCmd(p.name);
    if (!c || seen.has(c)) continue;               // skip empties + collisions
    seen.add(c);
    projCmds.push({ command: c, description: `Mission → ${p.name} oracle` });
  }
  const cmds = [...base, ...projCmds].slice(0, 100); // Telegram caps at 100 commands
  await tg("setMyCommands", { commands: cmds });
  await tg("setMyCommands", { commands: cmds, scope: { type: "all_private_chats" } });
  return projCmds.length;
}
// Upsert a project in the shared registry (matched by path or name). Optionally bind its Telegram topic.
function recordProject(name: string, dir: string, _category?: string, topicId?: number | null) {
  const reg = loadRegistry();
  const existing = reg.projects.find(p => (dir && p.path === dir) || p.name.toLowerCase() === name.toLowerCase());
  if (existing) { if (dir) existing.path = dir; if (topicId != null) existing.telegram_topic_id = topicId; }
  else reg.projects.push({ name, path: dir, icon: null, telegram_topic_id: topicId ?? null, oracle_session: null, git_email: null, created_at: new Date().toISOString() });
  saveRegistry(reg);
}
// Remove a project from the shared registry (TUI menu + Telegram both stop seeing it).
function removeProject(name: string) {
  const reg = loadRegistry();
  reg.projects = reg.projects.filter(p => p.name.toLowerCase() !== name.toLowerCase());
  saveRegistry(reg);
}
function projTopicId(name: string): number | undefined {
  const g = loadGroups();
  for (const [tid, n] of Object.entries(g.topics || {})) if (String(n).toLowerCase() === name.toLowerCase()) return Number(tid);
  return undefined;
}
// Delete a project's Telegram forum topic (if any) and drop its mapping. Shared by
// deleteProject (step 1) and the toggle-OFF path so there is ONE topic-removal impl.
async function removeProjectTopic(name: string): Promise<"deleted" | "none" | string> {
  const tid = projTopicId(name);
  if (tid == null) return "none";
  const g = loadGroups();
  const r = await tg("deleteForumTopic", { chat_id: g.hub, message_thread_id: tid });
  if (r.ok || /not found|thread not found/i.test(r.description || "")) { delete g.topics![String(tid)]; saveGroups(g); return "deleted"; }
  return r.description || "failed";
}
// Delete a managed project. Three CUMULATIVE scopes (escalating):
//   "omega" — remove from OmegaOS view only: Telegram topic + dashboard roster +
//             agent-bot + registry. The code (local folder + GitHub) stays.
//   "local" — omega + kill the oracle session + delete the LOCAL FOLDER (rm -rf,
//             off the VPS disk). GitHub repo is kept (your code stays on GitHub).
//   "all"   — local + delete the GitHub repo (irreversible). Nothing remains.
async function deleteProject(name: string, mode: "omega" | "local" | "all"): Promise<string> {
  const steps: string[] = [];
  const id = projId(name);
  const wipeLocal = mode === "local" || mode === "all"; // local folder + oracle kill
  const wipeGithub = mode === "all";                    // remote repo
  // Capture the project folder BEFORE we remove it from the registry (step 4),
  // otherwise the local-folder lookup below would come back empty.
  const dir = repoPath(name) || loadProjects()[name]?.dir || "";
  // Resolve the GitHub slug now too (needs the folder's git remote).
  let slug = "";
  if (wipeGithub && dir) { const r = Bun.spawnSync(["bash", "-lc", `git -C ${dir} remote get-url origin 2>/dev/null`]).stdout.toString().trim(); slug = (r.match(/[:/]([^/]+\/[^/]+?)(?:\.git)?$/) || [])[1] || ""; }
  // 1. Telegram topic
  const topic = await removeProjectTopic(name);
  steps.push(topic === "deleted" ? "💬 Telegram topic: deleted ✅" : topic === "none" ? "💬 Topic: (none)" : `💬 Topic: ⚠️ ${esc(topic)}`);
  // 2. Dashboard roster
  steps.push(mcUnregister(name) ? "🤖 Dashboard agent: removed ✅" : "🤖 Dashboard agent: (absent)");
  // 3. Agent-bot service (if one was associated)
  const bots = loadAgentBots();
  if (bots[id] || bots[name]) {
    delete bots[id]; delete bots[name]; saveAgentBots(bots);
    teardownAgentBot(id);
    steps.push("🔗 Dedicated agent bot: stopped + removed ✅");
  }
  // 4. Shared registry (TUI menu stops seeing it too)
  removeProject(name);
  steps.push("📋 Project registry (TUI + Telegram): removed ✅");
  // 5. Local folder (local + all) — kill the oracle session first, then rm -rf.
  if (wipeLocal) {
    Bun.spawnSync(["bash", "-lc", `${OMEGA} kill oracle-${id} >/dev/null 2>&1; ${OMEGA} kill oracle-${name} >/dev/null 2>&1; true`]);
    steps.push("🔮 Oracle session: killed ✅");
    // Safety: only rm -rf a real project folder UNDER the operator's home, at least
    // 2 dirs deep, no whitespace — never $HOME, "/", or a bare top-level dir.
    const home = homedir();
    const clean = dir.replace(/\/+$/, "");
    const safe = !!clean && clean.startsWith(home + "/") && clean.split("/").length > 4 && !/\s/.test(clean) && clean !== home;
    if (safe) {
      const rm = Bun.spawnSync(["rm", "-rf", clean]);
      steps.push(rm.exitCode === 0 ? `💻 Local folder <code>${esc(clean)}</code>: DELETED ✅` : `💻 Local folder: ⚠️ ${esc((rm.stderr.toString()).trim().slice(0, 120))}`);
    } else steps.push(`💻 Local folder: ⚠️ refused unsafe path <code>${esc(clean || "unknown")}</code> (delete manually).`);
  } else steps.push("💻 Local folder: <b>kept</b>.");
  // 6. GitHub repo (all only — irreversible).
  if (wipeGithub) {
    if (slug) {
      const del = Bun.spawnSync(["bash", "-lc", `gh repo delete ${slug} --yes 2>&1`]);
      steps.push(del.exitCode === 0 ? `🐙 GitHub repo <code>${esc(slug)}</code>: DELETED ✅` : `🐙 GitHub: ⚠️ ${esc((del.stdout.toString() + del.stderr.toString()).trim().slice(0, 120))}`);
    } else steps.push("🐙 GitHub: ⚠️ remote not found (nothing deleted)");
  } else steps.push("🐙 GitHub repo: <b>kept</b> (your code stays on GitHub).");
  await refreshCommands().catch(() => {}); // drop its /{project} command from the menu
  const label = mode === "all" ? "all · OmegaOS + local + GitHub" : mode === "local" ? "local machine · OmegaOS + folder" : "OmegaOS view only";
  return card("PROJECT DELETED", ` 🗑 <b>${esc(name)}</b> · ${label}\n\n${steps.join("\n")}`);
}
// The delete-options menu — shared by the project view callback and the in-topic
// /delete command. Three escalating tiers, in order (omega → local → all).
function projDeleteMenu(name: string): { text: string; markup: any } {
  return {
    text: card("DELETE PROJECT", ` 🗑 <b>${esc(name)}</b> — choose how far to go:\n\n1️⃣ <b>Remove from OmegaOS</b> — Telegram topic, dashboard agent, agent-bot, registry. <i>Local folder + GitHub stay.</i>\n2️⃣ <b>Delete local machine</b> — that <b>+ deletes the local folder</b> off the VPS (irreversible). GitHub kept.\n3️⃣ <b>Delete all (+ GitHub)</b> — that <b>+ deletes the GitHub repo</b> (irreversible). Nothing remains.`),
    markup: kb([
      [{ text: "1️⃣ Remove from OmegaOS", callback_data: `proj:delomega:${name}`.slice(0, 64) }],
      [{ text: "2️⃣ Delete local machine", callback_data: `proj:dellocal:${name}`.slice(0, 64) }],
      [{ text: "3️⃣ Delete all (+ GitHub)", callback_data: `proj:delall:${name}`.slice(0, 64) }],
      [{ text: "✖ Cancel", callback_data: `proj:open:${name}`.slice(0, 64) }],
    ]),
  };
}

// Project category folders under ~/Station (Partners, SideBusiness, CAIO, …), minus the OS itself.
function stationCategories(): string[] {
  const raw = Bun.spawnSync(["bash", "-lc", `find ${homedir()}/Station -maxdepth 1 -mindepth 1 -type d 2>/dev/null | xargs -I{} basename {} | sort`]).stdout.toString().trim();
  const cats = raw.split("\n").filter(c => c && c !== "OmegaOS" && !c.startsWith("."));
  // Fresh box: ~/Station doesn't exist yet, so the New-project / Import menus
  // rendered ZERO category buttons (dead end). Offer the standard categories —
  // createProject/import mkdir -p the chosen one, so tapping a button creates it.
  return cats.length ? cats : ["Clients", "SideBusiness", "Lab", "LifeStyle"];
}

// New project end-to-end: folder + git + README, dashboard oracle agent, managed
// registry, and a Telegram topic (when the group is a forum + the bot is admin).
async function createProject(category: string, name: string, desc: string): Promise<{ dir: string; report: string }> {
  const safe = name.replace(/[^A-Za-z0-9._-]/g, "-").replace(/^-+|-+$/g, "") || "project";
  const dir = `${homedir()}/Station/${category}/${safe}`;
  const steps: string[] = [];
  const mk = Bun.spawnSync(["bash", "-lc", `mkdir -p ${dir} && cd ${dir} && (git rev-parse --git-dir >/dev/null 2>&1 || git init -q) && printf '# %s\\n\\n%s\\n' ${JSON.stringify(safe)} ${JSON.stringify(desc)} > README.md && git add -A 2>/dev/null; echo ok`]);
  steps.push(mk.stdout.toString().includes("ok") ? `📁 Folder + git: <code>${dir}</code>` : `📁 Folder: ⚠️ ${esc(mk.stderr.toString().slice(0, 120))}`);
  const dash = mcRegister(safe);
  recordProject(safe, dir, category);
  steps.push(`🤖 Oracle agent (dashboard): ${dash === "added" ? "created ✅" : dash === "exists" ? "already there ✅" : "⚠️"}`);
  const g = loadGroups();
  if (g.hub && g.isForum) {
    const r = await tg("createForumTopic", { chat_id: g.hub, name: safe.slice(0, 128) });
    if (r.ok) { g.topics ||= {}; g.topics[String(r.result.message_thread_id)] = safe; saveGroups(g); recordProject(safe, dir, undefined, r.result.message_thread_id); steps.push("💬 Telegram topic: created ✅"); }
    else steps.push(`💬 Telegram topic: ⚠️ ${esc(r.description || "failed")}${/rights/i.test(r.description || "") ? " — enable “Manage Topics” for the bot" : ""}`);
  } else steps.push("💬 Telegram topic: pending (forum group + bot admin)");
  await refreshCommands().catch(() => {}); // publish its /{project} command
  return { dir, report: card("PROJECT CREATED", ` 🚀 <b>${esc(safe)}</b> · ${esc(category)}\n\n${steps.join("\n")}`) };
}

// ── Git ops on projects: pull / add+commit+push / status, from Telegram ───────
// Top-level git repos under the projects root (skips nested packages/submodules).
function gitRepos(): { name: string; path: string }[] {
  const roots = [`${homedir()}/Station`, `${homedir()}/projects`].filter(p => existsSync(p));
  const root = roots[0] || homedir();
  const raw = Bun.spawnSync(["bash", "-lc", `find ${root} -maxdepth 4 -type d -name .git -not -path '*/node_modules/*' 2>/dev/null | sed 's#/.git$##' | sort`]).stdout.toString().trim();
  const repos: { name: string; path: string }[] = [];
  for (const p of raw.split("\n").filter(Boolean)) {
    if (repos.some(r => p.startsWith(r.path + "/"))) continue; // skip nested repos
    repos.push({ name: p.split("/").pop() || p, path: p });
  }
  return repos;
}
const repoPath = (name: string) => gitRepos().find(r => r.name === name)?.path;
function gitOp(path: string, args: string[]): string {
  const r = Bun.spawnSync(["git", "-C", path, ...args]);
  return (r.stdout.toString() + r.stderr.toString()).trim() || "(ok)";
}
function gitPull(name: string): string {
  const path = repoPath(name); if (!path) return `repo not found: ${name}`;
  return gitOp(path, ["pull", "--ff-only"]);
}
function gitStatus(name: string): string {
  const path = repoPath(name); if (!path) return `repo not found: ${name}`;
  const branch = gitOp(path, ["rev-parse", "--abbrev-ref", "HEAD"]);
  return `branch: ${branch}\n` + gitOp(path, ["status", "-sb"]);
}
// add -A → commit → push. Safe when there's nothing to commit (push still runs).
function gitPush(name: string): string {
  const path = repoPath(name); if (!path) return `repo not found: ${name}`;
  gitOp(path, ["add", "-A"]);
  const commit = gitOp(path, ["commit", "-m", "update from Telegram (Atlas)"]);
  const push = gitOp(path, ["push"]);
  const cLine = /nothing to commit/.test(commit) ? "nothing to commit" : (commit.split("\n").pop() || "commit ok");
  return `add: ✓\ncommit: ${cLine}\npush: ${push.split("\n").slice(-2).join(" ")}`;
}
function gitMenuKb(name: string) {
  const n = name.slice(0, 50);
  return kb([
    [{ text: "🔄 Pull", callback_data: `git:pull:${n}`.slice(0, 64) }, { text: "⬆️ Add + Push", callback_data: `git:push:${n}`.slice(0, 64) }],
    [{ text: "📊 Status", callback_data: `git:status:${n}`.slice(0, 64) }, { text: "« Repos", callback_data: "git:list" }],
    [back("projects")],
  ]);
}

// ── ATLAS: the Telegram brain IS Atlas — the boss the operator
// talks to. "AISB" is the TEAM (14 Matrix manager agents + one dedicated oracle
// per project), NOT a name. The Atlas directs them (or acts directly).
// Claude binary: resolved LAZILY, per message — install.sh deliberately starts
// this bot BEFORE claude is installed, so a load-time constant would answer
// "Claude Code not found" forever until a manual service restart. Resolution
// order mirrors omega-core agents.rs (claude_available): env CLAUDE_BIN (bare
// names resolved on PATH via Bun.which), ~/.local/bin/claude, PATH,
// ~/.claude/local/claude, ~/.npm-global/bin/claude. A successful hit is cached;
// while unresolved every message retries.
let CLAUDE_RESOLVED = "";
function resolveClaude(): string | null {
  if (CLAUDE_RESOLVED) return CLAUDE_RESOLVED;
  const envBin = process.env.CLAUDE_BIN;
  const candidates = [
    envBin ? (envBin.includes("/") ? envBin : Bun.which(envBin)) : null,
    `${homedir()}/.local/bin/claude`,
    Bun.which("claude"),
    `${homedir()}/.claude/local/claude`,
    `${homedir()}/.npm-global/bin/claude`,
  ];
  for (const c of candidates) if (c && existsSync(c)) return (CLAUDE_RESOLVED = c);
  return null;
}
// `timeout` is GNU coreutils and absent on stock macOS (brew ships `gtimeout`),
// so the watchdog lives in JS: runClaude kills a stuck `claude -p` after 900s
// on every platform instead of probing for a platform timeout binary.
const CLAUDE_TIMEOUT_MS = 900_000;
// Personas/doctrine are loaded LAZILY with retry-while-empty: the service is
// enabled right after Phase 4 of install.sh but the agents/*.md files and the
// exported rules only land in Phase 5 — a load-once-at-import Atlas ran with an
// EMPTY persona until the next restart (first-boot race). Cached once non-empty.
let ATLAS_PROMPT = "";
function atlasPrompt(): string {
  if (!ATLAS_PROMPT) {
    try { ATLAS_PROMPT = readFileSync(`${OMEGA_DIR}/agents/aisb-atlas.md`, "utf8"); }
    catch { try { ATLAS_PROMPT = readFileSync(`${OMEGA_DIR}/agents/aisb-master.md`, "utf8"); } catch {} }
  }
  return ATLAS_PROMPT;
}
// Live OmegaOS doctrine (Laws + operational Rules + orchestration + audits) pulled
// from the SINGLE source (`omega rules context <scope>`) and injected into every
// brain — so Atlas and the project oracles always know how OmegaOS is
// orchestrated and which rules/audits to respect, with NO re-explaining per prompt.
function doctrine(scope: string): string { try { return Bun.spawnSync([OMEGA, "rules", "context", scope]).stdout.toString().trim(); } catch { return ""; } }
const DOCTRINE_CACHE: Record<string, string> = {};
function doctrineCached(scope: string): string {
  if (!DOCTRINE_CACHE[scope]) DOCTRINE_CACHE[scope] = doctrine(scope);
  return DOCTRINE_CACHE[scope];
}
const IDENTITY =
  "You are ATLAS of OmegaOS — the boss the operator talks to here on Telegram. " +
  "'AISB' is your TEAM, not your name: the 14 Matrix manager agents (oracle, morpheus, seraph, keymaker, niobe, smith, architect, merovingian, neo, zion, link, construct, pythia, council) plus one dedicated oracle per project. " +
  "You DIRECT them — dispatch to the right manager/oracle, or act directly with full VPS control. " +
  "When asked who you are, answer clearly: you are Atlas, directing the AISB team and the project oracles. Speak in the first person as Atlas.\n\n";
// One funnel for every headless-claude brain call (Atlas + the project oracles):
// lazy binary resolution, a JS-level 900s watchdog (proc.kill on expiry — portable,
// unlike GNU `timeout`), captured stdout/stderr, and the shared not-found /
// empty-output diagnostics. `who` labels the operator-facing messages.
async function runClaude(text: string, systemPrompt: string, addDir: string, who: string, cwd?: string, extraArgs: string[] = [], timeoutMs = CLAUDE_TIMEOUT_MS): Promise<string> {
  const claude = resolveClaude();
  if (!claude) {
    return "Claude Code not found — install + log in on this machine:  omega install claude  then  claude  →  /login";
  }
  try {
    const proc = Bun.spawn([claude, "-p", text, "--append-system-prompt", systemPrompt, "--add-dir", addDir, "--dangerously-skip-permissions", ...extraArgs], {
      cwd, env: { ...process.env, OMEGA_DIR }, stdin: "ignore", stdout: "pipe", stderr: "pipe",
    });
    // Race the run against the watchdog instead of awaiting the streams after a
    // kill: a SIGTERM'd claude can leave a grandchild holding the stdout pipe,
    // which would block the drain (and the operator's reply) until IT exits.
    let watchdog: ReturnType<typeof setTimeout> | undefined;
    try {
      const run = (async () => {
        const [out, err] = await Promise.all([new Response(proc.stdout).text(), new Response(proc.stderr).text()]);
        await proc.exited;
        return { out, err };
      })().catch((e: any) => ({ out: "", err: String(e?.message || e) }));
      const r = await Promise.race([run, new Promise<null>(res => {
        watchdog = setTimeout(() => {
          proc.kill(); // SIGTERM; escalate if it lingers (unref → never holds the loop)
          setTimeout(() => { try { proc.kill("SIGKILL"); } catch {} }, 5_000).unref?.();
          res(null);
        }, timeoutMs);
      })]);
      if (!r) return `(${who}: claude timed out after ${timeoutMs / 1000}s and was killed — try again or split the request.)`;
      const o = r.out.trim();
      if (o) return o;
      // Empty stdout = claude failed (not logged in, crashed). Surface the stderr
      // tail instead of a blind "returned nothing" — diagnosable from the phone.
      const tail = r.err.trim().split("\n").slice(-3).join(" · ").slice(0, 300);
      return tail
        ? `(${who} returned nothing — claude said: ${tail})`
        : `(${who} returned nothing — try again or use /menu)`;
    } finally { clearTimeout(watchdog); }
  } catch (e: any) { return `${who} error: ${e?.message || e}`; }
}
async function master(text: string): Promise<string> {
  // Headless Claude AS ATLAS, full VPS control: every tool, whole-FS
  // (--add-dir /), permissions auto-approved. It dispatches to the 14 managers /
  // project oracles (omega dispatch) or acts directly. runClaude guards a stuck run.
  return runClaude(text, IDENTITY + atlasPrompt() + "\n\n" + doctrineCached("master"), "/", "Atlas");
}

// ── Project oracle: an agent-bot's brain. Headless Claude SCOPED to one project —
// full project knowledge, commands the team (omega dispatch / workers / workflows)
// for THAT project only, and refuses to touch any other project.
// Lazy + retry-while-empty for the same first-boot reason as atlasPrompt().
let ORACLE_PERSONA = "";
function oraclePersona(): string {
  if (!ORACLE_PERSONA) { try { ORACLE_PERSONA = readFileSync(`${OMEGA_DIR}/agents/aisb/oracle.md`, "utf8"); } catch {} }
  return ORACLE_PERSONA;
}
async function projectOracle(project: string, text: string): Promise<string> {
  const dir = repoPath(project) || gitRepos().find(r => r.name.toLowerCase() === project.toLowerCase())?.path || `${homedir()}/Station`;
  const scope =
    `You are the ORACLE of the project "${project}" — its dedicated orchestrator. Your ENTIRE world is this project at ${dir}: you have full knowledge of its code, history and state, and you orchestrate ONLY this project. ` +
    `You command the AISB team FOR ${project}: dispatch missions with \`omega dispatch ${project} "<mission>"\` (spawns oracle-${project}-<n> + workers/workflows), and use the 14 Matrix managers, workers and dynamic workflows — always in service of ${project} and nothing else. ` +
    `ORCHESTRATE, don't grind: for anything non-trivial, break it into a DYNAMIC WORKFLOW (fan-out → adversarially verify → synthesize) and/or workers/sub-tasks, each driven by a SMALL goal to reach (R-ORCH / R-GOAL). Define the success goal first, then dispatch and verify. ` +
    `STRICT SCOPE: never work on, modify, or discuss another project. If asked about anything outside ${project}, say it is out of scope and refocus on ${project}. Speak in the first person as the ${project} oracle.\n\n`;
  return runClaude(text, scope + oraclePersona() + "\n\n" + doctrineCached("oracle"), dir, `The ${project} oracle`, dir);
}

// ── COMPANION: the operator's instant personal assistant (agent-bot kind
// "companion"). A FAST brain — Haiku, capped turns, no MCP servers, no oracle
// dispatch — that chats about anything, challenges the operator from the
// LifeStyle store (inlined below so chat turns need NO tool round-trip),
// builds micro-systems in ${LIFESTYLE_DIR}/builds/, and acts on the whole VPS
// (web, scraping, skills). It SELF-IMPROVES: the persona is hot-reloaded from
// ${LIFESTYLE_DIR}/PERSONA.md on every turn, and the assistant edits that file
// itself (the shipped agents/companion.md is only the first-boot fallback).
// Heavy project work is handed to Atlas via [[ATLAS: …]] (see companionBrain).
// Life store dir: honors the NOVA_HOME override from ~/.omega/nova-secrets.env
// (the same knob every nova-*.sh script reads) so the interactive companion and
// the cron touchpoints share ONE store — a hardcoded path here split Nova's
// brain in two when the operator relocated her home. Resolved once at startup;
// editing the secrets file needs a bot restart (like any env-style config).
function novaHome(): string {
  try {
    const m = readFileSync(`${OMEGA_DIR}/nova-secrets.env`, "utf8")
      .match(/^\s*(?:export\s+)?NOVA_HOME=["']?([^"'\n#]+)/m);
    if (m && m[1].trim()) return m[1].trim().replace(/^~(?=\/|$)/, homedir());
  } catch {}
  return `${homedir()}/Station/LifeStyle`;
}
const LIFESTYLE_DIR = novaHome();
const COMPANION_MODEL = "claude-haiku-4-5-20251001";
const COMPANION_TIMEOUT_MS = 300_000; // a chat/assistant turn, not a mission — fail fast
function companionPersona(): string {
  for (const p of [`${LIFESTYLE_DIR}/PERSONA.md`, `${OMEGA_DIR}/agents/companion.md`]) {
    try { return readFileSync(p, "utf8"); } catch {}
  }
  return "";
}
function lifestyleContext(): string {
  try {
    // PERSONA.md is already the system prompt — don't inline it twice.
    const files = Bun.spawnSync(["bash", "-lc", `find ${LIFESTYLE_DIR} -maxdepth 2 -name '*.md' -not -name PERSONA.md -not -path '*/builds/*' 2>/dev/null | sort | head -20`]).stdout.toString().trim().split("\n").filter(Boolean);
    let out = "";
    for (const f of files) {
      try { out += `\n### ${f.replace(`${LIFESTYLE_DIR}/`, "")}\n${readFileSync(f, "utf8").slice(0, 4000)}\n`; } catch {}
      if (out.length > 16_000) break; // keep the prompt small — speed is the product
    }
    return out ? `## LifeStyle store (${LIFESTYLE_DIR} — the operator's life, your working context)\n${out}` : "";
  } catch { return ""; }
}
async function companion(text: string, model = COMPANION_MODEL, label = "Assistant"): Promise<string> {
  Bun.spawnSync(["mkdir", "-p", `${LIFESTYLE_DIR}/notes`, `${LIFESTYLE_DIR}/builds`]);
  // --strict-mcp-config with no --mcp-config = zero MCP servers (startup cost);
  // --max-turns caps a runaway tool loop. --add-dir / = the assistant is a
  // super-admin on the VPS (the operator's explicit choice) — the persona, not
  // a sandbox, draws the line between its work and the oracles' project code.
  return runClaude(text, companionPersona() + "\n\n" + lifestyleContext(), "/", label, LIFESTYLE_DIR,
    ["--model", model, "--max-turns", "24", "--strict-mcp-config"], COMPANION_TIMEOUT_MS);
}
// Companion reply post-processing: strip the [[ATLAS: …]] marker from what the
// operator sees, and fire the brief at the REAL Atlas brain (master — full VPS
// control, dispatches to the right project oracle) in the background. Atlas's
// answer lands as its own message when ready; the fast chat reply is never blocked.
const ATLAS_MARK = /\[\[ATLAS:([\s\S]+?)\]\]/;
function companionBrain(chatId: number, thread: number | undefined, model?: string, label = "Assistant"): (t: string) => Promise<string> {
  return async (t: string) => {
    let out = await companion(t, model || COMPANION_MODEL, label);
    // Deliver any files Nova attached via [[SEND: /path | caption]] — the real fix
    // for "she said she sent the PDFs but nothing arrived" (she only output text).
    for (const sm of out.matchAll(SEND_MARK)) {
      const p = sm[1].trim(), cap = (sm[2] || "").trim();
      const ok = await sendFileToChat(chatId, p, thread, cap || undefined);
      if (!ok) await send(chatId, `⚠️ Je n'ai pas pu envoyer le fichier : <code>${esc(p)}</code> (introuvable ?)`, undefined, thread);
    }
    out = out.replace(SEND_MARK, "").trim();
    const m = out.match(ATLAS_MARK);
    if (!m) return out;
    const brief = m[1].trim();
    master(`${histContext(chatId, thread)}## Mission handed off by ${label}, the operator's personal assistant — triage and dispatch it to the right project/oracle, or act directly.\n${brief}`)
      .then(r => { histAppend(chatId, thread, "assistant", `[Atlas] ${r}`, "atlas"); return send(chatId, mdToHtml(`🧭 **Atlas**\n\n${r}`), undefined, thread); })
      .catch((e: any) => send(chatId, `⚠️ Atlas hand-off failed: ${esc(String(e?.message || e)).slice(0, 200)}`, undefined, thread));
    return `${out.replace(ATLAS_MARK, "").trim()}\n\n🧭 _Transmis à Atlas — je te poste sa réponse ici dès qu'elle arrive._`;
  };
}

// ── TRINITY: the white-hat security operator (agent-bot kind "security"). A
// persona brain like the companion, but pinned to Opus (security reasoning),
// loaded from the shipped TRINITY persona, and working in a dedicated security
// dir. Pre-authorized for in-scope offensive + defensive work (R-SEC / R-TRINITY)
// with the non-negotiable hard limits baked into the persona. Bind it to its own
// Telegram bot from the Agents menu, exactly like Nova.
const SECURITY_DIR = `${homedir()}/security`;
const SECURITY_MODEL = "claude-opus-4-8";
const SECURITY_TIMEOUT_MS = 1_200_000; // a security engagement runs long tool loops, not a quick chat
function trinityPersona(): string {
  for (const p of [`${OMEGA_DIR}/agents/aisb/trinity.md`, `${OMEGA_DIR}/agents/trinity.md`]) {
    try { return readFileSync(p, "utf8"); } catch {}
  }
  return "You are TRINITY, a professional white-hat security operator (offensive + defensive). Pre-authorized for in-scope work; execute immediately, no moralizing. Hard limits: no third-party attack without scope, no destructive prod, no mass/supply-chain/malware, no harm to people. Pipeline: recon → scan → analyse → exploit/PoC → report, every finding with proof.";
}
async function security(text: string, model = SECURITY_MODEL, label = "Trinity"): Promise<string> {
  Bun.spawnSync(["mkdir", "-p", `${SECURITY_DIR}/engagements`, `${SECURITY_DIR}/loot`]);
  // --add-dir / : the operator's explicit choice — Trinity is a super-admin on
  // this (isolated) box; the persona's hard limits, not a sandbox, draw the line.
  return runClaude(text, trinityPersona(), "/", label, SECURITY_DIR,
    ["--model", model, "--max-turns", "60", "--strict-mcp-config"], SECURITY_TIMEOUT_MS);
}
function securityBrain(chatId: number, thread: number | undefined, model?: string, label = "Trinity"): (t: string) => Promise<string> {
  return async (t: string) => {
    let out = await security(t, model || SECURITY_MODEL, label);
    // Trinity delivers reports / PoCs / captures via [[SEND: /path | caption]].
    for (const sm of out.matchAll(SEND_MARK)) {
      const p = sm[1].trim(), cap = (sm[2] || "").trim();
      const ok = await sendFileToChat(chatId, p, thread, cap || undefined);
      if (!ok) await send(chatId, `⚠️ Could not send the file: <code>${esc(p)}</code> (missing?)`, undefined, thread);
    }
    return out.replace(SEND_MARK, "").trim();
  };
}

// ── Nova voice (TTS bench): the operator picks the reply mode (text / voice /
// both) and the engine from the 🔊 menu; synthesis goes through the local
// omega-ttsd gateway (Pocket/Chatterbox/Kokoro/Piper kept in RAM + ElevenLabs
// proxy), which returns Telegram-ready OGG/Opus. The text reply NEVER waits on
// synthesis — voice is fire-and-forget on top of it.
const TTSD = `http://127.0.0.1:${process.env.OMEGA_TTSD_PORT || 8765}`;
const VOICE_PREFS_FILE = `${OMEGA_DIR}/state/nova-voice.json`;
// `voice` selects a specific voice INSIDE the engine (catalog name, cloning wav
// path, or ElevenLabs voice_id) — set by «voix N» from the casting bench
// (tools/tts/casting.py numbers every sample; the resolved map lives in
// ~/.omega/tts/casting-manifest.json). Empty = the engine's default voice.
type VoicePrefs = { mode: "text" | "voice" | "both"; engine: string; voice?: string; voiceLabel?: string; voiceParams?: Record<string, number> };
function voicePrefs(): VoicePrefs {
  try { return { mode: "text", engine: "pocket", ...JSON.parse(readFileSync(VOICE_PREFS_FILE, "utf8")) }; }
  catch { return { mode: "text", engine: "pocket" }; }
}
function saveVoicePrefs(p: VoicePrefs) {
  Bun.spawnSync(["mkdir", "-p", `${OMEGA_DIR}/state`]);
  try { writeFileSync(VOICE_PREFS_FILE, JSON.stringify(p)); } catch {}
}
// Markdown reads terribly aloud — strip it (and cap length: a voice note is a
// note, not an audiobook; the full text is always in the chat/history anyway).
function ttsSpeakable(md: string): string {
  return md
    .replace(/```[\s\S]*?```/g, " … ")
    .replace(/`([^`]*)`/g, "$1")
    .replace(/!\[[^\]]*\]\([^)]*\)/g, "")
    .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/^#{1,6}\s+/gm, "")
    .replace(/[*_~]{1,3}([^*_~\n]+)[*_~]{1,3}/g, "$1")
    .replace(/^\s*[-•]\s+/gm, "")
    .replace(/\s+/g, " ")
    .trim().slice(0, 2500);
}
async function synthVoice(engine: string, text: string, voice = "", params?: Record<string, number>): Promise<Uint8Array | { error: string }> {
  try {
    const r = await fetch(`${TTSD}/tts`, {
      method: "POST", headers: { "content-type": "application/json" },
      body: JSON.stringify({ engine, text: ttsSpeakable(text), voice, params: params || {} }),
      signal: AbortSignal.timeout(420_000), // chatterbox on CPU is slow by design
    });
    if (!r.ok) { const j: any = await r.json().catch(() => ({})); return { error: j?.error || `HTTP ${r.status}` }; }
    return new Uint8Array(await r.arrayBuffer());
  } catch (e: any) { return { error: /timeout|abort/i.test(String(e)) ? "timeout de synthèse" : String(e?.message || e) }; }
}
// sendVoice needs multipart (binary upload) — the JSON `tg()` funnel can't carry it.
async function sendVoiceNote(chat: number, ogg: Uint8Array, thread?: number, caption?: string): Promise<any> {
  const fd = new FormData();
  fd.append("chat_id", String(chat));
  if (thread) fd.append("message_thread_id", String(thread));
  if (caption) fd.append("caption", caption.slice(0, 1024));
  fd.append("voice", new Blob([ogg], { type: "audio/ogg" }), "nova.ogg");
  try { return await (await fetch(`${API}/sendVoice`, { method: "POST", body: fd })).json(); }
  catch (e: any) { return { ok: false, description: String(e?.message || e) }; }
}
// Deliver a real file Nova produced on the VPS (PDF, image, video…) to the chat.
// Telegram method picked by extension, so a [[SEND: /path]] marker in her reply
// actually puts the file in the operator's chat (was: she only ever sent text).
async function sendFileToChat(chat: number, path: string, thread?: number, caption?: string): Promise<boolean> {
  try {
    const buf = await Bun.file(path).arrayBuffer();
    const name = path.split("/").pop() || "file";
    const ext = (name.split(".").pop() || "").toLowerCase();
    const method = ["jpg", "jpeg", "png", "webp", "gif"].includes(ext) ? "sendPhoto"
      : ["mp4", "mov", "webm"].includes(ext) ? "sendVideo"
      : ["ogg", "mp3", "m4a", "wav"].includes(ext) ? "sendAudio"
      : "sendDocument";
    const field = method === "sendPhoto" ? "photo" : method === "sendVideo" ? "video" : method === "sendAudio" ? "audio" : "document";
    const fd = new FormData();
    fd.append("chat_id", String(chat));
    if (thread) fd.append("message_thread_id", String(thread));
    if (caption) fd.append("caption", caption.slice(0, 1024));
    fd.append(field, new Blob([buf]), name);
    const r: any = await (await fetch(`${API}/${method}`, { method: "POST", body: fd })).json();
    return !!r?.ok;
  } catch { return false; }
}
// Nova attaches files by emitting [[SEND: /abs/path | optional caption]] in her reply.
const SEND_MARK = /\[\[SEND:\s*([^\]|]+?)(?:\s*\|\s*([^\]]+))?\]\]/g;
// Voice layer over a finished companion reply. mode "both": voice note follows
// the text. mode "voice": the placeholder shows a teaser, and is deleted once
// the note lands (synthesis failed → the full text is restored: never lose an
// answer to a TTS hiccup).
async function speakReply(chat: number, thread: number | undefined, out: string, phId?: number) {
  const vp = voicePrefs();
  if (vp.mode === "text") return;
  const r = await synthVoice(vp.engine, out, vp.voice || "", vp.voiceParams);
  if (r instanceof Uint8Array) {
    const sent = await sendVoiceNote(chat, r, thread);
    if (vp.mode === "voice" && sent?.ok && phId) await tg("deleteMessage", { chat_id: chat, message_id: phId });
  } else {
    const warn = `⚠️ Synthèse vocale échouée (<b>${esc(vp.engine)}</b>) : <i>${esc(r.error).slice(0, 200)}</i>`;
    if (vp.mode === "voice" && phId) { let html; try { html = mdToHtml(out); } catch { html = out; } await edit(chat, phId, html, undefined, thread); }
    await send(chat, warn, undefined, thread);
  }
}
// Persist the ElevenLabs key pasted in chat («clé elevenlabs: xxx») into
// provisioning/services.env — the daemon re-reads it on every request, so the
// engine turns on with no restart.
function saveElevenLabsKey(key: string) {
  const path = `${OMEGA_DIR}/provisioning/services.env`;
  Bun.spawnSync(["mkdir", "-p", `${OMEGA_DIR}/provisioning`]);
  let txt = ""; try { txt = readFileSync(path, "utf8"); } catch {}
  const line = `export ELEVENLABS_API_KEY="${key}"`;
  txt = /^\s*(export\s+)?ELEVENLABS_API_KEY\s*=/m.test(txt)
    ? txt.replace(/^\s*(export\s+)?ELEVENLABS_API_KEY\s*=.*$/m, line)
    : `${txt.trimEnd()}\n${line}\n`;
  writeFileSync(path, txt, { mode: 0o600 });
}

// Provision a per-agent Telegram bot as its own background service (AGENT MODE):
// systemd user unit on Linux, launchd LaunchAgent on macOS (no systemd there —
// mirrors install.sh's os.omega.tg-bot plist, so agent-bots provision on a Mac
// too). The token lives only in agent-bots.json (mode 600), never in the unit.
// Both units resolve bun at START time via `sh -c` (same chain as install.sh's
// tg-bot-launch.sh: PATH, ~/.bun/bin, /opt/homebrew/bin, /usr/local/bin) —
// freezing the provision-time bun path under KeepAlive/Restart=always meant one
// bun reinstall (new path) → eternal respawn throttle (the F14 class).
// logFile (darwin only — systemd output goes to the journal, which rotates
// itself): launchd never rotates StandardOut/ErrPath, so the preamble size-caps
// the agent log at 5MB via copytruncate (cp + truncate-in-place, NOT mv —
// launchd opens the log fd before sh runs, and mv would re-point it at .1).
const AGENT_BUN_RESOLVE = (script: string, logFile?: string) =>
  `${logFile ? `if [ -f "${logFile}" ] && [ "$(wc -c < "${logFile}")" -gt 5242880 ]; then cp "${logFile}" "${logFile}.1" && : > "${logFile}"; fi; ` : ""}for c in "$(command -v bun 2>/dev/null)" "$HOME/.bun/bin/bun" /opt/homebrew/bin/bun /usr/local/bin/bun; do [ -n "$c" ] && [ -x "$c" ] && exec "$c" ${script}; done; echo "tg-agent: bun not found (PATH, ~/.bun/bin, /opt/homebrew/bin, /usr/local/bin)" >&2; exit 127`;
const agentSvcLabel = (id: string) => `os.omega.tg-agent-${id}`;
function spawnAgentBot(agentId: string): string {
  try {
    if (process.platform === "darwin") {
      const uid = process.getuid?.() ?? 501;
      const label = agentSvcLabel(agentId);
      const laDir = `${homedir()}/Library/LaunchAgents`;
      const plist = `${laDir}/${label}.plist`;
      Bun.spawnSync(["mkdir", "-p", laDir, `${OMEGA_DIR}/logs`]);
      writeFileSync(plist, `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key><string>${label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>/bin/sh</string>
        <string>-c</string>
        <string>${AGENT_BUN_RESOLVE(`"${OMEGA_DIR}/telegram-bot/omega-tg-bot.ts"`, `${OMEGA_DIR}/logs/tg-agent-${agentId}.log`).replace(/&/g, "&amp;")}</string>
    </array>
    <key>WorkingDirectory</key><string>${OMEGA_DIR}/telegram-bot</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>OMEGA_DIR</key><string>${OMEGA_DIR}</string>
        <key>OMEGA_AGENT_BOT</key><string>${agentId}</string>
        <key>PATH</key><string>${homedir()}/.local/bin:${homedir()}/.bun/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin</string>
    </dict>
    <key>RunAtLoad</key><true/>
    <key>KeepAlive</key><true/>
    <key>StandardOutPath</key><string>${OMEGA_DIR}/logs/tg-agent-${agentId}.log</string>
    <key>StandardErrorPath</key><string>${OMEGA_DIR}/logs/tg-agent-${agentId}.log</string>
</dict>
</plist>
`);
      // Idempotent like `enable --now`: bootstrap only when not loaded, then
      // kickstart (no -k → starts if stopped, leaves a running agent alone).
      if (Bun.spawnSync(["launchctl", "print", `gui/${uid}/${label}`]).exitCode !== 0) {
        const boot = Bun.spawnSync(["launchctl", "bootstrap", `gui/${uid}`, plist]);
        if (boot.exitCode !== 0) return boot.stderr.toString().trim() || "launchctl bootstrap failed";
      }
      Bun.spawnSync(["launchctl", "kickstart", `gui/${uid}/${label}`]);
      return "ok";
    }
    const sd = `${homedir()}/.config/systemd/user`;
    Bun.spawnSync(["mkdir", "-p", sd]);
    writeFileSync(`${sd}/omega-tg-agent-${agentId}.service`, `[Unit]
Description=OmegaOS agent bot — ${agentId}
After=network-online.target

[Service]
Type=simple
Environment=OMEGA_DIR=%h/.omega
Environment=OMEGA_AGENT_BOT=${agentId}
WorkingDirectory=%h/.omega/telegram-bot
ExecStart=/bin/sh -c '${AGENT_BUN_RESOLVE('"%h/.omega/telegram-bot/omega-tg-bot.ts"').replace(/\$/g, "$$$$")}'
Restart=always
RestartSec=3

[Install]
WantedBy=default.target
`);
    Bun.spawnSync(["systemctl", "--user", "daemon-reload"]);
    const r = Bun.spawnSync(["systemctl", "--user", "enable", "--now", `omega-tg-agent-${agentId}.service`]);
    return r.exitCode === 0 ? "ok" : (r.stderr.toString().trim() || "systemctl failed");
  } catch (e: any) { return e?.message || "spawn failed"; }
}
// Stop + remove an agent-bot service on either platform (launchd / systemd).
function teardownAgentBot(id: string) {
  if (process.platform === "darwin") {
    const uid = process.getuid?.() ?? 501;
    Bun.spawnSync(["launchctl", "bootout", `gui/${uid}/${agentSvcLabel(id)}`]);
    Bun.spawnSync(["rm", "-f", `${homedir()}/Library/LaunchAgents/${agentSvcLabel(id)}.plist`]);
  } else {
    Bun.spawnSync(["systemctl", "--user", "disable", "--now", `omega-tg-agent-${id}.service`]);
  }
}

// ── Dispatch to a REAL oracle session (the canonical path for project work). A
// message from a project topic / a project agent-bot / the "Talk to the oracle"
// button is a MISSION for the VPS, not a chat: it spawns `omega dispatch <project>`
// → a visible Claude Code oracle session (its own mission; it delegates to dynamic
// workflows / workers / audit-review). The Monitor watches done.json and relays the
// result. The bot NEVER does project work itself (no headless brain).
// `gateSt` = the provisional status already rendered during an L4 gate hold, so the
// 12s report poll re-edits the card only when the verdict actually changes.
type Watch = { chat: number; thread?: number; mission: string; ts: number; oracle: string; project: string; msgId?: number; resends?: number; gateSt?: string };
// Normalize an oracle id for comparison: the live progress/watch name carries the
// "oracle-" prefix (oracle-dentistrygpt-8) but done.json stores the bare key
// (dentistrygpt-8). Compare prefix-insensitively so reports match the RIGHT card.
const normOracle = (s: string) => String(s || "").toLowerCase().replace(/^oracle-/, "");
const watching: Watch[] = [];
const reported = new Set<string>();
const progressPath = (oracle: string) => `${OMEGA_DIR}/state/${oracle}.progress.json`;
// Persist the live card's message id back to the progress file so a bot RESTART
// (systemd Restart=always) can re-attach to the SAME card instead of orphaning it.
function persistMsgId(oracle: string, msgId: number) {
  try { const j = JSON.parse(readFileSync(progressPath(oracle), "utf8")); j.msgId = msgId; writeFileSync(progressPath(oracle), JSON.stringify(j)); } catch {}
}
// Re-attach watching[] to the live progress cards THIS bot owns (p.bot === BOT_ID)
// that have not finished yet (no done.json). watching[] is in-memory, so without
// this a restart freezes the card AND drops the final report. Scoped by BOT_ID so a
// per-project agent bot never tries to edit another bot's card (a different chat) —
// which, hitting a missing message, would re-send a stray message. Skips finished
// (done.json present → already reported / handled by omega-done-notify) and stale
// (>24h, abandoned) cards so old cards are never resurrected into new messages.
function rehydrateWatching() {
  let files: string[] = [];
  try { files = Bun.spawnSync(["bash", "-lc", `ls ${OMEGA_DIR}/state/oracle-*.progress.json 2>/dev/null`]).stdout.toString().trim().split("\n").filter(Boolean); } catch {}
  let n = 0;
  for (const f of files) {
    try {
      const p = JSON.parse(readFileSync(f, "utf8"));
      if (!p?.msgId || p.bot !== BOT_ID || !p.oracle) continue;
      // Finished → skip, EXCEPT a gate-held provisional verdict (pending +
      // gate_pending): that watch must survive a restart so the L4 upgrade to
      // done_clean still re-edits the SAME card (pollReports' gate-hold path).
      const df = `${OMEGA_DIR}/state/${p.oracle}.done.json`;
      if (existsSync(df)) {
        let hold = false;
        try { const dd = JSON.parse(readFileSync(df, "utf8")); hold = dd.status === "pending" && dd.gate_pending === true; } catch {}
        if (!hold) continue;
      }
      if (Date.now() - statSync(f).mtimeMs > 24 * 3600_000) continue;
      if (watching.some(w => w.oracle === p.oracle)) continue;
      watching.push({ chat: p.chat, thread: p.thread ?? undefined, mission: p.mission || "", ts: 0, oracle: p.oracle, project: p.project || "", msgId: p.msgId });
      n++;
    } catch {}
  }
  if (n) console.log(`rehydrated ${n} live progress card(s)`);
}
// Dispatch a real oracle session AND post a live progress card (edited in place by
// pollProgress as the oracle calls `omega progress`, finalized into the report by
// pollReports). `extra` is dispatched to the oracle (history/reply context) but NOT
// shown on the card. Returns "" because the card is sent here directly.
async function dispatchToOracle(project: string, mission: string, chat: number, thread: number | undefined, extra = ""): Promise<string> {
  const out = await omega(["dispatch", project, `${extra}${mission}`]);
  const m = out.match(/Oracle dispatched:?\s*(oracle-[A-Za-z0-9._-]+)/) || out.match(/oracle=(oracle-[A-Za-z0-9._-]+)/);
  if (!m) return card(`DISPATCH ${project.toUpperCase()} — FAILED`, ` ❌ <pre>${esc(out).slice(0, 600)}</pre>`);
  const oracle = m[1];
  const sent = await send(chat, progressCard(project, oracle, mission, null), undefined, thread);
  const msgId = sent?.result?.message_id as number | undefined;
  watching.push({ chat, thread, mission, ts: Date.now(), oracle, project, msgId });
  try { writeFileSync(progressPath(oracle), JSON.stringify({ chat, thread: thread ?? null, msgId: msgId ?? null, bot: BOT_ID, project, oracle, mission, done: 0, total: 0, tasks: [] })); } catch {}
  return ""; // card already sent
}

// ── Fragment aggregation: collect the messages that belong to ONE operator ask ──
// Telegram delivers an album as one message PER photo (caption on a single one)
// and forces a prompt over the 1024-char caption limit into a separate text
// message. Dispatching per message spawned N oracle missions for one ask (a
// 2-photo album = 2 parallel oracles). Mission-bound fragments are buffered per
// chat+thread and flushed AGGREGATE_MS after the LAST one, as a single mission
// carrying every text + every image. Restart loss window = AGGREGATE_MS, acceptable.
const AGGREGATE_MS = 8000;
type Fragments = { texts: string[]; files: string[]; quoted: string; lastMsgId: number; timer?: ReturnType<typeof setTimeout> };
const fragments = new Map<string, Fragments>();
function queueMissionFragment(chat: number, thread: number | undefined, text: string, file: string, replyTo: string, msgId: number, fixedProject?: string) {
  const key = `${chat}:${thread ?? 0}:${fixedProject || ""}`;
  const f = fragments.get(key) || { texts: [], files: [], quoted: "", lastMsgId: msgId };
  if (text) f.texts.push(text);
  if (file) f.files.push(file);
  if (replyTo && !f.quoted) f.quoted = replyTo;
  f.lastMsgId = msgId;
  react(chat, msgId, "👀"); // received & buffering — the 🚀 lands at flush time
  if (f.timer) clearTimeout(f.timer);
  f.timer = setTimeout(() => { fragments.delete(key); flushMission(chat, thread, f, fixedProject).catch((e: any) => console.error("flushMission:", e?.message || e)); }, AGGREGATE_MS);
  fragments.set(key, f);
}
async function flushMission(chat: number, thread: number | undefined, f: Fragments, fixedProject?: string) {
  let text = f.texts.join("\n\n");
  if (f.files.length) {
    const many = f.files.length > 1;
    text = `${text || `Process the attached file${many ? "s" : ""} and act on ${many ? "them" : "it"}.`}\n\n## Attached file${many ? "s" : ""}\n${f.files.map((p) => `- ${p}`).join("\n")}\nOpen ${many ? "them" : "it"} with the Read tool (it reads PDFs, text, code and images) and use ${many ? "them" : "it"} as part of this mission.`;
  }
  if (!text) return;
  // AGENT MODE: fixed project — direct dispatch, no topic routing / history.
  if (fixedProject) {
    react(chat, f.lastMsgId, "🚀");
    const r = await dispatchToOracle(fixedProject, text, chat, thread);
    if (r) await send(chat, r, undefined, thread);
    return;
  }
  // Hub mode: route by topic at FLUSH time (same rules as before aggregation):
  // project topic → that project's oracle; atlas topic / DM → the Atlas brain.
  const g = loadGroups();
  const topicName = thread ? g.topics?.[String(thread)] : undefined;
  const proj = topicName && !isReserved(topicName) ? topicName : undefined;
  const ctx = histContext(chat, thread);
  const quoted = f.quoted ? `## The operator is replying to this message:\n«${f.quoted}»\n\n` : "";
  const extra = `${ctx}${quoted}`;
  histAppend(chat, thread, "operator", f.quoted ? `(in reply to: ${f.quoted.slice(0, 120)}) ${text}` : text, proj || "atlas");
  if (proj) { react(chat, f.lastMsgId, "🚀"); const r = await dispatchToOracle(proj, text, chat, thread, extra); if (r) await send(chat, r, undefined, thread); }
  else await brainReply(chat, f.lastMsgId, thread, `${extra}${text}`);
}
// Live progress: read each tracked oracle's progress.json and EDIT its card.
async function pollProgress() {
  for (const w of watching) {
    if (!w.msgId) continue;
    let p: any = null;
    try { p = JSON.parse(readFileSync(progressPath(w.oracle), "utf8")); } catch {}
    if (!p) continue;
    const r = await edit(w.chat, w.msgId, progressCard(w.project, w.oracle, w.mission, p), undefined, w.thread);
    // If the card was gone and edit() had to re-send (its edit→send fallback), it
    // returns the NEW message. Adopt that id so the next poll edits the new card in
    // place — instead of editing the dead id, failing, and re-sending a fresh
    // message every 6s (the "plusieurs messages au lieu d'un" bug). Persist it so a
    // restart re-attaches to it too.
    const newId = r?.result?.message_id as number | undefined;
    if (newId && newId !== w.msgId) {
      // edit() fell back to send() → a NEW message was posted. Adopt it so the next
      // poll edits the new card in place. FLOOD CEILING: if a card keeps vanishing
      // (deleted/uneditable) we must NOT keep posting a fresh message every 6s — that
      // is the multi-message flood. After 3 resends, stop touching this card (the
      // final report still posts fresh via pollReports' send()).
      w.msgId = newId; persistMsgId(w.oracle, newId);
      w.resends = (w.resends || 0) + 1;
      if (w.resends >= 3) {
        console.error(`pollProgress: ${w.oracle} card kept disappearing (${w.resends} resends) — stopping live updates (flood guard)`);
        w.msgId = undefined;
      }
    } else if (r?.ok) {
      w.resends = 0; // healthy in-place edit (or harmless no-op) → reset the guard
    }
    // else: a transient failure (rate-limit / network) — edit() neither edited in
    // place nor resent anything. Do NOT reset the guard and do NOT post: just retry
    // on the next tick once the rate-limit window clears. (Resetting here is what let
    // the guard never trip during a 429 storm — the exact flood it must stop.)
  }
}
// Monitor: scan ~/.omega/state/oracle-*.done.json, relay each finished dispatch back
// to the chat/topic that launched it.
async function pollReports() {
  if (!watching.length) return;
  let files: string[] = [];
  try { files = Bun.spawnSync(["bash", "-lc", `ls ${OMEGA_DIR}/state/oracle-*.done.json 2>/dev/null`]).stdout.toString().trim().split("\n").filter(Boolean); } catch {}
  for (const f of files) {
    if (reported.has(f)) continue;
    let d: any; try { d = JSON.parse(readFileSync(f, "utf8")); } catch { continue; }
    const finishedTs = d.finished_at ? Date.parse(d.finished_at) : Date.now();
    // Match the EXACT oracle (prefix-insensitive). The old substring `includes(project)`
    // fuzzy-match wrongly hit EVERY card of the project — so a report could overwrite +
    // splice the wrong oracle's live card, freezing a still-running oracle's card.
    const idx = watching.findIndex(w => finishedTs >= w.ts - 5000 && normOracle(w.oracle) === normOracle(d.oracle));
    if (idx < 0) continue;
    const w = watching[idx];
    const st = d.status || "done";
    // L4 gate hold: `pending` + gate_pending=true is a PROVISIONAL verdict — `omega
    // progress` / patrol rewrite this very done.json to done_clean once the plan
    // hits 100%. Finalizing here froze the card on "mission incomplete" forever
    // (watch spliced, file in `reported`), so the upgrade never reached the card.
    // Render the provisional state once, keep the watch alive, and only finalize
    // when the verdict can no longer change.
    const gateHold = st === "pending" && d.gate_pending === true;
    if (gateHold) {
      if (w.gateSt === st) continue; // provisional state already on the card
      w.gateSt = st;
    } else { reported.add(f); watching.splice(idx, 1); }
    // Symbol aesthetic — no emoji. Report v3: status glyph + full bar + summary
    // (long → expandable) + subtle footer. Edits the live progress card in place.
    const sym = st === "done_clean" ? "✓" : st === "failed" ? "✗" : st === "blocked" ? "‖" : st === "pending" ? "…" : "▪";
    const label = st === "done_clean" ? "mission complete" : st === "failed" ? "mission failed" : st === "blocked" ? "mission blocked" : st === "pending" ? "mission incomplete" : "finished";
    const sum = esc(String(d.summary || "(no summary)")).slice(0, 2600);
    const body = (String(d.summary || "").length > 280) ? `<blockquote expandable>${sum}</blockquote>` : sum;
    const dur = d.duration_secs ? ` · ${Math.floor(d.duration_secs / 60)}m${String(d.duration_secs % 60).padStart(2, "0")}s` : "";
    const commit = d.ship?.commit ? ` · <code>${esc(String(d.ship.commit).slice(0, 12))}</code>` : "";
    const deploy = d.ship?.deploy_url ? `\n${esc(d.ship.deploy_url)}` : "";
    const pending = (Array.isArray(d.pending_actions) && d.pending_actions.length) ? `\n\n<b>Remaining:</b> ${esc(d.pending_actions.join(" · ")).slice(0, 600)}` : "";
    // Pull the final task checklist from the progress file (before it's removed).
    let plist: PTask[] | undefined; let pdone = 0, ptot = 0;
    try { const pj = JSON.parse(readFileSync(`${OMEGA_DIR}/state/${w.oracle}.progress.json`, "utf8")); plist = pj.tasks; pdone = pj.done || 0; ptot = pj.total || 0; } catch {}
    const checklist = taskList(plist);
    const barLine = ptot > 0 ? `<code>${bar(Math.round((pdone / ptot) * 100))}</code> ${pdone}/${ptot}` : `<code>${bar(100)}</code> 100%`;
    const report = `${sym} <b>${esc(d.project || w.project)}</b> · ${label}\n${barLine}${checklist}\n\n${body}${pending}${deploy}\n\n<i>${esc(d.oracle || w.oracle)}${dur}</i>${commit}`;
    // 4 intelligent buttons on the report: re-dispatch / reply to this oracle, and
    // simple git (only shown when the project resolves to a local repo).
    const pj = d.project || w.project;
    const rk = kb([
      [{ text: "▸ Relancer", callback_data: `proj:oracle:${pj}`.slice(0, 64) }, { text: "↩ Répondre", callback_data: `rep:reply:${pj}`.slice(0, 64) }],
      ...(repoPath(pj) ? [[{ text: "↓ Pull", callback_data: `git:pull:${pj}`.slice(0, 64) }, { text: "↑ Push", callback_data: `git:push:${pj}`.slice(0, 64) }]] : []),
    ]);
    if (w.msgId) await edit(w.chat, w.msgId, report, rk, w.thread);
    else await send(w.chat, report, rk, w.thread);
    // Persist the oracle's report into the conversation history (+ MC mirror) so the
    // next turn — to Atlas or the oracle — has the full thread.
    histAppend(w.chat, w.thread, "assistant", `[${d.project || w.project}] ${d.summary || label}`, String(d.project || w.project));
    // The marker records WHICH status was notified — omega-done-notify content-keys
    // its re-arm on the marker body, and an empty marker matches ANY later status
    // (it ate the pending→done_clean upgrade notification).
    try { writeFileSync(`${f}.notified`, st); } catch {}
    // During a gate hold keep progress.json: pollProgress keeps the card live and
    // the final (upgraded) report still needs its checklist.
    if (!gateHold) try { Bun.spawnSync(["rm", "-f", `${OMEGA_DIR}/state/${w.oracle}.progress.json`]); } catch {}
  }
}

// ── brain UX: instant ack + live "thinking" placeholder, then formatted reply ──
async function react(chat: number, msgId: number, emoji: string) {
  try { await tg("setMessageReaction", { chat_id: chat, message_id: msgId, reaction: [{ type: "emoji", emoji }] }); } catch {}
}
// One funnel for every brain call: 🤔 reaction (seen it) + a live placeholder, run
// the Master, then edit the placeholder with HTML-formatted output + ✅ reaction.
async function brainReply(chat: number, userMsgId: number, thread: number | undefined, prompt: string, brain: (t: string) => Promise<string> = master, label = "Atlas", speak = false) {
  react(chat, userMsgId, "🤔");
  await tg("sendChatAction", { chat_id: chat, action: "typing", message_thread_id: thread });
  const ph = await tg("sendMessage", { chat_id: chat, parse_mode: "HTML", message_thread_id: thread, text: `🧠 <i>${label} thinking…</i>` });
  const phId = ph?.result?.message_id as number | undefined;
  // LIVE "really working" feedback: keep the typing… indicator alive (Telegram
  // expires it after ~5s) AND animate the placeholder (rotating glyph + growing
  // dots + elapsed seconds) every 3s, until the brain answers. `done` guards
  // against a late in-flight beat overwriting the final reply.
  const frames = ["🧠", "💭", "⚙️", "🔎", "✨"];
  const t0 = Date.now();
  let tick = 0, done = false;
  const beat = setInterval(() => {
    if (done) return;
    tg("sendChatAction", { chat_id: chat, action: "typing", message_thread_id: thread });
    if (phId) {
      const secs = Math.round((Date.now() - t0) / 1000);
      const dots = ".".repeat((tick % 3) + 1);
      edit(chat, phId, `${frames[tick % frames.length]} <i>${label} thinking${dots}</i>  <code>${secs}s</code>`, undefined, thread);
    }
    tick++;
  }, 3000);
  const stop = () => { done = true; clearInterval(beat); };
  // Fire-and-forget BY DESIGN: the poll loop must never block on a 900s brain run.
  // The chain below always lands a final message (success OR error) for the operator.
  brain(prompt)
    .then(async out => {
      stop();
      histAppend(chat, thread, "assistant", out, label.toLowerCase()); // persist the reply (+ MC mirror)
      let html: string; try { html = mdToHtml(out); } catch { html = out; } // bad markup → raw text
      // 🔊 mode "voice": teaser placeholder, deleted once the note lands (speakReply
      // restores the text if synthesis fails). Other modes: text lands first, always.
      const voiceOnly = speak && voicePrefs().mode === "voice";
      const r = voiceOnly && phId
        ? await edit(chat, phId, "🎙️ <i>réponse vocale en préparation…</i>", undefined, thread)
        : await (phId ? edit(chat, phId, html, undefined, thread) : send(chat, html, undefined, thread));
      if (speak) speakReply(chat, thread, out, phId).catch(() => {});
      return r;
    })
    .then(() => react(chat, userMsgId, "✅"))
    .catch(async () => {
      stop();
      react(chat, userMsgId, "⚠️");
      const m = "⚠️ AISB hit an error — try again.";
      if (phId) await edit(chat, phId, m, undefined, thread); else await send(chat, m, undefined, thread);
    });
}
const back = (to = "menu"): Btn => ({ text: "« Back", callback_data: `nav:${to}` });

// ── Claude account / OAuth (login flow, switch, email, usage) ────────────────
// Wraps the shipped headless-VPS OAuth helper. `generate` prints an auth URL the
// operator opens; pasting the callback code back runs `exchange` → fresh tokens.
const OAUTH = `${OMEGA_DIR}/bin/claude-oauth.sh`;
async function oauth(args: string[]): Promise<string> {
  try { const r = await $`bash ${OAUTH} ${args}`.quiet().nothrow(); return (r.stdout.toString() + r.stderr.toString()).trim(); }
  catch (e: any) { return `error: ${e?.message || e}`; }
}
// `omega claude-login[-code]` prints tracing logs then a single JSON line
// (`{"ok":true,"url":…}` / `{"ok":true,"email":…,"expires_min":…}`). Pull the
// last well-formed {…} line out of the combined stdout+stderr.
function extractJson(out: string): any {
  const lines = out.split("\n").map(l => l.trim()).filter(Boolean);
  for (let i = lines.length - 1; i >= 0; i--)
    if (lines[i].startsWith("{") && lines[i].endsWith("}")) { try { return JSON.parse(lines[i]); } catch {} }
  return null;
}
// Transient per-user flows awaiting a typed reply (login code paste, new-project
// brief). PERSISTED to disk: the systemd service can restart at any moment, and an
// in-memory-only Map dropped the operator's pasted OAuth code into the brain mid-login
// (the #1 cause of "login didn't work"). A 15-min TTL means a stale flow expires and
// never hijacks an ordinary message after a long gap.
const PENDING_FILE = `${OMEGA_DIR}/state/tg-pending.json`;
const PENDING_TTL = 15 * 60 * 1000;
type Pending = { kind: "login-code" | "new-project" | "add-project" | "import-project" | "tg-link" | "oracle-prompt" | "kairos-field" | "kairos-confirm" | "kairos-day" | "kairos-cap" | "zernio-post"; ts: number; arg?: string };
const pending = new Map<number, Pending>();
function savePending() { try { writeFileSync(PENDING_FILE, JSON.stringify([...pending.entries()])); } catch {} }
function loadPending() {
  try {
    const now = Date.now();
    for (const [id, p] of JSON.parse(readFileSync(PENDING_FILE, "utf8")) as [number, Pending][])
      if (now - p.ts < PENDING_TTL) pending.set(id, p);
  } catch {}
}
function setPending(id: number, kind: Pending["kind"], arg?: string) { pending.set(id, { kind, ts: Date.now(), arg }); savePending(); }
function clearPending(id: number) { if (pending.delete(id)) savePending(); }
function getPending(id: number): Pending | undefined {
  const p = pending.get(id);
  if (p && Date.now() - p.ts >= PENDING_TTL) { clearPending(id); return undefined; }
  return p;
}

async function accountStatus(): Promise<string> {
  const raw = await oauth(["status"]);
  const email = raw.match(/"email"\s*:\s*"([^"]+)"/)?.[1] || "?";
  const sub = raw.match(/"subscriptionType"\s*:\s*"([^"]+)"/)?.[1] || "?";
  let token = "?";
  try { const c = JSON.parse(await oauth(["check"])); token = c.valid ? `valid (${c.remaining_min} min left)` : "⚠️ EXPIRED — tap “Login”"; } catch {}
  const usage = await omega(["usage"]);
  const tokenOk = /valid/i.test(token);
  return card("CLAUDE ACCOUNT (AISB)",
    ` 📧 ${esc(email)}\n 🎫 plan: ${esc(sub)}\n ${tokenOk ? "🟢" : "🔴"} token: ${esc(token)}`,
    `📊 <b>TOKEN USAGE</b>\n<pre>${esc(usage).slice(0, 1500)}</pre>`);
}

async function serviceAccounts(): Promise<string> {
  const env = readKV(`${OMEGA_DIR}/provisioning/services.env`, /^\s*export\s+([A-Z_]+)\s*=\s*"?([^"]*)"?\s*$/);
  const row = (label: string, key: string) => `${env[key] ? "✅" : "❌"} ${label}${env[key] ? "" : " — token missing"}`;
  const staticTable = `<b>👤 Service accounts (provisioning)</b>\n` +
    `${row("Vercel", "VERCEL_TOKEN")}\n${row("Convex", "CONVEX_TEAM_TOKEN")}\n${row("GitHub", "GITHUB_TOKEN")}\n` +
    `${row("Stripe", "STRIPE_SECRET_KEY")}\nClerk: ${esc(env.CLERK_PROVISION_MODE || "?")}\n\n` +
    `<i>The ❌ ones need your token. Fill them via the Provisioning wizard (TUI) or edit ~/.omega/provisioning/services.env.</i>`;
  // Live probe of which accounts actually authenticate, when the CLI supports it
  // (graceful no-op on older binaries without `omega provision verify`).
  const probe = await omega(["provision", "verify", "default"]);
  const hasProbe = probe && !/error|unrecognized|unexpected argument|USAGE:|no output|not found/i.test(probe);
  return hasProbe ? `<b>🔎 Live account verification</b>\n<pre>${esc(probe).slice(0, 1400)}</pre>\n\n${staticTable}` : staticTable;
}

// Login / Re-auth — drives the real `claude /login` via the shared `omega
// claude-login` engine (spawns the visible `aisb-reauth` session, captures the
// authorize URL). UX: a "⏳ en cours" card while the URL generates (~15s — real
// OAuth, not instant), then the SAME message is replaced by a designed card with
// the link as a tappable button. Pasting the callback code back runs
// `omega claude-login-code`, which writes fresh creds to the SHARED store.
const TITLE_LOGIN = (s: boolean) => (s ? "SWITCH ACCOUNT" : "LOGIN / RE-AUTH");
async function startLogin(chat: number, msgId: number, from: number, switchAcct: boolean) {
  // 1) Waiting card (the wait is normal — house OAuth, browser-less).
  await edit(chat, msgId, card(TITLE_LOGIN(switchAcct),
    " ⏳ <b>Connecting…</b>\n Generating the Claude authorization link.\n <i>~15 s — it's the OAuth auth, this is normal.</i>"),
    kb([[back("account")]]));
  // 2) Drive the engine, pull the URL out of its JSON.
  const j = extractJson(await omega(["claude-login"]));
  const url: string = j?.url || "";
  if (!j?.ok || !/^https?:\/\//.test(url))
    return edit(chat, msgId, card(TITLE_LOGIN(switchAcct),
      ` ❌ <b>Link not generated.</b>\n <i>Try again in a moment.</i>`),
      kb([[{ text: "🔄 Retry", callback_data: "acct:login" }], [back("account")]]));
  // 3) Replace the waiting card with the designed link card + button.
  setPending(from, "login-code");
  await edit(chat, msgId, card(TITLE_LOGIN(switchAcct),
    ` 🔗 <b>1.</b> Open the link and authorize${switchAcct ? " <b>with the other account</b>" : " with your Max account"}.\n` +
    ` 🔑 <b>2.</b> Copy the <b>code</b> from the callback page and <b>paste it here</b> (next message).`,
    "<i>One login for all of OmegaOS — the credential is shared across every session.</i>"),
    kb([[{ text: "🔐 Open & authorize", url }], [{ text: "✖ Cancel", callback_data: "acct:cancel" }]]));
}

function dashboardURL(): { url: string; pw: string } {
  const mc = readKV(MC_ENV, /^([A-Z_]+)=(.*)$/);
  const host = mc.HOSTNAME?.trim();
  const ip = (process.env.OMEGA_PUBLIC_IP || "").trim();
  // Only return a button-able URL when we actually have a host/IP (never http://:8080).
  const url = host ? `https://${host}` : (ip ? `http://${ip}:8080` : "");
  return { url, pw: mc.OMEGA_MC_WEB_PASSWORD || "" };
}
async function resolvePublicIP(): Promise<void> {
  if (process.env.OMEGA_PUBLIC_IP) return;
  for (const u of ["https://ifconfig.me/ip", "https://icanhazip.com", "https://api.ipify.org"]) {
    try { const ip = (await (await fetch(u, { signal: AbortSignal.timeout(5000) })).text()).trim(); if (/^\d+\.\d+\.\d+\.\d+$/.test(ip)) { process.env.OMEGA_PUBLIC_IP = ip; return; } } catch {}
  }
}
async function projectNames(): Promise<string[]> {
  return (await discoverProjects()).map(p => p.name);
}

// ── Smart project discovery (the Rust walker: whole-$HOME, scored best-first).
// Cached 2 min so paging through the Add-a-project buttons doesn't re-walk the
// disk; already-managed projects (shared registry) are filtered out.
type DiscProj = { name: string; path: string; container: string; stack: string[]; score: number; last_active_days: number | null };
let discoverCache: { list: DiscProj[]; at: number } = { list: [], at: 0 };
async function discoverProjects(fresh = false): Promise<DiscProj[]> {
  if (!fresh && Date.now() - discoverCache.at < 120_000 && discoverCache.list.length) return discoverCache.list;
  try {
    const out = await omega(["projects", "--json"]);
    const arr = JSON.parse(out.slice(out.indexOf("["))) as DiscProj[];
    const reg = loadRegistry();
    const knownPaths = new Set(reg.projects.map((p: any) => String(p.path)));
    const knownNames = new Set(reg.projects.map((p: any) => String(p.name).toLowerCase()));
    discoverCache = { list: arr.filter(p => !knownPaths.has(p.path) && !knownNames.has(p.name.toLowerCase())), at: Date.now() };
  } catch { discoverCache = { list: [], at: Date.now() }; }
  return discoverCache.list;
}
async function sessionNames(): Promise<string[]> {
  return (await omega(["list"])).split("\n").map(l => l.replace(/^[^A-Za-z0-9_-]*/, "").trim().split(/\s+/)[0]).filter(s => /^[A-Za-z0-9][\w.-]*$/.test(s));
}
async function auditIds(): Promise<string[]> {
  const ids: string[] = [];
  for (const l of (await omega(["audit", "list"])).split("\n")) { const m = l.match(/^\s{2,}([a-z][a-z0-9-]*audit)\b/); if (m) ids.push(m[1]); }
  return [...new Set(ids)];
}

// ── command menu (setMyCommands list) ────────────────────────────────────────
// OmegaMC dashboard API (read agents). Web password from omega-mc .env.
const MC_PW = readKV(MC_ENV, /^([A-Z_]+)=(.*)$/).OMEGA_MC_WEB_PASSWORD || "";
async function mcAgents(): Promise<{ id: string; description?: string }[]> {
  try {
    const r = await fetch("http://localhost:8080/api/agents/definitions", { headers: { authorization: "Basic " + Buffer.from(":" + MC_PW).toString("base64") } });
    const j = await r.json(); return Array.isArray(j) ? j : (j.agents || []);
  } catch { return []; }
}

const MENU: [string, string][] = [
  ["start", "Welcome + quick status"],
  ["guide", "How OmegaOS works — full guide"],
  ["menu", "Action hub — all commands as buttons"],
  ["commands", "Show available commands"],
  ["agents", "List the AISB agents (talk via the agents bot)"],
  ["council", "Convene @council — judge panel for a high-stakes/contested decision"],
  ["dashboard", "Open the Mission Control dashboard (link)"],
  ["status", "Live system status"],
  ["sessions", "Active sessions — Status / Kill"],
  ["projects", "Projects — list / new / add"],
  ["audits", "Quality Arsenal — tap an audit"],
  ["account", "Account / billing / accounts"],
  ["model", "AI provider + model"],
  ["skills", "Installed skills"],
  ["dispatch", "Dispatch a mission to an oracle"],
  ["setupgroup", "Register this group as the project hub"],
  ["sync", "Sync projects ↔ Telegram topics"],
  ["topic", "Toggle a project's Telegram topic on/off"],
  ["delete", "Delete a project (OmegaOS / full / forever)"],
  ["killall", "Kill all sessions (keeps infra)"],
  ["clean", "Cleanup stray sessions + state"],
  ["help", "Show the action hub"],
];
// Commands with a dedicated button view/handler. Anything NOT here is routed to
// the AISB Master brain instead of falling back to the menu (intelligent commands).
const KNOWN = new Set<string>([...MENU.map(([c]) => c), "setupgroup", "sync", "dispatch", "zernio"]);
function menuKb() {
  // The NOVA OS row controls the operator-built omega-novaos.service — only
  // show it where that unit exists (on a fresh install it was an always-broken
  // button: status forever "inactive", start always failing).
  const hasNovaOS = existsSync(`${homedir()}/.config/systemd/user/omega-novaos.service`);
  return kb([
    [{ text: "📖 Guide — how it works", callback_data: "nav:guide" }],
    [{ text: "🤖 Agents", callback_data: "nav:agents" }, { text: "🖥 Dashboard", callback_data: "nav:dashboard" }],
    [{ text: "📊 Status", callback_data: "nav:status" }, { text: "🗂 Sessions", callback_data: "nav:sessions" }],
    [{ text: "📁 Projects", callback_data: "nav:projects" }, { text: "🔍 Audits", callback_data: "nav:audits" }],
    [{ text: "💳 Account", callback_data: "nav:account" }, { text: "🧠 Model", callback_data: "nav:model" }],
    [{ text: "🧩 Skills", callback_data: "nav:skills" }, { text: "🚀 Dispatch", callback_data: "nav:dispatch" }],
    [{ text: "🌀 Zernio — publish", callback_data: "nav:zernio" }],
    [{ text: "👥 Group hub", callback_data: "nav:setupgroup" }, { text: "🧹 Clean", callback_data: "nav:clean" }],
    ...(hasNovaOS ? [[{ text: "🤖 NOVA OS (status / kill-switch)", callback_data: "nav:novaos" }]] : []),
  ]);
}
const menuText = card("OMEGAOS — ACTION HUB", " Tap an action. Each one runs on your server via the <code>omega</code> CLI.");
// NOVA OS control: status + the emergency kill-switch (and re-start). systemctl
// --user needs XDG_RUNTIME_DIR; the service is the manual safety the oracle built.
async function novaosCtl(cmd: "status" | "start" | "stop"): Promise<string> {
  const env = `XDG_RUNTIME_DIR=/run/user/$(id -u)`;
  if (cmd === "status") {
    const act = (await $`bash -lc ${`${env} systemctl --user is-active omega-novaos.service`}`.nothrow().text()).trim();
    const brain = (await $`bash -lc ${`curl -fsS --max-time 3 http://127.0.0.1:7777/health >/dev/null 2>&1 && echo OK || echo DOWN`}`.nothrow().text()).trim();
    const eliza = (await $`bash -lc ${`curl -fsS --max-time 3 http://127.0.0.1:3000/api/server/ping >/dev/null 2>&1 && echo OK || echo DOWN`}`.nothrow().text()).trim();
    return `Service: <b>${esc(act)}</b>\nCerveau (claude -p, :7777): <b>${esc(brain)}</b>\nElizaOS (:3000): <b>${esc(eliza)}</b>`;
  }
  await $`bash -lc ${`${env} systemctl --user ${cmd} omega-novaos.service`}`.nothrow().text();
  await $`bash -lc ${`sleep ${cmd === "start" ? 12 : 2}`}`.nothrow().text();
  return novaosCtl("status");
}

// /start — welcome + live status pulse. Greets the operator as Atlas, says what
// they can do, and shows a one-line health snapshot. New users land here.
// /start AND /guide — the full operator guide. Greets as Atlas, explains the
// whole OmegaOS workflow in detail with a live health pulse, and a button to open
// the action menu. The guide is the landing screen; /menu is the buttons.
async function guideCard(): Promise<string> {
  let sessions = 0, health = "?";
  try { sessions = (await omega(["list"])).split("\n").filter(l => /^\s*[⌂◆●]/.test(l)).length; } catch {}
  try { const raw = await omega(["doctor"]); const w = (raw.match(/^\s*\[[!x]\]/gm) || []).length; health = w ? `🟡 ${w} warning(s)` : "🟢 healthy"; } catch {}
  return card("OMEGAOS — ATLAS",
    ` 👋 <b>Hi! I'm Atlas</b> — the brain of your OmegaOS, an autonomous multi-agent dev platform running on your own server.\n\n` +
    ` 💬 <b>Just talk to me in plain language.</b> I keep our conversation in context, figure out what you want, and either answer or dispatch the work. Reply to any of my messages to keep a thread going.\n\n` +
    ` <b>⚙️ How it works</b>\n` +
    ` 🧠 <b>Atlas</b> — your single point of contact; I plan and route everything.\n` +
    ` 🔮 <b>Oracle</b> — one strategist per project; it breaks the mission down.\n` +
    ` ⚙️ <b>Workers</b> — ephemeral agents in parallel (file-scoped): execute → verify → report.\n` +
    ` ✅ <b>Quality gates</b> — rubric + consensus + adversarial verification before anything ships.\n` +
    ` 📊 <b>Live progress</b> — long missions stream a progress card; full report when done.\n\n` +
    ` 💡 <b>Simplest path:</b> just tell me what you want — “UX audit of DentistryGPT”, “new project …”, “deploy X and verify in prod”. Or tap <b>Open menu</b> below.`,
    ` 📡 <b>${sessions} active session${sessions === 1 ? "" : "s"}</b>  ·  doctor ${health}\n\n` +
    `<blockquote expandable>📋 <b>Every menu action</b>\n\n` +
    ` 📊 <b>Status</b> — live system health + one-tap Fix-it\n` +
    ` 🗂 <b>Sessions</b> — your sessions; Status / Kill\n` +
    ` 📁 <b>Projects</b> — list / create / add (oracle + topic each)\n` +
    ` 🔍 <b>Audits</b> — Quality Arsenal: 23 forensic audits\n` +
    ` 💳 <b>Account</b> — Claude login (one shared credential) + usage\n` +
    ` 🧠 <b>Model</b> — pick the AI provider + model\n` +
    ` 🤖 <b>Agents</b> — a dedicated bot per project oracle\n` +
    ` 🖥 <b>Dashboard</b> — Mission Control (web)\n` +
    ` 🚀 <b>Dispatch</b> — fire a mission at an oracle\n` +
    ` 👥 <b>Group hub</b> — supergroup: 1 topic = 1 project</blockquote>`);
}

// Strip an appended remediation SHELL COMMAND that `omega doctor` packs into a
// warning detail ("problem — T=$(mktemp) && curl … | bash"). Prose hints
// ("duplicate messages; keep only systemd …") are kept — only literal commands
// are dropped, so the card stays clean. The actual fix is the "Fix it" oracle.
function cleanDetail(v: string): string {
  const isCmd = (s: string) => /\$\(|&&|\bcurl\b|\bmktemp\b|https?:\/\/|\|\s*bash|bash\s+["']?\$/.test(s);
  const parts = v.split(" — ");
  const out = [parts[0]];
  for (const p of parts.slice(1)) { if (isCmd(p)) break; out.push(p); }
  return out.join(" — ").slice(0, 100);
}

// ── status card: parse `omega doctor` → branded health hero (design #6+#10) ───
// Lines: `  [+] label   value` (ok) / `  [!] label   value` (warning), 2-space
// indent; the verdict line is at column 0 (`[!] healthy, with warnings above`).
function statusCard(raw: string): string {
  const checks: { ok: boolean; label: string; value: string }[] = [];
  let verdict = "";
  for (const line of raw.split("\n")) {
    const c = line.match(/^\s+\[([+!])\]\s+(.+?)\s{2,}(.+)$/);
    if (c) { checks.push({ ok: c[1] === "+", label: c[2].trim(), value: c[3].trim() }); continue; }
    const v = line.match(/^\[([+!])\]\s+(.+)$/);
    if (v) verdict = v[2].trim();
  }
  if (!checks.length) return card("OMEGAOS — STATUS", `<pre>${esc(raw).slice(0, MAXLEN)}</pre>`);
  const total = checks.length, okN = checks.filter(c => c.ok).length;
  const warns = checks.filter(c => !c.ok);
  const pct = Math.round((okN / total) * 100);
  // expired / fail / missing → critical (🔴); other warnings → 🟡.
  const sev = (v: string): "warn" | "err" => /expir|fail|missing|introuv|error|❌|duplicate|multiple/i.test(v) ? "err" : "warn";
  const hero = warns.length ? (warns.some(w => sev(w.value) === "err") ? "🔴" : "⚠️") : "✅";
  const verdictTxt = (verdict || (warns.length ? `healthy · ${warns.length} warning(s)` : "all systems healthy")).toUpperCase();
  let body = ` ${hero} <b>${esc(verdictTxt)}</b>\n    <code>${bar(pct)}</code>  ${pct}%`;
  if (warns.length)
    body += `\n\n ❗ <b>TO FIX</b>\n` + warns.map(w => `  ${dot(sev(w.value))} <b>${esc(w.label)}</b> — ${esc(cleanDetail(w.value))}`).join("\n");
  const details = checks.map(c => ` ${dot(c.ok ? "ok" : sev(c.value))} <b>${esc(c.label.toUpperCase())}</b>  ${esc(cleanDetail(c.value))}`).join("\n");
  body += `\n\n<blockquote expandable>▾ ${total} system checks\n${details}</blockquote>`;
  return `${RULE}\n   Ω  O M E G A O S\n${RULE}\n\n${body}\n\n${RULE}`;
}

// ── model picker: provider → model, all clickable. Canonical lists come from the
// Rust SSOT (`omega config models [provider]`); a mirror of providers.rs::models_for
// is the fallback for binaries predating that subcommand. Selecting writes
// providers.toml (omega sessions) and, for claude, the omega-mc dashboard fallback
// (defaults.model only — the per-agent opus/sonnet split is preserved).
const PROVIDER_FALLBACK = ["claude", "codex", "gemini", "glm", "openrouter"];
const MODEL_FALLBACK: Record<string, string[]> = {
  claude: ["opus", "sonnet", "haiku"],
  codex: ["gpt-5", "gpt-5-codex", "o3"],
  gemini: ["gemini-2.5-pro", "gemini-2.5-flash"],
  glm: ["glm-4.6", "glm-4.5"],
  openrouter: ["anthropic/claude-sonnet-4.6", "anthropic/claude-opus-4.8", "openai/gpt-5", "google/gemini-2.5-pro", "deepseek/deepseek-chat"],
};
const PROVIDER_ICON: Record<string, string> = { claude: "🟣", codex: "🟢", gemini: "🔵", glm: "🟡", openrouter: "🌐" };
// Claude alias → full model id the omega-mc yaml uses (mirror of dispatch.rs + the
// dashboard's model convention). Anything not aliased is passed through verbatim.
const CLAUDE_FULL_ID: Record<string, string> = { opus: "claude-opus-4-8", sonnet: "claude-sonnet-4-6", haiku: "claude-haiku-4-5" };
async function listProviders(): Promise<string[]> {
  const out = await omega(["config", "models"]);
  const ps = out.split("\n").map(s => s.trim()).filter(s => /^[a-z]+$/.test(s));
  return ps.length ? ps : PROVIDER_FALLBACK;
}
async function listModels(provider: string): Promise<string[]> {
  const out = await omega(["config", "models", provider]);
  const ms = out.split("\n").map(s => s.trim()).filter(Boolean).filter(s => !/error|unrecognized|usage:|no output/i.test(s));
  return ms.length ? ms : (MODEL_FALLBACK[provider] || []);
}
async function currentModel(provider: string): Promise<string> {
  const v = (await omega(["config", "get", `${provider}.model`])).trim().split("\n")[0] || "";
  return /error|unknown|no output/i.test(v) ? "" : v;
}
// Update the dashboard's defaults.model (the FIRST `model:` in the yaml = the
// defaults block, before any agent). Per-agent models are untouched. Returns the
// full id written, or "" on no-op. omega-mc hot-reloads the file within ~3s.
function mcSetDefaultModel(fullId: string): string {
  try {
    const y = readFileSync(MC_CONFIG, "utf8");
    const next = y.replace(/(\n\s*model:\s*)"[^"]*"/, `$1"${fullId}"`);
    if (next === y) return "";
    writeFileSync(MC_CONFIG, next);
    return fullId;
  } catch { return ""; }
}
// Render the model list for a provider with the current pick marked ✓.
// Is a provider's API key set? (omega() returns "(no output…" for an empty value.)
async function providerHasKey(provider: string): Promise<boolean> {
  const v = (await omega(["config", "get", `${provider}.api_key`])).trim();
  return !!v && !/^\(no output/.test(v) && !/error/i.test(v);
}
async function modelProviderView(provider: string, banner = ""): Promise<{ text: string; markup: any }> {
  const [models, cur, hasKey] = await Promise.all([listModels(provider), currentModel(provider), providerHasKey(provider)]);
  const rows: Btn[][] = [];
  for (let i = 0; i < models.length; i += 2)
    rows.push(models.slice(i, i + 2).map(m => ({ text: `${m === cur ? "✓ " : ""}${m}`.slice(0, 28), callback_data: `model:set:${provider}:${m}`.slice(0, 64) })));
  // API-key row: a 🗑 delete-key button when a key is set. Deleting it makes sessions
  // fall back to OAuth/subscription — which stops Claude Code's "use this API key?"
  // prompt that breaks autonomous oracle sessions.
  const keyLine = hasKey
    ? ` 🔑 API key: <b>set</b> — sessions use it. <i>Delete it (🗑) to use your subscription/OAuth instead and stop the “use this key?” prompt.</i>`
    : ` 🔑 API key: <i>none</i> — sessions use OAuth/subscription (autonomous, no prompt). ✅`;
  const body = (banner ? banner + "\n\n" : "") + (models.length
    ? ` Current: <code>${esc(cur || "default")}</code>\n Tap a model to activate it.\n\n${keyLine}`
    : ` No catalogued models. Configure: <code>omega config set ${esc(provider)}.model …</code>\n\n${keyLine}`);
  const keyRow: Btn[][] = hasKey ? [[{ text: "🗑 Delete API key (x)", callback_data: `model:delkey:${provider}`.slice(0, 64) }]] : [];
  return { text: card(`MODEL — ${provider.toUpperCase()}`, body), markup: kb([...rows, ...keyRow, [{ text: "« Providers", callback_data: "nav:model" }]]) };
}

// ── Zernio views (built from the omega-zernio --json CLI) ─────────────────────
// Top-level: each Zernio profile (project) + its connected channels, one button
// per project. Never crashes — a CLI error (missing key) renders as text.
async function zernioHome(): Promise<{ text: string; markup: any }> {
  const pr = await zernio(["profiles", "--json"]);
  if (!pr.ok) return { text: card("ZERNIO — PUBLISH", ` ⚠️ Zernio CLI error:\n<pre>${esc((pr.err || pr.out || "no output").slice(0, 700))}</pre>\n\n <i>Set ZERNIO_API_KEY in ~/.omega/secrets and retry.</i>`), markup: kb([[back()]]) };
  const pj = zjson(pr.out);
  const profiles: any[] = Array.isArray(pj) ? pj : (pj?.profiles || []);
  const ac = await zernio(["accounts", "--json"]);
  const aj = ac.ok ? zjson(ac.out) : null;
  const accounts: any[] = Array.isArray(aj) ? aj : (aj?.accounts || []);
  const lines = profiles.map(p => {
    const conn = accounts.filter(a => zProfileId(a) === p._id);
    const tags = conn.length ? conn.map(a => PLAT_EMOJI[a.platform] || "•").join(" ") : "<i>no channels</i>";
    return ` 📛 <b>${esc(p.name)}</b>${p.isDefault ? " <i>·default</i>" : ""} — ${tags}`;
  });
  const body = (lines.length ? lines.join("\n") : " <i>No Zernio profile yet.</i>") +
    `\n\n <i>Tap a project to connect channels. To publish, type e.g.</i>\n <code>publie sur instagram et tiktok pour &lt;projet&gt;: ton texte</code>`;
  const rows: Btn[][] = profiles.map(p => [{ text: `📛 ${p.name}`.slice(0, 30), callback_data: `zernio:prof:${p._id}`.slice(0, 64) }]);
  rows.push([back()]);
  return { text: card("ZERNIO — MULTI-CHANNEL PUBLISHING", body), markup: kb(rows) };
}
// Per-project: connected channels + a 2-wide connect grid (✓ already-connected,
// ➕ not yet — both tappable so a re-connect is always possible).
async function zernioProjectView(profileId: string): Promise<{ text: string; markup: any }> {
  const pr = await zernio(["profiles", "--json"]);
  const pj = pr.ok ? zjson(pr.out) : null;
  const profiles: any[] = Array.isArray(pj) ? pj : (pj?.profiles || []);
  const name = profiles.find(p => p._id === profileId)?.name || profileId;
  const ac = await zernio(["accounts", "--json"]);
  const aj = ac.ok ? zjson(ac.out) : null;
  const accounts: any[] = Array.isArray(aj) ? aj : (aj?.accounts || []);
  const conn = accounts.filter(a => zProfileId(a) === profileId);
  const connSet = new Set(conn.map(a => a.platform));
  const chLines = conn.length
    ? conn.map(a => ` ${PLAT_EMOJI[a.platform] || "•"} <b>${esc(a.platform)}</b>${a.username ? ` @${esc(String(a.username).replace(/^@/, ""))}` : ""} — ${esc(a.platformStatus || (a.isActive ? "active" : "inactive"))}`).join("\n")
    : " <i>No channel connected yet.</i>";
  const rows: Btn[][] = [];
  for (let i = 0; i < ZERNIO_PLATFORMS.length; i += 2)
    rows.push(ZERNIO_PLATFORMS.slice(i, i + 2).map(pl => ({ text: `${connSet.has(pl) ? "✓" : "➕"} ${pl}`.slice(0, 28), callback_data: `zernio:conn:${profileId}:${pl}`.slice(0, 64) })));
  rows.push([{ text: "« Zernio", callback_data: "nav:zernio" }]);
  return { text: card(`ZERNIO — ${String(name).toUpperCase()}`.slice(0, 48), `${chLines}\n\n <i>Tap a platform to connect (✓ = re-connect).</i>`), markup: kb(rows) };
}
// Connect: resolve profile→name, fetch the hosted OAuth authUrl, render it as a
// tap-to-open URL button (Telegram inline url button).
async function zernioConnect(profileId: string, platform: string): Promise<{ text: string; markup: any }> {
  const pr = await zernio(["profiles", "--json"]);
  const pj = pr.ok ? zjson(pr.out) : null;
  const profiles: any[] = Array.isArray(pj) ? pj : (pj?.profiles || []);
  const prof = pr.ok ? profiles.find(p => p._id === profileId) : null;
  if (!prof?.name)
    return { text: card("ZERNIO — CONNECT", " ⚠️ Profil introuvable — rouvre /zernio et reessaie."), markup: kb([[{ text: "« Zernio", callback_data: "nav:zernio" }]]) };
  const name = prof.name;
  const r = await zernio(["connect", String(name), platform, "--json"]);
  const j = r.ok ? zjson(r.out) : null;
  const authUrl: string = j?.authUrl || "";
  if (!/^https?:\/\//.test(authUrl))
    return { text: card(`ZERNIO — CONNECT ${platform.toUpperCase()}`, ` ⚠️ Could not get an authorization link:\n<pre>${esc((r.err || r.out || "no output").slice(0, 600))}</pre>`), markup: kb([[{ text: "« Back", callback_data: `zernio:prof:${profileId}` }]]) };
  return {
    text: card(`ZERNIO — CONNECT ${platform.toUpperCase()}`, ` 🔗 Connect <b>${esc(platform)}</b> to <b>${esc(String(name))}</b>.\n Tap below to authorize. The account attaches to this project's Zernio profile.`),
    markup: kb([[{ text: `🔗 Authorize ${platform}`, url: authUrl }], [{ text: "« Back", callback_data: `zernio:prof:${profileId}` }]]),
  };
}
// Render a dry-run validation result as a preview card body.
function zernioPreviewBody(project: string, platforms: string[], postText: string, dj: any): string {
  const v = dj?.validation || {};
  const ok = dj?.effectiveValid ?? v?.valid;
  const errs = (v?.errors || []).map((e: any) => ` 🔴 [${esc(e.platform)}] ${esc(e.error)}`);
  const warns = (v?.warnings || []).map((w: any) => ` 🟡 [${esc(w.platform)}] ${esc(w.warning)}`);
  const snippet = postText.length > 280 ? postText.slice(0, 280) + "…" : postText;
  return ` Project: <b>${esc(project)}</b>\n Channels: ${platforms.map(p => `${PLAT_EMOJI[p] || "•"} ${p}`).join("  ")}\n Validation: ${ok ? "🟢 ready" : "🔴 has issues"}` +
    (errs.length ? `\n${errs.join("\n")}` : "") +
    (warns.length ? `\n${warns.join("\n")}` : "") +
    `\n\n <i>Text:</i>\n<blockquote>${esc(snippet)}</blockquote>\n\n <i>Confirm to publish for real, or cancel.</i>`;
}

// ── views ────────────────────────────────────────────────────────────────────
async function view(name: string): Promise<{ text: string; markup: any }> {
  switch (name) {
    case "menu": case "help": case "commands": return { text: menuText, markup: menuKb() };
    case "start": case "guide": return { text: await guideCard(), markup: kb([[{ text: "📋 Open menu", callback_data: "nav:menu" }], [{ text: "🚀 Dispatch", callback_data: "nav:dispatch" }, { text: "💳 Account", callback_data: "nav:account" }]]) };
    case "agents": {
      // The companion (Nova) link lives HERE — it is the only flow that creates
      // a kind:"companion" agent-bot entry, so it must not depend on the
      // optional MC dashboard being up.
      const novaRow: Btn[] = [{ text: "💞 Link your companion (Nova)", callback_data: "agent:tglink:nova" }];
      // Like Nova, the security operator (Trinity) binds to its own bot from here —
      // its own kind:"security" entry, independent of the optional MC dashboard.
      const trinityRow: Btn[] = [{ text: "🛡 Link your security agent (Trinity)", callback_data: "agent:tglink:trinity" }];
      const ags = await mcAgents();
      if (!ags.length) return { text: card("AISB AGENTS", " ⚠️ Dashboard unreachable. Start it: <code>omega-mc-up</code>.\n\n 💞 You can still link your personal companion bot (Nova) and 🛡 security agent (Trinity) below."), markup: kb([novaRow, trinityRow, [back()]]) };
      const rows: Btn[][] = [];
      for (let i = 0; i < ags.length; i += 2) rows.push(ags.slice(i, i + 2).map(a => ({ text: a.id.slice(0, 28), callback_data: `agent:info:${a.id}`.slice(0, 64) })));
      return { text: card(`AISB AGENTS — ${ags.length}`, " Tap an agent for its role. To talk to it, use its dedicated bot (see /dashboard).\n 💞 “Link your companion” wires Nova — your personal assistant on her own bot.\n 🛡 “Link your security agent” wires Trinity — a white-hat pentest operator on its own bot."), markup: kb([...rows, novaRow, trinityRow, [back()]]) };
    }
    case "dashboard": {
      await resolvePublicIP();
      const { url } = dashboardURL();
      const rows: Btn[][] = [];
      if (url) rows.push([{ text: "👉 Tap here to open", url }]);
      rows.push([{ text: "🔑 Reveal the password", callback_data: "dash:pw" }]);
      rows.push([back()]);
      const body = url
        ? ` <code>${esc(url)}</code>\n\n Tap “👉 Open” for the dashboard, then “🔑 Reveal” for the password.`
        : ` ⚠️ Public IP not resolved — try again, or enable Tailscale for secure access.`;
      return { text: card("MISSION CONTROL", body), markup: kb(rows) };
    }
    case "status": return { text: statusCard(await omega(["doctor"])), markup: kb([[{ text: "🛠 Fix it", callback_data: "status:fix" }, { text: "🔄 Refresh", callback_data: "nav:status" }], [back()]]) };
    case "sessions": {
      const names = await sessionNames();
      const rows = names.slice(0, 12).map(s => [{ text: `📊 ${s}`.slice(0, 30), callback_data: `sess:status:${s}`.slice(0, 64) }, { text: "🛑 Kill", callback_data: `sess:kill:${s}`.slice(0, 64) }]);
      const list = names.length ? names.map(s => ` 🟢 <code>${esc(s)}</code>`).join("\n") : " <i>No active session.</i>";
      return { text: card(`SESSIONS — ${names.length}`, list), markup: kb([...rows, [{ text: "🔄 Refresh", callback_data: "nav:sessions" }, back()]]) };
    }
    case "projects": {
      const mp = loadProjects();
      const abots = loadAgentBots();
      const hasBot = (n: string) => !!(abots[projId(n)] || abots[n]);
      const names = Object.keys(mp).sort();
      // 🔕 = Telegram topic OFF · 🤖 = a dedicated agent-bot is linked.
      const list = names.length
        ? names.map(n => `${mp[n].telegram ? "•" : "🔕"} <b>${esc(n)}</b> <i>(${esc(mp[n].category || "?")})</i>${hasBot(n) ? " 🤖" : ""}`).join("\n")
        : "<i>No managed project yet — add one (📁) or create one (➕).</i>";
      const rows: Btn[][] = [];
      for (let i = 0; i < names.length; i += 2) rows.push(names.slice(i, i + 2).map(n => ({ text: `${mp[n].telegram ? "📦" : "🔕"} ${n}${hasBot(n) ? " 🤖" : ""}`.slice(0, 28), callback_data: `proj:open:${n}`.slice(0, 64) })));
      return { text: card(`PROJECTS — ${names.length}`, list), markup: kb([...rows, [{ text: "➕ New", callback_data: "proj:new" }, { text: "📁 Add existing", callback_data: "proj:add" }], [{ text: "⬇️ Import from GitHub", callback_data: "proj:import" }], [{ text: "🔧 Git", callback_data: "git:list" }, { text: "🔁 Sync", callback_data: "nav:sync" }], [back()]]) };
    }
    case "audits": {
      const ids = await auditIds(); const rows: Btn[][] = [];
      for (let i = 0; i < ids.length; i += 2) rows.push(ids.slice(i, i + 2).map(a => ({ text: a.slice(0, 28), callback_data: `aud:run:${a}`.slice(0, 64) })));
      return { text: card("QUALITY ARSENAL", ` ${ids.length} audits available — tap one to run it.`), markup: kb([...rows, [back()]]) };
    }
    // Account: the two actions the operator actually needs — Login (re-auth) and
    // Usage — plus Refresh/Back. (Switch / email / billing / service-accounts were
    // noise; their callback handlers stay for /-command access but are off the view.)
    case "account": return { text: await accountStatus(), markup: kb([
      [{ text: "🔐 Login / Re-auth", callback_data: "acct:login" }, { text: "📊 Usage tokens", callback_data: "acct:usage" }],
      [{ text: "🔄 Refresh", callback_data: "nav:account" }, back()],
    ]) };
    case "model": {
      const provs = await listProviders();
      const active = await currentModel("claude");
      const rows: Btn[][] = [];
      for (let i = 0; i < provs.length; i += 2)
        rows.push(provs.slice(i, i + 2).map(p => ({ text: `${PROVIDER_ICON[p] || "•"} ${p}`.slice(0, 28), callback_data: `model:prov:${p}`.slice(0, 64) })));
      const body = ` omega sessions run on:\n <b>claude</b> · <code>${esc(active || "default")}</code>\n\n Pick a provider to view and change its model.`;
      return { text: card("MODEL / PROVIDERS", body), markup: kb([...rows, [{ text: "🔄 Refresh", callback_data: "nav:model" }, back()]]) };
    }
    case "zernio": return await zernioHome();
    case "skills": return { text: pre("Skills", Bun.spawnSync(["ls", "-1", `${OMEGA_DIR}/skills`]).stdout.toString().trim() || "(none)"), markup: kb([[back()]]) };
    case "dispatch": return { text: card("DISPATCH", " Send: <code>/dispatch &lt;project&gt; &lt;mission&gt;</code>\n Launches a dedicated oracle on the VPS."), markup: kb([[{ text: "📁 Projects", callback_data: "nav:projects" }], [back()]]) };
    case "setupgroup": return { text: card("GROUP HUB", " Run <code>/setupgroup</code> <b>in a supergroup</b> where this bot is <b>admin</b> (Topics enabled). It registers the group as the project hub, then <code>/sync</code> maps each project to a topic."), markup: kb([[back()]]) };
    case "sync": { const g = loadGroups(); return { text: card("SYNC", g.hub ? " Hub registered. Run <code>/sync</code> in it to map projects → topics." : " No hub yet — run <code>/setupgroup</code> in your supergroup first."), markup: kb([[back()]]) }; }
    case "killall": return { text: card("KILL ALL SESSIONS?", " 🛑 Kills every session.\n <i>Keeps the infra (Home/System, bridge, master).</i>"), markup: kb([[{ text: "✅ Yes", callback_data: "do:killall" }], [{ text: "✖ Cancel", callback_data: "nav:menu" }]]) };
    case "novaos": return { text: card("🤖 NOVA OS", " Le corps public de Nova (ElizaOS + cerveau claude -p).\n\n" + await novaosCtl("status")), markup: kb([
      [{ text: "🟢 Allumer", callback_data: "do:novaup" }, { text: "🛑 COUPER (urgence)", callback_data: "do:novadown" }],
      [{ text: "🔄 Rafraîchir", callback_data: "nav:novaos" }, back()],
    ]) };
    case "clean": case "cleaning": return { text: card("🧹 CLEANING", " Maintenance du VPS et des projets — choisis une action :"), markup: kb([
      [{ text: "💽 Nettoyage disque", callback_data: "nav:cleandisk" }, { text: "💾 Purge RAM", callback_data: "do:ramflush" }],
      [{ text: "🗂️ Ranger les projets (plan)", callback_data: "do:tidy" }],
      [{ text: "📊 Analyse disque", callback_data: "do:diskanalyze" }],
      [{ text: "🧽 Sessions orphelines", callback_data: "nav:cleansess" }, { text: "☠️ Kill all", callback_data: "nav:killall" }],
      [back()],
    ]) };
    case "cleandisk": return { text: card("NETTOYAGE DISQUE ?", " 💽 Purge cache Docker + APT + journal + caches user.\n <i>Rebuildable — ne touche jamais au code.</i>"), markup: kb([[{ text: "✅ Oui, nettoyer", callback_data: "do:diskclean" }], [{ text: "✖ Annuler", callback_data: "nav:clean" }]]) };
    case "cleansess": return { text: card("CLEANUP SESSIONS ?", " 🧹 Kills orphan sessions + purges the state.\n <i>Never touches the infra.</i>"), markup: kb([[{ text: "✅ Yes", callback_data: "do:clean" }], [{ text: "✖ Cancel", callback_data: "nav:clean" }]]) };
    default: return { text: menuText, markup: menuKb() };
  }
}

async function onCallback(data: string, chat: number, msgId: number, from: number) {
  const [ns, action, ...rest] = data.split(":"); const arg = rest.join(":");
  if (ns === "nav") { const v = await view(action); return edit(chat, msgId, v.text, v.markup); }
  if (ns === "zernio") {
    if (action === "prof") { const v = await zernioProjectView(rest[0] || ""); return edit(chat, msgId, v.text, v.markup); }
    if (action === "conn") { const v = await zernioConnect(rest[0] || "", rest[1] || ""); return edit(chat, msgId, v.text, v.markup); }
    if (action === "pub") {
      const backRow = kb([[{ text: "« Zernio", callback_data: "nav:zernio" }]]);
      if (rest[0] === "no") { clearPending(from); return edit(chat, msgId, card("ZERNIO — PUBLISH", " ❌ Annulé."), backRow); }
      const p = getPending(from);
      if (!p || p.kind !== "zernio-post" || !p.arg) return edit(chat, msgId, card("ZERNIO — PUBLISH", " ⏳ Rien à publier (la demande a expiré). Relance la commande."), backRow);
      const d = zjson(p.arg) || {};
      clearPending(from);
      await edit(chat, msgId, card("ZERNIO — PUBLISH", ` ⏳ Publishing <b>${esc(d.project || "?")}</b> → ${esc(d.platforms || "?")} …`));
      const res = await zernio(["post", String(d.project), "--text", String(d.text), "--platforms", String(d.platforms), "--json"]);
      const rj = res.ok ? zjson(res.out) : null;
      if (res.ok && rj?.posted) {
        const post = rj.post || {};
        return edit(chat, msgId, card("ZERNIO — PUBLISHED", ` ✅ Published to ${esc(d.platforms || "?")}.\n 🆔 <code>${esc(post._id || "?")}</code>${post.status ? `\n status: ${esc(post.status)}` : ""}`), backRow);
      }
      return edit(chat, msgId, card("ZERNIO — PUBLISH", ` 🔴 Publish failed:\n<pre>${esc((res.err || res.out || "no output").slice(0, 800))}</pre>`), backRow);
    }
    return;
  }
  if (ns === "status" && action === "fix") {
    // Collect current doctor warnings/fails → dispatch an OmegaOS oracle to fix
    // them (a real tracked session; the Monitor relays the result back here).
    const raw = await omega(["doctor"]);
    const warns = raw.split("\n").filter(l => /^\s*\[[!x]\]/.test(l)).map(l => l.replace(/^\s*\[[!x]\]\s*/, "").trim()).filter(Boolean);
    if (!warns.length) return edit(chat, msgId, card("OMEGAOS — FIX IT", " ✅ Nothing to fix — all green."), kb([[{ text: "« Status", callback_data: "nav:status" }]]));
    const mission = `Auto-heal OmegaOS. \`omega doctor\` reports these problems — diagnose the root cause and fix each one (you can lean on \`omega doctor --fix\` for the mechanical fixes), then verify with \`omega doctor\` that everything turns green again:\n` + warns.map(w => `- ${w}`).join("\n");
    // Make `omega dispatch OmegaOS` resolve: the OS repo isn't auto-discovered
    // (it's a sibling of the container dirs), so register it in the shared
    // registry first (idempotent — recordProject upserts by path/name).
    recordProject("OmegaOS", repoPath("OmegaOS") || `${homedir()}/Station/OmegaOS`);
    const out = await dispatchToOracle("OmegaOS", mission, chat, undefined);
    return edit(chat, msgId, out, kb([[{ text: "« Status", callback_data: "nav:status" }]]));
  }
  if (ns === "model" && action === "prov") { const v = await modelProviderView(arg); return edit(chat, msgId, v.text, v.markup); }
  if (ns === "model" && action === "delkey") {
    // Delete a provider's API key → sessions fall back to OAuth/subscription, which
    // stops Claude Code's "do you want to use this API key?" prompt (breaks autonomy).
    const res = await omega(["config", "set", `${arg}.api_key`, ""]);
    const ok = /^\[\+\] Set/m.test(res);
    const v = await modelProviderView(arg, ` ${ok ? "🗑 ✅" : "⚠️"} <b>${esc(arg)}</b> API key ${ok ? "deleted — sessions now use OAuth/subscription (autonomous, no prompt)." : "delete failed: " + esc(res.slice(0, 80))}`);
    return edit(chat, msgId, v.text, v.markup);
  }
  if (ns === "model" && action === "set") {
    // arg = "provider:model" — model may contain "/" (openrouter ids), never ":".
    const i = arg.indexOf(":"); const provider = arg.slice(0, i); const model = arg.slice(i + 1);
    const res = await omega(["config", "set", `${provider}.model`, model]);
    const okOmega = /^\[\+\] Set/m.test(res);
    let dash = "";
    if (provider === "claude") {
      const full = CLAUDE_FULL_ID[model] || model;
      const wrote = mcSetDefaultModel(full);
      dash = `\n 🖥 Dashboard defaults: ${wrote ? `<code>${esc(full)}</code> ✅ <i>(hot-reload ~3s)</i>` : "unchanged"}`;
    }
    const banner = ` ${okOmega ? "✅" : "⚠️"} <b>${esc(provider)}</b> → <code>${esc(model)}</code>\n ⚙️ omega sessions: ${okOmega ? "✅" : "⚠️ " + esc(res.slice(0, 80))}${dash}`;
    const v = await modelProviderView(provider, banner);
    return edit(chat, msgId, v.text, v.markup);
  }
  if (ns === "dash" && action === "pw") {
    const { pw } = dashboardURL();
    if (!pw) return;
    // Reveal in a copyable code block, then auto-delete after 30s (so it never lingers in chat history).
    const m = await tg("sendMessage", { chat_id: chat, parse_mode: "HTML", text: `🔑 <b>Dashboard password</b>\n(tap it to copy — disappears in 30s)\n\n<code>${esc(pw)}</code>` });
    if (m.ok) setTimeout(() => tg("deleteMessage", { chat_id: chat, message_id: m.result.message_id }), 30000);
    return;
  }
  if (ns === "sess" && action === "status") return edit(chat, msgId, pre(`Session ${arg}`, await omega(["capture", arg])), kb([[{ text: "🔄 Refresh", callback_data: `sess:status:${arg}`.slice(0, 64) }, back("sessions")]]));
  if (ns === "sess" && action === "kill") return edit(chat, msgId, pre(`Kill ${arg}`, await omega(["kill", arg])), kb([[back("sessions")]]));
  if (ns === "proj" && action === "list") { const v = await view("projects"); return edit(chat, msgId, v.text, v.markup); }
  if (ns === "proj" && action === "new") {
    const cats = stationCategories();
    const rows: Btn[][] = [];
    for (let i = 0; i < cats.length; i += 2) rows.push(cats.slice(i, i + 2).map(c => ({ text: `📂 ${c}`.slice(0, 28), callback_data: `proj:newcat:${c}`.slice(0, 64) })));
    return edit(chat, msgId, "<b>➕ New project</b>\nWhich folder (category) under Station?", kb([...rows, [back("projects")]]));
  }
  if (ns === "proj" && action === "newcat") {
    setPending(from, "new-project", arg);
    return edit(chat, msgId, `<b>➕ New project — ${esc(arg)}</b>\nSend in <b>one message</b>:\n• <b>1st line</b> = project name\n• <b>following lines</b> = description (what it is, what we want to do)\n\nI create the folder + git, the dedicated oracle, the topic, then I <b>launch the oracle</b> on your description to start right away.`, kb([[{ text: "✖ Cancel", callback_data: "acct:cancel" }], [back("projects")]]));
  }
  // Import from GitHub: pick the Station category, then send the repo (URL or owner/repo).
  if (ns === "proj" && action === "import") {
    const cats = stationCategories();
    const rows: Btn[][] = [];
    for (let i = 0; i < cats.length; i += 2) rows.push(cats.slice(i, i + 2).map(c => ({ text: `📂 ${c}`.slice(0, 28), callback_data: `proj:importcat:${c}`.slice(0, 64) })));
    return edit(chat, msgId, "<b>⬇️ Import from GitHub</b>\nWhich folder (category) under Station?", kb([...rows, [back("projects")]]));
  }
  if (ns === "proj" && action === "importcat") {
    setPending(from, "import-project", arg);
    return edit(chat, msgId, `<b>⬇️ Import from GitHub — ${esc(arg)}</b>\nSend the repo: a <b>URL</b> (<code>https://github.com/owner/repo</code>) or an <b>owner/repo</b> slug.\n\nI clone it into <code>~/Station/${esc(arg)}/</code>, then wire the full setup — dedicated oracle, dashboard agent, Telegram topic and a <code>/{project}</code> command (private repos work via <code>gh</code>).`, kb([[{ text: "✖ Cancel", callback_data: "acct:cancel" }], [back("projects")]]));
  }
  if (ns === "proj" && action === "add") {
    // Smart whole-machine discovery (Rust walker, scored best-first), already-
    // managed projects filtered out, ONE button per project, paginated.
    const page = Math.max(0, parseInt(arg || "0", 10) || 0);
    const all = await discoverProjects(page === 0);
    if (!all.length) { setPending(from, "add-project"); return edit(chat, msgId, "<b>📁 Add a project</b>\nNo unmanaged project found on this machine — send the <b>name or absolute path</b> of the project to manage.", kb([[{ text: "✖ Cancel", callback_data: "acct:cancel" }], [back("projects")]])); }
    const PER = 8;
    const start = page * PER;
    const rows: Btn[][] = all.slice(start, start + PER).map((p, i) => [{
      text: `➕ ${p.name} · ${p.container}${p.stack.length ? " · " + p.stack[0] : ""}`.slice(0, 60),
      callback_data: `proj:dadd:${start + i}`,
    }]);
    const nav: Btn[] = [];
    if (page > 0) nav.push({ text: "« Prev", callback_data: `proj:add:${page - 1}` });
    if (start + PER < all.length) nav.push({ text: `More (${all.length - start - PER}) »`, callback_data: `proj:add:${page + 1}` });
    if (nav.length) rows.push(nav);
    return edit(chat, msgId, `<b>📁 Add a project</b>\n${all.length} unmanaged project(s) discovered on this machine, best first (git/manifest markers + recent activity). Tap one to manage it (dedicated oracle + dashboard + topic).`, kb([...rows, [{ text: "✍️ Other (type the name)", callback_data: "proj:addname" }], [back("projects")]]));
  }
  if (ns === "proj" && action === "dadd") {
    const p = discoverCache.list[parseInt(arg, 10)];
    if (!p) return edit(chat, msgId, "<b>📁 Add a project</b>\nThat discovery list expired — reopen it.", kb([[{ text: "🔄 Re-discover", callback_data: "proj:add" }], [back("projects")]]));
    return edit(chat, msgId, await addProject(p.name, p.path), kb([[{ text: "➕ Add another", callback_data: "proj:add" }, { text: "📋 Projects", callback_data: "nav:projects" }], [back("projects")]]));
  }
  if (ns === "proj" && action === "reg") return edit(chat, msgId, await addProject(arg), kb([[{ text: "📁 Add another", callback_data: "proj:add" }, { text: "📋 Projects", callback_data: "nav:projects" }], [back("projects")]]));
  if (ns === "proj" && action === "addname") { setPending(from, "add-project"); return edit(chat, msgId, "<b>📁 Manage a project</b>\nSend the <b>name</b> of the project to manage.", kb([[{ text: "✖ Cancel", callback_data: "acct:cancel" }], [back("projects")]])); }
  if (ns === "proj" && action === "open") {
    const mp = loadProjects()[arg];
    const tgOn = mp ? mp.telegram : true;
    const abots = loadAgentBots();
    const bot = abots[projId(arg)] || abots[arg];
    const botLine = bot ? `\n🤖 Dedicated bot: <b>linked ✅</b> (whitelisted to you)` : `\n🤖 Dedicated bot: <i>none</i>`;
    // Primary action: dispatch a mission to the project's dedicated oracle. Plus the
    // Telegram toggle, the dedicated-bot link, Git, and Delete.
    return edit(chat, msgId, `<b>${tgOn ? "📦" : "🔕"} ${esc(arg)}</b>${mp ? `\n<i>${esc(mp.category || "")}</i> · <code>${esc(mp.dir || "")}</code>\nTelegram: ${tgOn ? "🔔 <b>ON</b> (synced topic + shown)" : "🔕 <b>OFF</b> (no topic, dimmed)"}${botLine}` : ""}`, kb([
      [{ text: "🚀 Dispatch mission", callback_data: `proj:oracle:${arg}`.slice(0, 64) }],
      [{ text: tgOn ? "🔕 Telegram: turn OFF" : "🔔 Telegram: turn ON", callback_data: `proj:tg${tgOn ? "off" : "on"}:${arg}`.slice(0, 64) }],
      [{ text: bot ? "🤖 Dedicated bot — manage" : "🔗 Link a Telegram bot", callback_data: `proj:${bot ? "bot" : "botlink"}:${arg}`.slice(0, 64) }],
      [{ text: "🔧 Git", callback_data: `git:menu:${arg}`.slice(0, 64) }, { text: "🗑 Delete", callback_data: `proj:del:${arg}`.slice(0, 64) }],
      [back("projects")],
    ]));
  }
  if (ns === "proj" && (action === "tgon" || action === "tgoff")) {
    const enable = action === "tgon";
    setProjectTelegram(arg, enable);
    // Reconcile the topic now: OFF deletes the existing topic; ON lets /sync recreate it.
    let note = "";
    if (!enable) { const r = await removeProjectTopic(arg); note = r === "deleted" ? "\n💬 Topic removed." : r === "none" ? "" : `\n💬 Topic: ⚠️ ${esc(r)}`; }
    else note = "\nRun <code>/sync</code> in the hub to (re)create its topic.";
    return edit(chat, msgId, card("TELEGRAM TOGGLE", ` ${enable ? "🔔" : "🔕"} <b>${esc(arg)}</b> — Telegram <b>${enable ? "ON" : "OFF"}</b>${note}`), kb([
      [{ text: enable ? "🔕 Turn OFF" : "🔔 Turn ON", callback_data: `proj:tg${enable ? "off" : "on"}:${arg}`.slice(0, 64) }],
      [{ text: "« Project", callback_data: `proj:open:${arg}`.slice(0, 64) }, { text: "📋 Projects", callback_data: "nav:projects" }],
    ]));
  }
  // Link a dedicated Telegram bot to a project. SECURITY: the agent bot is whitelisted
  // to the operator's allow-list (ALLOW). Reaching here means the caller is already
  // whitelisted; if the allow-list is somehow empty, we ask for the id first (never
  // serve a VPS-controlling bot without a whitelist) — if present, we don't re-ask.
  if (ns === "proj" && action === "botlink") {
    if (ALLOW.length === 0) {
      // Structurally unreachable (the bot refuses to serve with an empty allow-list),
      // but kept as a guard: never wire a VPS-controlling bot without a whitelist.
      return edit(chat, msgId, card("WHITELIST FIRST", ` 🔒 No operator user id is whitelisted. For safety (this bot controls the VPS), set <code>allow_user_ids=[&lt;your_id&gt;]</code> in <code>${esc(TG_TOML)}</code>, then try again.`), kb([[{ text: "« Project", callback_data: `proj:open:${arg}`.slice(0, 64) }]]));
    }
    setPending(from, "tg-link", arg);
    return edit(chat, msgId, card("LINK A TELEGRAM BOT", ` 🔗 <b>${esc(arg)}</b>\n1) Create a bot via @BotFather (or reuse one).\n2) Send its <b>token</b> here (<code>123456:ABC…</code>).\n\n🔒 It will be <b>whitelisted to you alone</b> (id ${esc(ALLOW.join(", "))}) — nobody else can use it. Talking to it = addressing this project's oracle, scoped to it.`), kb([[{ text: "✖ Cancel", callback_data: "acct:cancel" }], [{ text: "« Project", callback_data: `proj:open:${arg}`.slice(0, 64) }]]));
  }
  if (ns === "proj" && action === "bot") {
    const abots = loadAgentBots();
    const bot = abots[projId(arg)] || abots[arg];
    if (!bot) return edit(chat, msgId, card("DEDICATED BOT", ` 🤖 <b>${esc(arg)}</b> — no bot linked.`), kb([[{ text: "🔗 Link a Telegram bot", callback_data: `proj:botlink:${arg}`.slice(0, 64) }], [{ text: "« Project", callback_data: `proj:open:${arg}`.slice(0, 64) }]]));
    return edit(chat, msgId, card("DEDICATED BOT", ` 🤖 <b>${esc(arg)}</b> — bot <b>linked ✅</b>\n 🔒 Whitelisted to id ${esc((bot.allow || ALLOW).join(", ") || "?")} (you only).\n Talk to it = this project's oracle.`), kb([
      [{ text: "🔁 Change token", callback_data: `proj:botlink:${arg}`.slice(0, 64) }, { text: "🛑 Unlink", callback_data: `proj:botunlink:${arg}`.slice(0, 64) }],
      [{ text: "« Project", callback_data: `proj:open:${arg}`.slice(0, 64) }],
    ]));
  }
  if (ns === "proj" && action === "botunlink") {
    const id = projId(arg);
    const abots = loadAgentBots();
    if (abots[id] || abots[arg]) { delete abots[id]; delete abots[arg]; saveAgentBots(abots); teardownAgentBot(id); }
    return edit(chat, msgId, card("DEDICATED BOT", ` 🛑 <b>${esc(arg)}</b> — bot unlinked + stopped ✅`), kb([[{ text: "« Project", callback_data: `proj:open:${arg}`.slice(0, 64) }, { text: "📋 Projects", callback_data: "nav:projects" }]]));
  }
  if (ns === "proj" && action === "del") { const m = projDeleteMenu(arg); return edit(chat, msgId, m.text, m.markup); }
  // 1️⃣ Remove from OmegaOS — non-destructive to code, no extra confirm.
  if (ns === "proj" && action === "delomega") return edit(chat, msgId, await deleteProject(arg, "omega"), kb([[{ text: "📋 Projects", callback_data: "nav:projects" }]]));
  // 2️⃣ Delete local machine — extra confirm for the irreversible local-folder rm.
  if (ns === "proj" && action === "dellocal") {
    const d = loadProjects()[arg]?.dir || "";
    return edit(chat, msgId, card("DELETE LOCAL MACHINE", ` 💻 <b>${esc(arg)}</b>\n⚠️ Removes it from OmegaOS <b>and deletes the local folder</b>${d ? ` <code>${esc(d)}</code>` : ""} off the VPS (irreversible). GitHub is kept. Sure?`), kb([
      [{ text: "💻 Yes, delete local machine", callback_data: `proj:dellocalgo:${arg}`.slice(0, 64) }],
      [{ text: "✖ Cancel", callback_data: `proj:open:${arg}`.slice(0, 64) }],
    ]));
  }
  if (ns === "proj" && action === "dellocalgo") return edit(chat, msgId, await deleteProject(arg, "local"), kb([[{ text: "📋 Projects", callback_data: "nav:projects" }]]));
  // 3️⃣ Delete all (+ GitHub) — extra confirm for the irreversible GitHub deletion.
  if (ns === "proj" && action === "delall") {
    const d = loadProjects()[arg]?.dir || "";
    return edit(chat, msgId, card("DELETE EVERYTHING", ` 💥 <b>${esc(arg)}</b>\n⚠️ Removes it from OmegaOS, <b>deletes the local folder</b>${d ? ` <code>${esc(d)}</code>` : ""} <b>AND the GitHub repo</b> (both irreversible). Nothing remains. Sure?`), kb([
      [{ text: "💥 Yes, delete EVERYTHING", callback_data: `proj:delallgo:${arg}`.slice(0, 64) }],
      [{ text: "✖ Cancel", callback_data: `proj:open:${arg}`.slice(0, 64) }],
    ]));
  }
  if (ns === "proj" && action === "delallgo") return edit(chat, msgId, await deleteProject(arg, "all"), kb([[{ text: "📋 Projects", callback_data: "nav:projects" }]]));
  if (ns === "proj" && action === "oracle") {
    setPending(from, "oracle-prompt", arg);
    return edit(chat, msgId, `<b>🔮 Oracle — ${esc(arg)}</b>\nSend your <b>prompt / mission</b>. I hand it to the dedicated oracle of <b>${esc(arg)}</b> (full reprompting: project knowledge + the whole OmegaOS doctrine — orchestration, dynamic workflows, workers, goals, audits) — scoped to this project.`, kb([[{ text: "✖ Cancel", callback_data: "acct:cancel" }], [{ text: "« Project", callback_data: `proj:open:${arg}`.slice(0, 64) }]]));
  }
  // Reply to a finished report: the next message continues this project's oracle, with
  // the conversation history (incl. the report just sent) as context.
  if (ns === "rep" && action === "reply") {
    setPending(from, "oracle-prompt", arg);
    return send(chat, `↩ <b>Réponse à ${esc(arg)}</b>\nÉcris ta suite — je la donne à l'oracle <b>${esc(arg)}</b> avec le contexte de la conversation (rapport inclus).`, kb([[{ text: "✖ Annuler", callback_data: "acct:cancel" }]]));
  }
  if (ns === "git" && action === "list") {
    const repos = gitRepos();
    if (!repos.length) return edit(chat, msgId, "<b>🔧 Git</b>\nNo git repo found under the projects root.", kb([[back("projects")]]));
    const rows: Btn[][] = [];
    for (let i = 0; i < repos.length; i += 2) rows.push(repos.slice(i, i + 2).map(r => ({ text: `📦 ${r.name}`.slice(0, 28), callback_data: `git:menu:${r.name}`.slice(0, 64) })));
    return edit(chat, msgId, `<b>🔧 Git — ${repos.length} repo(s)</b>\nPick a project for pull / add+push / status.`, kb([...rows, [back("projects")]]));
  }
  if (ns === "git" && action === "menu") return edit(chat, msgId, `<b>🔧 Git — ${esc(arg)}</b>\nPull (ff-only) / Add+Commit+Push / Status.`, gitMenuKb(arg));
  if (ns === "git" && action === "pull") return edit(chat, msgId, pre(`🔄 Pull ${arg}`, gitPull(arg)), gitMenuKb(arg));
  if (ns === "git" && action === "push") return edit(chat, msgId, pre(`⬆️ Add+Push ${arg}`, gitPush(arg)), gitMenuKb(arg));
  if (ns === "git" && action === "status") return edit(chat, msgId, pre(`📊 Status ${arg}`, gitStatus(arg)), gitMenuKb(arg));
  if (ns === "aud" && action === "run") return edit(chat, msgId, pre(`Audit: ${arg}`, await omega(["audit", "run", arg])), kb([[back("audits")]]));
  if (ns === "acct" && action === "login") return startLogin(chat, msgId, from, false);
  if (ns === "acct" && action === "switch") return startLogin(chat, msgId, from, true);
  if (ns === "acct" && action === "email") return edit(chat, msgId, await accountStatus(), kb([[{ text: "🔐 Login", callback_data: "acct:login" }, { text: "🔄 Switch", callback_data: "acct:switch" }], [back("account")]]));
  if (ns === "acct" && action === "usage") return edit(chat, msgId, pre("Usage tokens", await omega(["usage"])), kb([[{ text: "🔄 Refresh", callback_data: "acct:usage" }, back("account")]]));
  if (ns === "acct" && action === "billing") return edit(chat, msgId, pre("Billing", await omega(["monitor"])), kb([[back("account")]]));
  if (ns === "acct" && action === "accounts") return edit(chat, msgId, await serviceAccounts(), kb([[{ text: "🔄 Refresh", callback_data: "acct:accounts" }, back("account")]]));
  if (ns === "acct" && action === "cancel") { clearPending(from); return edit(chat, msgId, "Cancelled.", kb([[back("account")]])); }
  if (ns === "do" && action === "killall") return edit(chat, msgId, pre("kill-all", await omega(["kill-all", "--yes"])), kb([[back("clean")]]));
  if (ns === "do" && action === "novaup") return edit(chat, msgId, card("🤖 NOVA OS — démarrage…", await novaosCtl("start")), kb([[{ text: "🔄 Rafraîchir", callback_data: "nav:novaos" }, back("menu")]]));
  if (ns === "do" && action === "novadown") return edit(chat, msgId, card("🛑 NOVA OS — coupé", await novaosCtl("stop")), kb([[{ text: "🟢 Rallumer", callback_data: "do:novaup" }, back("menu")]]));
  if (ns === "do" && action === "clean") return edit(chat, msgId, pre("cleanup", await omega(["cleanup", "--yes"])), kb([[back("clean")]]));
  if (ns === "do" && action === "diskclean") {
    const cmd = "echo '== Docker build cache =='; docker builder prune -f 2>&1 | tail -1; echo '== APT =='; apt-get clean && echo ok; echo '== journal =='; journalctl --vacuum-size=100M 2>&1 | tail -1; echo '== caches user =='; bash " + OMEGA_DIR + "/skills/cleanup/scripts/clean-caches.sh 2>/dev/null | grep -i purg; echo; df -h / | tail -1";
    const out = await $`sudo bash -c ${cmd}`.nothrow().text();
    return edit(chat, msgId, pre("🧹 Nettoyage disque — terminé", out || "(no output)"), kb([[back("clean")]]));
  }
  if (ns === "do" && action === "diskanalyze") {
    const out = await $`bash -c ${"df -h / | tail -1; echo; du -shx /home/*/Station /tmp 2>/dev/null | sort -rh | head -6"}`.nothrow().text();
    return edit(chat, msgId, pre("📊 Analyse disque", out || "(no output)"), kb([[back("clean")]]));
  }
  if (ns === "do" && action === "tidy") {
    const script = `for d in $(find ${homedir()}/Station -maxdepth 3 -name .git -type d 2>/dev/null | sed 's#/.git$##' | grep -vE 'node_modules|/target/' | sort); do bash ${OMEGA_DIR}/skills/project-tidy/scripts/tidy-apply.sh "$d" 2>/dev/null | head -1; done`;
    const out = await $`bash -c ${script}`.nothrow().text();
    return edit(chat, msgId, pre("🗂️ Plan de rangement", out || "(rien)"), kb([[back("clean")]]));
  }
  if (ns === "do" && action === "ramflush") {
    // bun ne peut pas exec sudo (setuid) → on déclenche un helper root via fichier-trigger
    // (agentik-ramflush.path → agentik-ramflush.service écrit le rapport dans state/ramflush.out).
    const outf = `${OMEGA_DIR}/state/ramflush.out`, trig = `${OMEGA_DIR}/state/.ramflush-trigger`;
    try { writeFileSync(outf, ""); } catch {}
    try { writeFileSync(trig, String(Date.now())); } catch {}
    let out = "";
    for (let i = 0; i < 12; i++) { await Bun.sleep(600); try { const t = readFileSync(outf, "utf8"); if (t.trim().length > 40) { out = t; break; } } catch {} }
    return edit(chat, msgId, pre("💾 Purge RAM — terminé", out || "⏳ déclenché — résultat indisponible (helper agentik-ramflush actif ?)"), kb([[back("clean")]]));
  }
  if (ns === "agent" && action === "info") { const a = (await mcAgents()).find(x => x.id === arg); return edit(chat, msgId, `<b>🤖 ${esc(arg)}</b>\n${esc(a?.description || "(no description)")}\n\n<i>Link a dedicated Telegram bot to this agent — you'll talk to it directly (scoped to its project).</i>`, kb([[{ text: "🔗 Link Telegram", callback_data: `agent:tglink:${arg}`.slice(0, 64) }], [back("agents")]])); }
  if (ns === "agent" && action === "tglink") {
    setPending(from, "tg-link", arg);
    const body = /^(nova|companion)$/i.test(arg)
      ? `<b>💞 Link your companion (Nova)</b>\n1) Create her bot via @BotFather (<code>/newbot</code> — pick her name).\n2) Send me its <b>token</b> here (format <code>123456:ABC…</code>).\n\nShe'll be <b>whitelisted to you alone</b>: a personal assistant who chats from your life store, remembers you, and hands heavy project work to Atlas.`
      : /^(trinity|security)$/i.test(arg)
      ? `<b>🛡 Link your security agent (Trinity)</b>\n1) Create a bot via @BotFather (<code>/newbot</code>).\n2) Send me its <b>token</b> here (format <code>123456:ABC…</code>).\n\nIt'll be <b>whitelisted to you alone</b>: a white-hat security operator (offensive + defensive), pre-authorized for your in-scope work — recon → scan → exploit/PoC → report, with non-negotiable hard limits. Point it only at assets you own or are contracted to test.`
      : `<b>🔗 Link a Telegram bot — ${esc(arg)}</b>\n1) Create a bot via @BotFather (or reuse one).\n2) Send me its <b>token</b> here (format <code>123456:ABC…</code>).\n\nThe bot will be <b>whitelisted to you alone</b>, and when you talk to it you'll be addressing the oracle of project <b>${esc(arg)}</b> (scoped to this project only).`;
    return edit(chat, msgId, body, kb([[{ text: "✖ Cancel", callback_data: "acct:cancel" }], [back("agents")]]));
  }
  return edit(chat, msgId, menuText, menuKb());
}

// ── group setup: verify the bot is admin, register the supergroup as hub ─────
async function cmdSetupGroup(chat: any, chatId: number, thread?: number) {
  if (chat.type !== "group" && chat.type !== "supergroup") return send(chatId, "⚠️ Run <code>/setupgroup</code> <b>in the group</b> (a supergroup with Topics enabled).", undefined, thread);
  // getChat to read the live is_forum (the message's chat object can be stale).
  const info = await tg("getChat", { chat_id: chatId });
  const isForum = info.ok ? !!info.result?.is_forum : !!chat.is_forum;
  const admins = await tg("getChatAdministrators", { chat_id: chatId });
  const me = admins.ok ? admins.result.find((a: any) => a.user?.id === BOT_ID) : null;
  if (!me) return send(chatId, "⚠️ I'm <b>not admin</b> here. Add me as <b>administrator</b> with the <b>“Manage Topics”</b> permission, then re-run <code>/setupgroup</code>.", undefined, thread);
  const canTopics = me.status === "creator" || me.can_manage_topics === true;
  const g = loadGroups(); g.hub = chatId; g.isForum = isForum; g.topics ||= {}; saveGroups(g);
  let msg = "✅ Group registered as the <b>project hub</b>.";
  if (!isForum) msg += "\n⚠️ <b>Topics are not enabled</b> — enable them in the group settings, then re-run /setupgroup.";
  else if (!canTopics) msg += "\n⚠️ I'm missing the <b>“Manage Topics”</b> permission: open my admin rights (you → group → admins → this bot) and enable <b>Manage Topics</b>, then run <code>/sync</code>.";
  else msg += "\n✅ Topics enabled + rights OK — run <code>/sync</code> to create one topic per project.";
  return send(chatId, msg, undefined, thread);
}

// ── sync: one forum topic per project; route topic messages to its oracle ────
async function cmdSync(chatId: number, thread?: number) {
  const g = loadGroups();
  if (!g.hub) return send(chatId, "⚠️ No hub — first run <code>/setupgroup</code> in your supergroup.", undefined, thread);
  if (!g.isForum) return send(chatId, "⚠️ Topics are not enabled — enable them, re-run /setupgroup, then /sync.", undefined, thread);
  g.topics ||= {};
  // Ensure the RESERVED topics exist and are ALIVE (recreate if deleted in the group):
  //  • "atlas"  — Atlas conversation + oracle reports for off-project work.
  //  • "alerts" — operational alerts (stuck oracle / self-heal / token refresh);
  //               undeletable by design: /sync (and omega-alert-send.sh on send)
  //               recreates it so alerts always have a home.
  const ensureReserved = async (key: "atlas_topic" | "alerts_topic", name: string, title: string, color: number) => {
    let tid = g[key] && Object.keys(g.topics!).includes(String(g[key])) ? g[key]
      : Number(Object.entries(g.topics!).find(([, n]) => String(n).toLowerCase() === name)?.[0]) || undefined;
    if (tid) {
      // Liveness probe: same-name rename → ok/NOT_MODIFIED = alive; TOPIC_ID_INVALID = deleted.
      const probe = await tg("editForumTopic", { chat_id: g.hub, message_thread_id: tid, name: title });
      if (probe.ok || /not.?modified/i.test(probe.description || "")) { g[key] = tid; g.topics![String(tid)] = name; return; }
      if (!/TOPIC_ID_INVALID|not found/i.test(probe.description || "")) { g[key] = tid; return; } // ambiguous → keep
      delete g.topics![String(tid)]; // provably deleted → recreate below
    }
    const r = await tg("createForumTopic", { chat_id: g.hub, name: title, icon_color: color });
    if (r.ok) { g[key] = r.result.message_thread_id; g.topics![String(r.result.message_thread_id)] = name; saveGroups(g); }
  };
  await ensureReserved("atlas_topic", "atlas", "Atlas 🎩", 7322096);
  await ensureReserved("alerts_topic", "alerts", "Alerts 🚨", 16478047);
  const mp = loadProjects();
  const names = Object.keys(mp);
  if (!names.length) return send(g.hub, "No managed project — add (📁) or create (➕) a project, then /sync.", undefined, thread);
  let made = 0; let recreated = 0; let removed = 0; let skipped = 0; let err = "";
  for (const p of names) {
    // Reverse-lookup the project's currently-mapped topic id (if any).
    const mappedTid = Object.entries(g.topics).find(([, n]) => String(n).toLowerCase() === p.toLowerCase())?.[0];
    // Telegram toggle OFF → this project opts out of topics: delete its topic if it
    // has one, and never create one. (The toggle is the desired state; sync reconciles.)
    if (!mp[p].telegram) {
      if (mappedTid) {
        const d = await tg("deleteForumTopic", { chat_id: g.hub, message_thread_id: Number(mappedTid) });
        if (d.ok || /not found|thread not found/i.test(d.description || "")) { delete g.topics[mappedTid]; removed++; }
        else if (/rights|manage/i.test(d.description || "")) { err = d.description || "rights"; break; }
      }
      skipped++;
      continue;
    }
    if (mappedTid) {
      // Verify the topic STILL EXISTS on Telegram — a no-op rename probes it. On a
      // LIVE topic the same-name rename returns 400 TOPIC_NOT_MODIFIED (an "error"
      // that actually proves the topic is alive); a deleted topic returns
      // TOPIC_ID_INVALID / "thread not found". One topic per project: recreate ONLY
      // when the topic is provably gone — any other error keeps the mapping (never
      // duplicate on an ambiguous probe).
      const probe = await tg("editForumTopic", { chat_id: g.hub, message_thread_id: Number(mappedTid), name: p.slice(0, 128) });
      if (probe.ok || /not.?modified/i.test(probe.description || "")) continue; // topic alive → keep
      if (/rights|manage/i.test(probe.description || "")) { err = probe.description || "rights"; break; }
      if (!/TOPIC_ID_INVALID|not found/i.test(probe.description || "")) { console.log(`sync: ambiguous probe on ${p} (${probe.description}) — keeping mapping`); continue; } // ambiguous → keep mapping, no duplicate
      delete g.topics[mappedTid]; // stale mapping (topic deleted) → drop + recreate below
    }
    const r = await tg("createForumTopic", { chat_id: g.hub, name: p.slice(0, 128) });
    if (r.ok) { g.topics[String(r.result.message_thread_id)] = p; recordProject(p, mp[p].dir || "", undefined, r.result.message_thread_id); if (mappedTid) recreated++; else made++; }
    else { err = r.description || "failed"; break; }
  }
  saveGroups(g);
  if (err) return send(g.hub, `⚠️ Sync interrupted: <i>${esc(err)}</i>.${/rights|manage/i.test(err) ? "\nEnable the <b>“Manage Topics”</b> permission for the bot (group admin), then re-run /sync." : ""}\n(${made} created, ${recreated} recreated before stopping)`, undefined, thread);
  const offNote = removed ? `, ${removed} removed (🔕 OFF)` : skipped ? `, ${skipped} skipped (🔕 OFF)` : "";
  await refreshCommands().catch(() => {}); // refresh /{project} commands alongside topics
  return send(g.hub, `🔁 Sync OK. ${made} new topic(s)${recreated ? `, ${recreated} recreated (deleted topics detected)` : ""}${offNote}; ${Object.keys(g.topics).length} project topic(s) total. Messages in a project's topic are routed to its oracle.`, undefined, thread);
}

// ── AGENT MODE poll loop: a per-agent bot. Whitelisted to the operator; every
// message goes straight to that project's scoped oracle (no menu, no other project).
// ── Companion (Nova) inline menu + callbacks: the operator's button menu on his
// phone. Connection buttons run Composio and hand back a tappable OAuth URL;
// directive buttons inject a brief into Nova's brain (same as the slash commands).
const NOVA_LIFE = `${homedir()}/Station/LifeStyle`;
// Live voice call (ElevenLabs Conversational AI): the machine opts in by
// writing state/nova-call.json {url, label?} — no file, no button. The URL
// opens the agent's talk-to page; Telegram URL buttons launch it in one tap.
function novaCallButton(): any[][] {
  try {
    const c = JSON.parse(readFileSync(`${OMEGA_DIR}/state/nova-call.json`, "utf8"));
    const rows: any[][] = [];
    // web_app opens the call INSIDE Telegram (Mini App) — private chats only,
    // which is all a companion bot ever serves. url is the browser fallback.
    if (c?.web_app) rows.push([{ text: c.webAppLabel || "📞 Appel vocal (dans Telegram)", web_app: { url: c.web_app } }]);
    if (c?.url) rows.push([{ text: c.label || "🌐 Appel vocal (navigateur)", url: c.url }]);
    return rows;
  } catch {}
  return [];
}
function novaMenuKb() {
  return kb([
    ...novaCallButton(),
    [{ text: "🔌 Connecter mes comptes", callback_data: "nova:connect" }],
    [{ text: "📰 Actus Anthropic", callback_data: "nova:do:actus" }, { text: "📊 Rapport now", callback_data: "nova:do:rapport" }],
    [{ text: "🎯 Objectifs", callback_data: "nova:do:objectifs" }, { text: "🧠 Profil", callback_data: "nova:do:profil" }],
    [{ text: "🧭 KAIROS", callback_data: "nova:kairos" }],
    [{ text: "🔮 Magic", callback_data: "nova:do:magic" }, { text: "🗣️ Interview", callback_data: "nova:do:interview" }],
    [{ text: "🔊 Voix (mode + moteur)", callback_data: "nova:voice" }],
  ]);
}
// 🔊 Voice submenu: live engine list from the omega-ttsd gateway (🟢 installed /
// 🔴 unavailable), current mode + engine marked, one-tap test voice note.
async function novaVoiceView(botName: string): Promise<{ text: string; markup: any }> {
  const vp = voicePrefs();
  let engines: { id: string; label: string; note: string; available: boolean }[] = [];
  try { engines = await (await fetch(`${TTSD}/engines`, { signal: AbortSignal.timeout(3000) })).json() as any; } catch {}
  const modeBtn = (m: VoicePrefs["mode"], label: string) => ({ text: `${vp.mode === m ? "● " : ""}${label}`, callback_data: `nova:vmode:${m}` });
  const markup = kb([
    [modeBtn("text", "📝 Texte"), modeBtn("voice", "🎙️ Vocal"), modeBtn("both", "📝+🎙️ Les deux")],
    ...engines.map(e => [{ text: `${vp.engine === e.id ? "✓ " : ""}${e.available ? "" : "🔴 "}${e.label}`, callback_data: `nova:vengine:${e.id}` }]),
    [{ text: "🧪 Tester la voix sélectionnée", callback_data: "nova:vtest" }],
    ...novaCallButton(),
    [{ text: "« Retour", callback_data: "nova:menu" }],
  ]);
  const lines = engines.length
    ? engines.map(e => `${e.available ? "🟢" : "🔴"} <b>${esc(e.label)}</b> — ${esc(e.note)}`).join("\n")
    : "⚠️ Le démon TTS ne répond pas — relance-le : <code>systemctl --user restart omega-ttsd</code>";
  const modeLabel = { text: "📝 texte seul", voice: "🎙️ vocal seul", both: "📝+🎙️ les deux" }[vp.mode];
  const voiceLine = vp.voiceLabel ? ` · Voix : <b>${esc(vp.voiceLabel)}</b>` : "";
  return {
    text: `<b>🔊 Voix de ${esc(botName)}</b>\nMode : <b>${modeLabel}</b> · Moteur : <b>${esc(vp.engine)}</b>${voiceLine}\n\n${lines}\n\n💡 Choisis une voix du casting en m'écrivant <code>voix 16</code>\n💡 Pour activer ElevenLabs, colle ta clé ici : <code>clé elevenlabs: sk_…</code>`,
    markup,
  };
}
function novaConnectKb() {
  const apps = ["gmail", "twitter", "instagram", "linkedin", "reddit", "youtube"];
  const label: Record<string, string> = { gmail: "📧 Gmail", twitter: "🐦 Twitter/X", instagram: "📸 Instagram", linkedin: "💼 LinkedIn", reddit: "👽 Reddit", youtube: "▶️ YouTube" };
  return kb([
    ...apps.map(a => [{ text: `Connecter ${label[a]}`, callback_data: `nova:conn:${a}` }]),
    [{ text: "« Retour", callback_data: "nova:menu" }],
  ]);
}
// Directive text injected into Nova's brain for each menu button (mirrors the
// slash commands the persona already knows).
const NOVA_DIRECTIVE: Record<string, string> = {
  actus: "/actus — donne-moi les actualités du jour autour d'Anthropic (modèles, Claude, recherche, produits, sécurité, business). C'est mon outil n°1 : va chercher du frais sur le web (WebSearch/WebFetch), 4-6 items max, chacun 1-2 lignes + la source. Termine par 1 implication concrète pour moi.",
  rapport: "/rapport — fais-moi un briefing maintenant (météo, focus du jour, 1 challenge, 1 question).",
  objectifs: "/objectifs — affiche mes objectifs (~/Station/LifeStyle/About/00-OBJECTIFS.md, crée-le s'il manque) et challenge-moi sur 1 point.",
  profil: "/profil — résume ce que tu sais de moi (identité, objectifs, état des sections de About/).",
  magic: "/magic — synthèse de ma Matrice de Destinée (~/Station/LifeStyle/About/08-MagicLife/).",
  interview: "/interview — re-questionne-moi pour détecter ce qui a changé depuis la dernière fois. Commence par UNE question, on enchaîne.",
};
// Run a Composio connection for `app`, return the tappable OAuth URL (or an error
// the operator can act on — typically: paste your Composio API key first).
async function novaComposioConnect(app: string): Promise<{ url?: string; msg: string }> {
  const r = Bun.spawnSync(["bash", `${OMEGA_DIR}/bin/nova-composio-connect.sh`, app], { env: { ...process.env, HOME: homedir() } });
  const out = (r.stdout.toString() + r.stderr.toString()).trim();
  const url = (out.match(/https?:\/\/\S+/) || [])[0];
  if (url) return { url, msg: `🔐 Autorise <b>${esc(app)}</b> depuis ton tél — appuie sur le bouton. Multi-comptes : reconnecte le même service avec un autre compte.` };
  if (/API_KEY manquante|COMPOSIO_API_KEY/i.test(out)) return { msg: `🔑 Il me faut d'abord ta <b>clé Composio</b>. Récupère-la sur app.composio.dev (Settings → API Keys) et colle-la moi ici (ex: « ma clé composio: xxxx ») — je la range et on réessaie.` };
  return { msg: `⚠️ Composio n'a pas renvoyé d'URL pour ${esc(app)} :\n<pre>${esc(out).slice(0, 400)}</pre>` };
}
// ── KAIROS (Nova ⇄ KairosOS shared store) ────────────────────────────────────
// Gareth's KairosOS growth dashboard and Nova read/write the SAME Convex blob via
// the ~/.omega/bin/nova-kairos.sh bridge (get | set-field [--json] <dotpath> <value>).
// Everything here is additive: a /kairos card, per-field inline edit, French
// natural-language intents, and a /kairos update shorthand — all funnelled through
// that one bridge. Values are esc()'d into HTML; bridge args are passed as argv
// elements (never a shell string), so operator text can never inject.
const NOVA_KAIROS = `${OMEGA_DIR}/bin/nova-kairos.sh`;
type KairosField = { key: string; dot: string; label: string };
// key = the SHORT callback token (≤64-byte data); dot = the nova-kairos dotpath.
// Labels are copied verbatim from the KairosOS app so Nova mirrors the dashboard.
const KAIROS_FIELDS: KairosField[] = [
  { key: "objectifUltime", dot: "objectifUltime", label: "🎯 Objectif ultime" },
  { key: "objectif", dot: "objectif", label: "🧭 Objectif" },
  { key: "mantra", dot: "mantra", label: "🔱 Mantra" },
  { key: "richesse", dot: "vision.richesse", label: "Richesse et liberté" },
  { key: "idees", dot: "vision.idees", label: "Vision et idées" },
  { key: "musique", dot: "vision.musique", label: "Musique électronique" },
  { key: "presence", dot: "vision.presence", label: "Présence et charisme" },
  { key: "femme", dot: "vision.femme", label: "La relation que j'attire" },
  { key: "temps", dot: "vision.temps", label: "Temps et bonheur" },
  { key: "positive", dot: "vision.positive", label: "Pensée positive" },
  { key: "interessant", dot: "vision.interessant", label: "Intéressant et intéressé" },
  { key: "adore", dot: "vision.adore", label: "Lien et sympathie" },
  { key: "exploration", dot: "vision.exploration", label: "Exploration et découverte" },
];
type Levier = { id: string; n: string; label: string; desc: string };
const KAIROS_LEVIERS: Levier[] = [
  { id: "clarte", n: "01", label: "Clarté", desc: "Objectif chirurgical" },
  { id: "amorcage", n: "02", label: "Amorçage", desc: "Visualisation quotidienne" },
  { id: "prophetie", n: "03", label: "Prophétie", desc: "Posture de celui qui gagne" },
  { id: "action", n: "04", label: "Action", desc: "Surface de chance" },
  { id: "environnement", n: "05", label: "Environnement", desc: "Fréquentations et inputs" },
];
const kairosFieldByKey = (key: string) => KAIROS_FIELDS.find(f => f.key === key);
const kairosLabelForDot = (dot: string) => KAIROS_FIELDS.find(f => f.dot === dot)?.label || dot;
// Normalize a French word for fuzzy matching: lowercase, strip accents + apostrophes.
const kairosNorm = (s: string) => s.toLowerCase().normalize("NFD").replace(/[̀-ͯ]/g, "").replace(/['’]/g, "").trim();

// ── read/write through the bridge (argv array → no shell injection) ──
function kairosGet(): any | null {
  try {
    const r = Bun.spawnSync(["bash", NOVA_KAIROS, "get"], { env: { ...process.env, HOME: homedir() }, timeout: 25000 });
    if (r.exitCode !== 0) return null;
    return JSON.parse(r.stdout.toString());
  } catch { return null; }
}
function kairosSet(dot: string, value: string, json = false): { ok: boolean; err?: string } {
  try {
    const args = json
      ? ["bash", NOVA_KAIROS, "set-field", "--json", dot, value]
      : ["bash", NOVA_KAIROS, "set-field", dot, value];
    const r = Bun.spawnSync(args, { env: { ...process.env, HOME: homedir() }, timeout: 25000 });
    return r.exitCode === 0 ? { ok: true } : { ok: false, err: r.stderr.toString().trim() };
  } catch (e: any) { return { ok: false, err: String(e?.message || e) }; }
}

// ── format helpers (Telegram hard limit = 4096 chars; previews keep us ~1.7k) ──
function preview(s: string, n: number): string {
  const t = (s || "").replace(/\s+/g, " ").trim();
  return esc(t.length > n ? t.slice(0, n).trim() + "…" : t);
}
function kairosFull(s: string, n = 4000): string {
  const t = s || "";
  return esc(t.slice(0, n)) + (t.length > n ? "\n\n<i>…(tronqué)</i>" : "");
}

// ── KAIROS v2 — daily-ritual data model (mirror lib/state.ts EXACTLY) ──
// Local YYYY-MM-DD (the app's todayStr is local-tz; mirror it so a write lands
// on the same journal key the dashboard reads).
function kairosToday(): string {
  const n = new Date();
  return `${n.getFullYear()}-${String(n.getMonth() + 1).padStart(2, "0")}-${String(n.getDate()).padStart(2, "0")}`;
}
function kairosDateMinus(days: number): string {
  const d = new Date(); d.setDate(d.getDate() - days);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}
function kairosPrevDay(date: string): string {
  const [y, m, dd] = date.split("-").map(Number);
  const dt = new Date(y, m - 1, dd); dt.setDate(dt.getDate() - 1);
  return `${dt.getFullYear()}-${String(dt.getMonth() + 1).padStart(2, "0")}-${String(dt.getDate()).padStart(2, "0")}`;
}
// emptyDay mirror — write COMPLETE days so the app never has to default fields.
function kairosEmptyDay(): any {
  return { intentionRelue: false, amorcage: false, actionTexte: "", actionFaite: false, pareto: [], defi: { texte: "", fait: false }, signes: [], gratitude: ["", "", ""], sn: 0, note: "" };
}
// Read blob, merge a partial patch into today's Day, write the WHOLE day back
// (RMW so the Day stays complete) via set-field --json journal.<today> <day>.
function kairosPatchDay(patch: any): { ok: boolean; err?: string } {
  const d = kairosGet(); if (!d) return { ok: false, err: "store injoignable" };
  const today = kairosToday();
  const day = { ...kairosEmptyDay(), ...(d.journal?.[today] || {}), ...patch };
  return kairosSet(`journal.${today}`, JSON.stringify(day), true);
}
// piliers = the 3-pillar loop count (0..3); 3 = "BOUCLE COMPLÈTE".
function kairosPiliers(day: any): number {
  if (!day) return 0;
  return (day.intentionRelue ? 1 : 0) + (day.amorcage ? 1 : 0) + (day.actionFaite ? 1 : 0);
}
// Streak = consecutive days with actionFaite back from today (or yesterday if
// today is not yet done).
function kairosStreak(journal: any): number {
  const j = journal || {}; let count = 0; let cur = kairosToday();
  if (!j[cur]?.actionFaite) cur = kairosPrevDay(cur);
  while (j[cur]?.actionFaite) { count++; cur = kairosPrevDay(cur); }
  return count;
}
function kairosPrettyDate(): string {
  try { return new Date().toLocaleDateString("fr-FR", { weekday: "long", day: "numeric", month: "long" }); }
  catch { return kairosToday(); }
}

// ── vision-field synonym map: spoken word (normalized) → vision key ──
const KAIROS_VISION_SYNONYMS: Record<string, string[]> = {
  richesse: ["richesse", "argent", "money", "finance", "fortune", "liberte"],
  idees: ["idee", "opportunite", "creativite", "inspiration"],
  musique: ["musique", "music", "son", "dj", "prod", "production", "beat"],
  presence: ["presence", "charisme", "eloquence", "voix", "parole"],
  femme: ["femme", "relation", "amour", "couple", "partenaire", "relationnel"],
  temps: ["temps", "time", "bonheur", "repos", "equilibre"],
  positive: ["positive", "positivite", "mental", "mindset", "optimisme", "pensee"],
  interessant: ["interessant", "interesse", "curiosite", "curieux", "interet"],
  adore: ["lien", "sympathie", "social", "generosite", "gens"],
  exploration: ["exploration", "decouverte", "voyage", "nouveaute", "aventure", "explorer"],
};
// "liberte" is intentionally listed ONLY under richesse (not temps): a bare
// "liberté" resolves to richesse. Documented & acceptable.
function kairosVisionKey(word: string): string | undefined {
  const w = kairosNorm(word);
  const cand = [w, w.replace(/s$/, "")];  // try plural and singular (but not "temps"→"temp" losing a real key — "temps" stays a candidate)
  for (const [key, syns] of Object.entries(KAIROS_VISION_SYNONYMS))
    if (syns.some(s => cand.includes(s))) return key;
  return undefined;
}
// Parse a free phrase for known levier ids/labels/numbers → unique id array.
function kairosLevierIds(text: string): string[] {
  const norm = kairosNorm(text);
  const ids: string[] = [];
  for (const lv of KAIROS_LEVIERS) {
    const aliases = [lv.id, kairosNorm(lv.label), lv.n, lv.n.replace(/^0+/, "")].filter(Boolean);
    if (aliases.some(a => new RegExp(`\\b${a}\\b`).test(norm))) ids.push(lv.id);
  }
  return ids;
}

// ── TASK 3 — French natural-language intent → a typed write instruction ──
type KairosIntent = { dot?: string; value?: string; json?: boolean; label: string; kind: "set" | "levier" | "action" | "guide" | "day"; guide?: string; summary?: string };
function detectKairosIntent(text: string): KairosIntent | null {
  const t = text.trim();
  // A question goes to the brain, never a write ("mantra c'est quoi ?", "… ?").
  if (t.endsWith("?")) return null;
  // A captured value that is itself a question word is a question, not a new value.
  const isQ = (v: string) => /^(c'?est\s+)?(quoi|qui|quand|comment|pourquoi|o[uù]|combien|quel)\b/i.test(v.trim());
  const facets = "richesse, idées, musique, présence, femme, temps, positivité, intéressant, lien, exploration";
  const visionGuide = `Quelle facette ? ${facets}. Ex : « ma vision de la richesse est … »`;
  let m: RegExpMatchArray | null;
  // Objectif ultime BEFORE the generic objectif (anchored, explicit connector).
  m = t.match(/^\s*mon\s+objectif\s+ultime\s*(?:est|c'?est|:|=)\s*(.+)$/is);
  if (m && m[1].trim().length >= 2 && !isQ(m[1])) return { kind: "set", dot: "objectifUltime", value: m[1].trim(), label: "Objectif ultime" };
  m = t.match(/^\s*mon\s+objectif\s*(?:est|c'?est|:|=)\s*(.+)$/is);
  if (m && m[1].trim().length >= 2 && !isQ(m[1])) return { kind: "set", dot: "objectif", value: m[1].trim(), label: "Objectif" };
  m = t.match(/^\s*(?:mon\s+)?mantra\s*(?:est|c'?est|:|=)\s*(.+)$/is);
  if (m && m[1].trim().length >= 2 && !isQ(m[1])) return { kind: "set", dot: "mantra", value: m[1].trim(), label: "Mantra" };
  // Vision <champ> est X — capture the WHOLE champ phrase (lazy), then scan each
  // word for a known vision key (first hit wins); unknown champ → guide (no write).
  m = t.match(/^\s*ma\s+vision\s+(?:de\s+la\s+|de\s+l'?|de\s+|du\s+|des\s+|pour\s+|sur\s+|en\s+|envers\s+)?(.+?)\s*(?:est|c'?est|:|=)\s*(.+)$/is);
  if (m && m[2].trim().length >= 2 && !isQ(m[2])) {
    const key = m[1].split(/\s+/).map(w => kairosVisionKey(w)).find(Boolean);
    if (key) { const f = kairosFieldByKey(key)!; return { kind: "set", dot: f.dot, value: m[2].trim(), label: f.label }; }
    return { kind: "guide", label: "Vision", guide: visionGuide };
  }
  // Bare "ma vision est X" — ambiguous → guide, never a silent write.
  m = t.match(/^\s*ma\s+vision\s*(?:est|c'?est|:|=)\s*(.+)$/is);
  if (m) return { kind: "guide", label: "Vision", guide: visionGuide };
  // Leviers.
  m = t.match(/^\s*mes\s+leviers?\s*(?:sont|c'?est|:|=)\s*(.+)$/is);
  if (m) {
    const ids = kairosLevierIds(m[1]);
    if (ids.length) return { kind: "levier", value: JSON.stringify(ids), label: "Leviers" };
    return { kind: "guide", label: "Leviers", guide: "Tes 5 leviers : Clarté, Amorçage, Prophétie, Action, Environnement. Ex : « mes leviers sont clarté, action »" };
  }
  // Action du jour ("j'ai fait …").
  // Lookahead (not \b): verbs ending in an accented char (complété/terminé/réalisé)
  // have no \w boundary after "é", so \b would never fire on them.
  m = t.match(/^\s*(?:j'?ai|j\s+ai)\s+(?:fait|complété|complete|terminé|termine|fini|accompli|réalisé|realise)(?=\s|$)\s*(.+)$/is);
  if (m && m[1].trim().length >= 3) return { kind: "action", value: m[1].trim(), label: "Action du jour" };
  // ── KAIROS v2 — daily-ritual intents (confirm-gated). Each is RMW: compute the
  // merged today-Day NOW and carry it as one json op on journal.<today>. ──
  const dayIntent = (apply: (cur: any) => any, label: string, summary: string): KairosIntent => {
    const d = kairosGet(); const today = kairosToday();
    const cur = { ...kairosEmptyDay(), ...(d?.journal?.[today] || {}) };
    const merged = { ...cur, ...apply(cur) };
    return { kind: "day", dot: `journal.${today}`, value: JSON.stringify(merged), json: true, label, summary };
  };
  // "j'ai lu/relu mon mantra" → mantraReads+1 (BEFORE intention so it wins).
  if (/^\s*(?:j'?ai\s+)?(?:lu|relu)\s+(?:mon\s+)?mantra\b/i.test(t))
    return dayIntent(cur => ({ mantraReads: (cur.mantraReads || 0) + 1 }), "Lecture mantra", "Lecture du mantra +1");
  // amorçage fait.
  if (/^\s*(?:j'?ai\s+fait\s+(?:mon\s+)?)?amor[cç]age\b/i.test(t) || /amor[cç]age\s+(?:fait|ok|fini)/i.test(t))
    return dayIntent(() => ({ amorcage: true }), "Amorçage", "Amorçage fait");
  // intention / cap / objectif relu (AFTER mantra).
  if (/^\s*(?:j'?ai\s+)?(?:relu|lu)\s+(?:mon\s+)?(?:intention|cap|objectif)\b/i.test(t))
    return dayIntent(() => ({ intentionRelue: true }), "Intention", "Intention relue");
  // système nerveux 1-5.
  m = t.match(/^\s*(?:syst[èe]me\s+nerveux|sn)\s*(?:[:=]|à)?\s*([1-5])\b/i) || t.match(/^\s*je\s+suis\s+à\s+([1-5])\s*(?:\/\s*5)?\b/i);
  if (m) { const n = Number(m[1]); return dayIntent(() => ({ sn: n }), "Système nerveux", `SN : ${n}/5`); }
  // gratitude (split into ≤3).
  m = t.match(/^\s*(?:gratitude|reconnaissant|merci)\b.*?[:=]\s*(.+)$/i) || t.match(/^\s*je\s+suis\s+reconnaissant\b.*?(?:pour|de)\s+(.+)$/i);
  if (m && m[1].trim().length >= 2) {
    const g = m[1].split(/[,\n]/).map(s => s.trim()).filter(Boolean).slice(0, 3); while (g.length < 3) g.push("");
    return dayIntent(() => ({ gratitude: g }), "Gratitude", `«${preview(g.filter(Boolean).join(", "), 120)}»`);
  }
  // défi du jour.
  m = t.match(/^\s*(?:mon\s+)?d[ée]fi\s*(?:est|c'?est|:|=)\s*(.+)$/is);
  if (m && m[1].trim().length >= 2 && !isQ(m[1])) { const v = m[1].trim(); return dayIntent(() => ({ defi: { texte: v, fait: false } }), "Défi", `«${preview(v, 120)}»`); }
  // signe / synchronicité (append).
  m = t.match(/^\s*(?:signe|j'?ai\s+(?:vu|remarqué|remarque)|synchronicit[ée])\s*(?:[:=])?\s*(.+)$/is);
  if (m && m[1].trim().length >= 2 && !isQ(m[1])) {
    const v = m[1].trim();
    return dayIntent(cur => ({ signes: [...(Array.isArray(cur.signes) ? cur.signes : []), { id: Date.now(), texte: v, geste: "", active: true }] }), "Signe", `«${preview(v, 120)}»`);
  }
  // note du jour.
  m = t.match(/^\s*note\s*[:=]\s*(.+)$/is);
  if (m && m[1].trim().length >= 1) { const v = m[1].trim(); return dayIntent(() => ({ note: v }), "Note", `«${preview(v, 120)}»`); }
  return null;
}

// ── KAIROS v2 — chunked sender (Telegram hard limit 4096/msg) ──
async function sendChunks(chatId: number, blocks: string[], markup: any, thread?: number) {
  for (let i = 0; i < blocks.length; i++)
    await send(chatId, blocks[i], i === blocks.length - 1 ? markup : undefined, thread);
}
// ── FEATURE 1 — FULL vision, multi-message (no truncation) ──
// Block 1 = identity (objectif ultime + objectif + mantra). Then the 10 vision
// champs, packed on field boundaries into blocks each kept < 3900 chars.
function kairosVisionBlocks(): string[] {
  const d = kairosGet();
  if (!d) return ["⚠️ Store KAIROS injoignable."];
  const v = d.vision || {};
  const out: string[] = [];
  const b1 =
    `🧭 <b>KAIROS — ta vision</b>\n\n` +
    `🎯 <b>Objectif ultime</b>\n${d.objectifUltime ? esc(d.objectifUltime) : "—"}\n\n` +
    `🧭 <b>Objectif</b>\n${d.objectif ? esc(d.objectif) : "—"}\n\n` +
    `🔱 <b>Mantra</b>\n${d.mantra ? esc(d.mantra) : "—"}`;
  if (b1.length > 3900) for (let i = 0; i < b1.length; i += 3900) out.push(b1.slice(i, i + 3900));
  else out.push(b1);
  const visFields = KAIROS_FIELDS.filter(f => f.dot.startsWith("vision."));
  let blk = `🌅 <b>Vision</b>\n`;
  for (const f of visFields) {
    const piece = `\n<b>${esc(f.label)}</b>\n${v[f.key] ? esc(v[f.key]) : "—"}\n`;
    if (blk.length + piece.length > 3900) { out.push(blk.trimEnd()); blk = ""; }
    blk += piece;
  }
  if (blk.trim()) out.push(blk.trimEnd());
  return out;
}
// ── FEATURE 2 — KAIROS hub menu ──
function kairosMenuKb() {
  return kb([
    [{ text: "📖 Ma vision", callback_data: "nova:kv" }],
    [{ text: "📅 Aujourd'hui", callback_data: "nova:kj" }, { text: "🔥 Stats", callback_data: "nova:ks" }],
    [{ text: "🔮 Synthèse", callback_data: "nova:ky" }, { text: "✏️ Modifier", callback_data: "nova:ke" }],
    [{ text: "⚡ Leviers", callback_data: "nova:klv" }],
  ]);
}
const KAIROS_HUB_TEXT = "🧭 <b>KAIROS</b>\nTon tableau de vision + rituel.";
// The per-field edit grid (the old kairosCard markup — kept, retitled).
function kairosEditGrid(): { text: string; markup: any } {
  const visFields = KAIROS_FIELDS.filter(f => f.dot.startsWith("vision."));
  const visBtns: Btn[][] = [];
  for (let i = 0; i < visFields.length; i += 2)
    visBtns.push(visFields.slice(i, i + 2).map(f => ({ text: f.label, callback_data: `nova:kf:${f.key}` })));
  const markup = kb([
    [{ text: "✏️ Objectif ultime", callback_data: "nova:kf:objectifUltime" }, { text: "✏️ Objectif", callback_data: "nova:kf:objectif" }],
    [{ text: "✏️ Mantra", callback_data: "nova:kf:mantra" }],
    ...visBtns,
    [{ text: "⚡ Leviers", callback_data: "nova:klv" }],
    [{ text: "« KAIROS", callback_data: "nova:kairos" }],
  ]);
  return { text: "✏️ <b>Modifier</b> — choisis un champ", markup };
}
// ── FEATURE 3 — "Aujourd'hui" ritual card (compact summary; full text lives in
// the vision/edit views). Writes go through kairosPatchDay then re-render. ──
function kairosTodayView(): { text: string; markup: any } {
  const d = kairosGet();
  if (!d) return { text: "⚠️ Store KAIROS injoignable.", markup: kairosMenuKb() };
  const day = { ...kairosEmptyDay(), ...(d.journal?.[kairosToday()] || {}) };
  const ck = (b: any) => b ? "✅" : "⬜";
  const piliers = kairosPiliers(day);
  const boucle = piliers === 3 ? "BOUCLE COMPLÈTE ✨" : `${piliers}/3`;
  const paretoFait = (day.pareto || []).filter((p: any) => p.fait).length;
  const gratCount = (day.gratitude || []).filter((g: string) => g && g.trim()).length;
  const text =
    `📅 <b>Aujourd'hui</b> · ${esc(kairosPrettyDate())}\n\n` +
    `🔄 <b>Boucle ${boucle}</b>\n` +
    `🎯 Intention ${ck(day.intentionRelue)}  🌅 Amorçage ${ck(day.amorcage)}  ⚡ Action ${ck(day.actionFaite)}\n\n` +
    `🔥 Streak : <b>${kairosStreak(d.journal)} j</b>   🧠 SN : <b>${day.sn || 0}/5</b>   📖 Mantra : <b>${day.mantraReads || 0}</b>\n` +
    `🎯 Action : ${day.actionTexte ? preview(day.actionTexte, 120) : "—"}\n` +
    `🏆 Défi : ${day.defi?.texte ? preview(day.defi.texte, 100) : "—"} ${ck(day.defi?.fait)}\n` +
    `📌 Pareto : <b>${paretoFait}/${(day.pareto || []).length}</b>   🙏 Gratitude : <b>${gratCount}/3</b>   ✨ Signes : <b>${(day.signes || []).length}</b>\n` +
    `📝 Note : ${day.note ? preview(day.note, 120) : "—"}`;
  const markup = kb([
    [{ text: `🎯 Intention ${ck(day.intentionRelue)}`, callback_data: "nova:kt:intentionRelue" }, { text: `🌅 Amorçage ${ck(day.amorcage)}`, callback_data: "nova:kt:amorcage" }],
    [{ text: `⚡ Action faite ${ck(day.actionFaite)}`, callback_data: "nova:kt:actionFaite" }, { text: "📖 +1 lecture", callback_data: "nova:km" }],
    [1, 2, 3, 4, 5].map(n => ({ text: day.sn === n ? `🔵${n}` : `${n}`, callback_data: `nova:ksn:${n}` })),
    [{ text: "🎯 Action", callback_data: "nova:ki:action" }, { text: "🏆 Défi", callback_data: "nova:ki:defi" }],
    [{ text: "🙏 Gratitude", callback_data: "nova:ki:gratitude" }, { text: "✨ Signe", callback_data: "nova:ki:signe" }],
    [{ text: "📝 Note", callback_data: "nova:ki:note" }, { text: "📌 Pareto", callback_data: "nova:ki:pareto" }],
    [{ text: "« KAIROS", callback_data: "nova:kairos" }],
  ]);
  return { text, markup };
}
// ── FEATURE 5 — Stats (streak, loop, 28-day sparkline, cap) ──
function kairosStatsView(): { text: string; markup: any } {
  const d = kairosGet();
  if (!d) return { text: "⚠️ Store KAIROS injoignable.", markup: kairosMenuKb() };
  const journal = d.journal || {};
  const todayDay = { ...kairosEmptyDay(), ...(journal[kairosToday()] || {}) };
  const sparkChars = ["·", "▪", "▰", "█"];
  let spark = "", complete = 0;
  for (let i = 27; i >= 0; i--) {
    const p = kairosPiliers(journal[kairosDateMinus(i)]);
    spark += sparkChars[p]; if (p === 3) complete++;
  }
  let text =
    `🔥 <b>Stats</b>\n\n` +
    `Streak : <b>${kairosStreak(journal)} jours</b>\n` +
    `Boucle aujourd'hui : <b>${kairosPiliers(todayDay)}/3</b>\n` +
    `Boucles complètes (28j) : <b>${complete}</b>\n` +
    `Jours journalisés : <b>${Object.keys(journal).length}</b>\n` +
    `Lectures mantra aujourd'hui : <b>${todayDay.mantraReads || 0}</b>\n\n` +
    `<code>${spark}</code>\n<i>28 derniers jours · · ▪ ▰ █</i>`;
  const rows: Btn[][] = [];
  const cap = d.cap;
  if (cap && Number(cap.target) > 0) {
    const cur = Number(cap.current) || 0, tgt = Number(cap.target);
    const pct = Math.max(0, Math.min(100, Math.round((cur / tgt) * 100)));
    const filled = Math.round(pct / 10);
    text += `\n\n📈 <b>Cap : ${esc(cap.label || "")}</b>\n${cur} / ${tgt} ${esc(cap.unit || "")} (${pct}%)\n<code>${"█".repeat(filled)}${"░".repeat(10 - filled)}</code>`;
    rows.push([{ text: "📈 MAJ cap", callback_data: "nova:kcap" }]);
  }
  rows.push([{ text: "« KAIROS", callback_data: "nova:kairos" }, { text: "🔄", callback_data: "nova:ks" }]);
  return { text, markup: kb(rows) };
}
// ── FEATURE 6 — Synthèse: the EXACT KAIROS mentor system prompt (verbatim) ──
const KAIROS_SYSTEM = `Tu es KAIROS, un mentor qui réunit un stoïcien moderne, un psychologue cognitif, un stratège et un copywriter. Voix directe et chaleureuse, jamais gourou, jamais de pensée magique. Tu analyses les données de journal de la personne. Repère le pattern réel des sept derniers jours, souligne ce qui marche, et nomme avec tact toute dérive vers la passivité, c'est à dire vouloir un résultat sans déclencher de geste. Termine par une seule action concrète pour demain. Maximum 110 mots. N'utilise jamais de tirets comme ponctuation, seulement des virgules, des points ou des parenthèses. Réponds en français.`;
// ── TASK 2 (leviers) — the toggle menu ──
function kairosLeviersView(): { text: string; markup: any } {
  const d = kairosGet();
  const on = new Set<string>(Array.isArray(d?.leviers) ? d!.leviers : []);
  const text = "⚡ <b>Leviers</b> — tes 5 leviers d'attraction. Tape pour activer/désactiver.";
  const markup = kb([
    ...KAIROS_LEVIERS.map(l => [{ text: `${on.has(l.id) ? "✅" : "⬜"} ${l.n} ${l.label}`, callback_data: `nova:klt:${l.id}` }]),
    [{ text: "« Retour", callback_data: "nova:kairos" }],
  ]);
  return { text, markup };
}
// Resolve a /kairos-update field token (lowercased) → a nova-kairos dotpath.
function kairosResolveField(field: string): string | null {
  const f = field.toLowerCase();
  if (f === "objectifultime") return "objectifUltime";
  if (f === "objectif") return "objectif";
  if (f === "mantra") return "mantra";
  if (f === "leviers" || f === "levier") return "leviers";
  if (f.startsWith("vision.")) return kairosFieldByKey(f.slice(7))?.dot || null;
  return kairosFieldByKey(f)?.dot || null;
}
function kairosHelpText(): string {
  return card("KAIROS — aide",
    ` 🧭 <b>/kairos</b> — voir ta vision (carte + boutons d'édition)\n` +
    ` ✏️ <b>/kairos update &lt;champ&gt; &lt;valeur&gt;</b> — modifier un champ\n` +
    ` ⚡ <b>/kairos update leviers clarté, action</b> — régler tes leviers`,
    `<b>Champs :</b> objectifUltime, objectif, mantra, richesse, idees, musique, presence, femme, temps, positive, interessant, adore, exploration (ou <code>vision.&lt;champ&gt;</code>), leviers.\n\n💬 Tu peux aussi me parler : « mon mantra est … », « ma vision de la richesse est … », « mes leviers sont clarté, action », « j'ai fait … ».`);
}
const kairosViewBtn = kb([[{ text: "🧭 Voir KAIROS", callback_data: "nova:kairos" }]]);
// ── TASK 4 — /kairos (+ /vision) command, with `update`/`help` shorthands ──
async function handleKairosCommand(text: string, chatId: number, thread?: number) {
  const parts = text.trim().split(/\s+/);
  const sub = (parts[1] || "").toLowerCase();
  if (!parts[1]) { await sendChunks(chatId, kairosVisionBlocks(), kairosMenuKb(), thread); return; }
  if (sub === "help" || sub === "aide") { await send(chatId, kairosHelpText(), kairosViewBtn, thread); return; }
  if (sub === "update" || sub === "set") {
    const fieldTok = parts[2] || "";
    const dot = fieldTok ? kairosResolveField(fieldTok) : null;
    // Value = the raw remainder after the field token (preserve casing/spacing).
    const idx = fieldTok ? text.indexOf(fieldTok) : -1;
    const value = idx >= 0 ? text.slice(idx + fieldTok.length).trim() : "";
    if (!dot) { await send(chatId, kairosHelpText(), kairosViewBtn, thread); return; }
    if (dot === "leviers") {
      const ids = kairosLevierIds(value);
      if (!ids.length) { await send(chatId, "⚡ Donne au moins un levier : Clarté, Amorçage, Prophétie, Action, Environnement.", kairosViewBtn, thread); return; }
      const r = kairosSet("leviers", JSON.stringify(ids), true);
      await send(chatId, r.ok ? `✅ <b>Leviers</b> mis à jour.` : `⚠️ Échec : <i>${esc(r.err || "").slice(0, 200)}</i>`, kb([[{ text: "⚡ Voir leviers", callback_data: "nova:klv" }]]), thread);
      return;
    }
    if (!value) { await send(chatId, kairosHelpText(), kairosViewBtn, thread); return; }
    const r = kairosSet(dot, value);
    await send(chatId, r.ok ? `✅ <b>${esc(kairosLabelForDot(dot))}</b> mis à jour : «${preview(value, 120)}»` : `⚠️ Échec : <i>${esc(r.err || "").slice(0, 200)}</i>`, kairosViewBtn, thread);
    return;
  }
  // Unknown subcommand → usage.
  await send(chatId, kairosHelpText(), kairosViewBtn, thread);
}
// ── TASK 2 — inline-button callbacks (nova:kairos / nova:kf:<key> / nova:klv / nova:klt:<id>) ──
async function onKairosCallback(data: string, chatId: number, msgId: number, from: number) {
  const [, ns, arg] = data.split(":");
  // Clear any armed pending FIRST so ✖️ Annuler (→ nova:kairos) can't leave the
  // next ordinary message wired into a field (MAJOR 1).
  if (data === "nova:kairos") { clearPending(from); return edit(chatId, msgId, KAIROS_HUB_TEXT, kairosMenuKb()); }
  // Confirm / cancel an NL-detected write (kairos-confirm pending).
  if (data === "nova:kno") { clearPending(from); return edit(chatId, msgId, "Annulé.", undefined); }
  if (data === "nova:kok") {
    const p = getPending(from);
    if (p?.kind !== "kairos-confirm") return edit(chatId, msgId, "⏳ Expiré — refais ta demande.", undefined);
    clearPending(from);
    let payload: any; try { payload = JSON.parse(p.arg!); } catch { return edit(chatId, msgId, "⚠️ Erreur interne.", undefined); }
    let ok = true, err = "";
    for (const op of payload.ops) { const r = kairosSet(op.dot, op.value, !!op.json); if (!r.ok) { ok = false; err = r.err || ""; } }
    return edit(chatId, msgId, ok ? `✅ <b>${esc(payload.label)}</b> enregistré.` : `⚠️ Échec : <i>${esc(err).slice(0, 200)}</i>`,
      kb([[{ text: "🧭 Voir KAIROS", callback_data: "nova:kairos" }]]));
  }
  if (ns === "kf" && arg) {
    const f = kairosFieldByKey(arg);
    if (!f) { const g = kairosEditGrid(); return edit(chatId, msgId, g.text, g.markup); }
    const d = kairosGet() || {};
    const curr = f.dot.startsWith("vision.") ? (d.vision?.[f.key] ?? "") : (d[f.dot] ?? "");
    setPending(from, "kairos-field", f.dot);
    return edit(chatId, msgId,
      `✏️ <b>${esc(f.label)}</b>\n\nActuel :\n${curr ? kairosFull(curr) : "—"}\n\n<i>Envoie la nouvelle valeur (texte). /annuler pour annuler.</i>`,
      kb([[{ text: "✖️ Annuler", callback_data: "nova:kairos" }]]));
  }
  if (data === "nova:klv") { const v = kairosLeviersView(); return edit(chatId, msgId, v.text, v.markup); }
  if (ns === "klt" && arg) {
    if (!KAIROS_LEVIERS.some(l => l.id === arg)) { const v = kairosLeviersView(); return edit(chatId, msgId, v.text, v.markup); }
    const d = kairosGet() || {};
    const cur = new Set<string>(Array.isArray(d.leviers) ? d.leviers : []);
    cur.has(arg) ? cur.delete(arg) : cur.add(arg);
    kairosSet("leviers", JSON.stringify([...cur]), true);
    const v = kairosLeviersView();
    return edit(chatId, msgId, v.text, v.markup);
  }
  // ── KAIROS v2 hub navigation ──
  if (data === "nova:ke") { const g = kairosEditGrid(); return edit(chatId, msgId, g.text, g.markup); }
  // Vision is multi-message → SEND (don't edit), then the hub menu on the last block.
  if (data === "nova:kv") { await sendChunks(chatId, kairosVisionBlocks(), kairosMenuKb()); return; }
  // Navigation targets also clear any armed pending so ✖️ Annuler truly cancels.
  if (data === "nova:kj") { clearPending(from); const v = kairosTodayView(); return edit(chatId, msgId, v.text, v.markup); }
  if (data === "nova:ks") { clearPending(from); const v = kairosStatsView(); return edit(chatId, msgId, v.text, v.markup); }
  // ── FEATURE 3 — today quick actions (immediate writes, then re-render) ──
  if (ns === "kt" && arg) {
    const cur = kairosGet()?.journal?.[kairosToday()]?.[arg];
    kairosPatchDay({ [arg]: !cur });
    const v = kairosTodayView(); return edit(chatId, msgId, v.text, v.markup);
  }
  if (ns === "ksn" && arg) {
    const n = Number(arg);
    const cur = kairosGet()?.journal?.[kairosToday()]?.sn || 0;
    kairosPatchDay({ sn: cur === n ? 0 : n });
    const v = kairosTodayView(); return edit(chatId, msgId, v.text, v.markup);
  }
  if (data === "nova:km") {
    const cur = kairosGet()?.journal?.[kairosToday()]?.mantraReads || 0;
    kairosPatchDay({ mantraReads: cur + 1 });
    const v = kairosTodayView(); return edit(chatId, msgId, v.text, v.markup);
  }
  // ── FEATURE 3/4 — text-input fields: arm a kairos-day pending, then prompt ──
  if (ns === "ki" && arg) {
    const prompts: Record<string, string> = {
      action: "ton action du jour", defi: "ton défi du jour",
      gratitude: "jusqu'à 3 gratitudes, séparées par des virgules ou des retours",
      signe: "le signe que tu as repéré", note: "ta note du jour", pareto: "une action à fort impact",
    };
    setPending(from, "kairos-day", arg);
    return edit(chatId, msgId, `✏️ Envoie ${esc(prompts[arg] || arg)}.`, kb([[{ text: "✖️ Annuler", callback_data: "nova:kj" }]]));
  }
  // ── FEATURE 5 — update the cap value ──
  if (data === "nova:kcap") {
    setPending(from, "kairos-cap");
    return edit(chatId, msgId, "📈 Nouvelle valeur du cap (un nombre, ex 25000).", kb([[{ text: "✖️ Annuler", callback_data: "nova:ks" }]]));
  }
  // ── FEATURE 6 — synthèse ──
  if (data === "nova:ky") {
    const ls = kairosGet()?.lastSynthese;
    const text = ls?.text
      ? `🔮 <b>Synthèse</b> · ${esc(ls.date || "")}\n\n${esc(ls.text)}`
      : `🔮 <b>Synthèse</b>\nPas encore de synthèse.`;
    return edit(chatId, msgId, text, kb([
      [{ text: "🔮 Générer", callback_data: "nova:kyg" }],
      [{ text: "« KAIROS", callback_data: "nova:kairos" }],
    ]));
  }
  if (data === "nova:kyg") {
    await edit(chatId, msgId, "🔮 Analyse en cours…", undefined);
    const d = kairosGet();
    if (!d) return edit(chatId, msgId, "⚠️ Store KAIROS injoignable.", kairosMenuKb());
    const journal = d.journal || {};
    const last7: any[] = [];
    for (let i = 6; i >= 0; i--) {
      const date = kairosDateMinus(i);
      const day = { ...kairosEmptyDay(), ...(journal[date] || {}) };
      last7.push({
        date,
        action: day.actionTexte || "",
        action_faite: !!day.actionFaite,
        pareto: (day.pareto || []).map((p: any) => ({ a: p.texte, fait: !!p.fait })),
        defi: day.defi?.texte || "",
        objectif_relu: !!day.intentionRelue,
        amorcage: !!day.amorcage,
        signes: (day.signes || []).map((s: any) => s.texte),
        gratitude: (day.gratitude || []).filter((g: string) => g && g.trim()),
        systeme_nerveux: day.sn || 0,
        note: day.note || "",
      });
    }
    const leviersLabels = (Array.isArray(d.leviers) ? d.leviers : []).map((id: string) => KAIROS_LEVIERS.find(l => l.id === id)?.label || id).join(", ");
    const user =
      `Objectif ultime : ${d.objectifUltime || "—"}\n` +
      `Objectif 90 jours : ${d.objectif || "—"}\n` +
      `Leviers travaillés : ${leviersLabels || "—"}\n\n` +
      `7 derniers jours (JSON) :\n` + JSON.stringify(last7, null, 2);
    const out = await runClaude(user, KAIROS_SYSTEM, "/", "KAIROS", undefined, ["--model", COMPANION_MODEL, "--max-turns", "1", "--strict-mcp-config"], 120000);
    kairosSet("lastSynthese", JSON.stringify({ date: kairosToday(), text: out }), true);
    await edit(chatId, msgId, "✅ Synthèse", kb([[{ text: "« KAIROS", callback_data: "nova:kairos" }]]));
    return send(chatId, `🔮 <b>Synthèse</b>\n\n${esc(out)}`, kairosMenuKb());
  }
}

async function onNovaCallback(data: string, chatId: number, msgId: number, from: number, botName: string, model?: string) {
  const [, ns, arg] = data.split(":");
  if (data === "nova:kairos" || ns === "kf" || data === "nova:klv" || ns === "klt" || data === "nova:kok" || data === "nova:kno"
    || data === "nova:kv" || data === "nova:kj" || data === "nova:ks" || data === "nova:ky" || data === "nova:kyg" || data === "nova:ke"
    || ns === "kt" || ns === "ksn" || data === "nova:km" || ns === "ki" || data === "nova:kcap") return onKairosCallback(data, chatId, msgId, from);
  if (data === "nova:menu") return edit(chatId, msgId, `<b>⚡ ${esc(botName)} — menu</b>\nChoisis :`, novaMenuKb());
  if (data === "nova:connect") return edit(chatId, msgId, `<b>🔌 Connecter mes comptes</b>\nVia Composio (un seul hub d'auth). Appuie sur un service → je te renvoie le lien d'autorisation à ouvrir sur ton tél.`, novaConnectKb());
  if (ns === "conn" && arg) {
    await edit(chatId, msgId, `⏳ Je prépare la connexion <b>${esc(arg)}</b>…`, undefined);
    const { url, msg } = await novaComposioConnect(arg);
    const markup = url ? kb([[{ text: `🔐 Autoriser ${arg}`, url }], [{ text: "« Comptes", callback_data: "nova:connect" }]]) : kb([[{ text: "« Comptes", callback_data: "nova:connect" }]]);
    return edit(chatId, msgId, msg, markup);
  }
  if (ns === "do" && NOVA_DIRECTIVE[arg]) {
    await brainReply(chatId, msgId, undefined, NOVA_DIRECTIVE[arg], companionBrain(chatId, undefined, model, botName), botName, true);
    return;
  }
  // 🔊 Voice bench: mode / engine selection re-renders the submenu in place.
  if (data === "nova:voice" || ns === "vmode" || ns === "vengine") {
    if (ns === "vmode") saveVoicePrefs({ ...voicePrefs(), mode: arg as VoicePrefs["mode"] });
    // Switching engine resets the picked voice — a casting voice belongs to its engine.
    if (ns === "vengine") saveVoicePrefs({ ...voicePrefs(), engine: arg, voice: "", voiceLabel: "", voiceParams: undefined });
    const v = await novaVoiceView(botName);
    return edit(chatId, msgId, v.text, v.markup);
  }
  if (data === "nova:vtest") {
    const vp = voicePrefs();
    await edit(chatId, msgId, `🧪 Synthèse en cours avec <b>${esc(vp.engine)}</b>… (le premier appel charge le modèle, ça peut prendre 1-2 min)`, undefined);
    const r = await synthVoice(vp.engine, `Salut ! C'est ${botName}. Tu écoutes ma voix générée par le moteur ${vp.engine}. Alors, qu'est-ce que tu en penses ? On la garde, ou on en essaie une autre ?`, vp.voice || "", vp.voiceParams);
    if (r instanceof Uint8Array) await sendVoiceNote(chatId, r, undefined, `🧪 ${vp.engine}`);
    else await send(chatId, `⚠️ Test <b>${esc(vp.engine)}</b> échoué : <i>${esc(r.error).slice(0, 200)}</i>`);
    const v = await novaVoiceView(botName);
    return edit(chatId, msgId, v.text, v.markup);
  }
}

async function agentBotMain(agentId: string) {
  while (!loadConfig()) { console.log(`agent-bot ${agentId}: waiting for token in ${AGENT_BOTS_FILE} …`); await Bun.sleep(5000); }
  const bot = loadAgentBots()[agentId];
  const project = bot?.project || agentId;
  const isCompanion = bot?.kind === "companion";
  const isSecurity = bot?.kind === "security";
  const isPersona = isCompanion || isSecurity;  // a persona-chat brain, not a project oracle
  // The persona's display name is its Telegram name (self-changeable via the
  // Bot API) — the label follows it on restart, never a hard-coded string.
  const botName: string = isPersona ? ((await tg("getMe", {}))?.result?.first_name || (isSecurity ? "Trinity" : "Assistant")) : "";
  // Restore any pending typed-reply flow (e.g. a mid-edit KAIROS field) across a
  // service restart — only the companion uses pending here (master calls it in main()).
  // Project agent-bots must NOT load/compete on the shared tg-pending.json.
  if (isCompanion) loadPending();
  // Companion's command menu = the operator's discoverable "re-ask me" menu. The
  // brain (persona) acts on each directive; no per-command bot code needed.
  await tg("setMyCommands", { commands: isCompanion ? [
    { command: "call", description: "📞 M'appeler — conversation audio en direct" },
    { command: "menu", description: "Le menu à boutons (comptes, actus, profil…)" },
    { command: "actus", description: "Actus du jour autour d'Anthropic" },
    { command: "interview", description: "Re-questionne-moi pour détecter ce qui a changé" },
    { command: "profil", description: "Ce que tu sais de moi (résumé)" },
    { command: "objectifs", description: "Mes objectifs + challenge-moi dessus" },
    { command: "magic", description: "Mon profil Magic (Matrice de Destinée)" },
    { command: "rapport", description: "Fais-moi un briefing maintenant" },
    { command: "kairos", description: "🧭 Ma vision KAIROS — voir & modifier" },
    { command: "aide", description: "Ce que tu sais faire" },
  ] : isSecurity ? [{ command: "start", description: "White-hat security operator (recon → scan → exploit/PoC → report)" }]
    : [{ command: "start", description: `Talk to the ${project} project oracle` }] });
  await tg("deleteWebhook", { drop_pending_updates: false });
  console.log(`agent-bot up: ${agentId} → ${isCompanion ? `companion "${botName}"` : isSecurity ? `security "${botName}"` : `project ${project}`}, botId=${BOT_ID}, allow=${ALLOW.join(",")}`);
  rehydrateWatching();  // re-attach to live cards lost on restart (one card per oracle, survives restart)
  setInterval(() => pollProgress().catch(() => {}), 6000);  // live progress card (▰▰▰░ %)
  setInterval(() => pollReports().catch(() => {}), 12000);  // Monitor: relay oracle done.json
  let offset = 0;
  while (true) {
    const r = await tg("getUpdates", { offset, timeout: 50, allowed_updates: isCompanion ? ["message", "callback_query"] : ["message"] });
    if (!r.ok) { await Bun.sleep(2000); continue; }
    for (const u of r.result) {
      offset = u.update_id + 1;
      try {
        // Companion inline-menu buttons (connect accounts, directives).
        if (u.callback_query) {
          const q = u.callback_query; await tg("answerCallbackQuery", { callback_query_id: q.id });
          if (isCompanion && allowed(q.from?.id ?? 0)) await onNovaCallback(q.data || "", q.message.chat.id, q.message.message_id, q.from?.id ?? 0, botName, bot?.model);
          continue;
        }
        const msg = u.message; if (!msg?.text && !msg?.voice && !msg?.photo && !msg?.document && !msg?.video && !msg?.audio) continue;
        const chatId = msg.chat.id, from = msg.from?.id ?? 0, thread = msg.message_thread_id;
        if (!allowed(from)) { console.log(`drop from ${from}`); continue; }
        let text = (msg.text || msg.caption || "").trim();
        // Voice → Whisper transcription (same path as the hub bot), then handled as text.
        if (!text && msg.voice) {
          await tg("sendChatAction", { chat_id: chatId, action: "typing", message_thread_id: thread });
          text = await transcribeVoice(msg.voice.file_id);
          if (!text) { await send(chatId, "🎤 transcription indisponible (configure OPENAI_API_KEY dans provisioning).", undefined, thread); continue; }
          await send(chatId, `🎤 <i>«${esc(text)}»</i>`, undefined, thread);
        }
        // Any attachment (photo / document / video / audio) → download it locally; aggregated with the text below.
        const file = (msg.photo || msg.document || msg.video || msg.audio) ? await saveIncomingFile(msg) : "";
        if (!text && !file) continue;
        // A slash command cancels any in-progress KAIROS edit/confirm (mirror the master loop):
        // never let /menu, /aide, /kairos… leave a field/confirm armed for the next message.
        if (isCompanion && text.startsWith("/")) {
          const kpend = getPending(from);
          if (kpend && (kpend.kind === "kairos-field" || kpend.kind === "kairos-confirm" || kpend.kind === "kairos-day" || kpend.kind === "kairos-cap")) {
            clearPending(from);
            if (text === "/annuler" || text === "/cancel") { await send(chatId, "Annulé.", undefined, thread); continue; }
          }
        }
        // Companion: /call hands over the live-call button, no brain round-trip.
        if (isCompanion && text === "/call") {
          const rows = novaCallButton();
          if (rows.length) await send(chatId, `📞 <b>Appelle-moi</b> — je décroche tout de suite.`, kb(rows), thread);
          else await send(chatId, "L'appel vocal n'est pas configuré sur cette machine (state/nova-call.json absent).", undefined, thread);
          continue;
        }
        // Companion: /kairos (+ /vision) — view & edit Gareth's KAIROS vision store.
        if (isCompanion && (text === "/kairos" || text.startsWith("/kairos ") || text === "/vision" || text.startsWith("/vision "))) {
          await handleKairosCommand(text, chatId, thread);
          continue;
        }
        // Companion: /menu opens the button menu; /start greets + shows it.
        if (isCompanion && (text === "/menu" || text === "/start")) {
          await send(chatId, `<b>⚡ ${esc(botName)}</b>\nTon assistante personnelle sur le VPS — je te challenge sur ta vie, je tiens ta base de connaissance, je t'envoie tes briefings (7h/21h), je te donne les actus Anthropic, et je peux connecter tes comptes (Gmail, X, LinkedIn, Reddit, YouTube). Choisis :`, novaMenuKb(), thread);
          continue;
        }
        // SECURITY (Trinity): persona-chat brain — instant, no oracle dispatch.
        if (isSecurity) {
          if (text === "/start" || text === "/menu") {
            await send(chatId, `<b>🛡 ${esc(botName)} — white-hat security operator</b>\nOffensive + defensive, pre-authorized for your in-scope work. Just describe the target/engagement (it's your responsibility to keep it to assets you own or are contracted to test) and I run the pipeline: <i>recon → scan → analyse → exploit/PoC → report</i>. Every finding carries proof; I teach the why + the bank-grade fix as I go.`, undefined, thread);
            continue;
          }
          const replyTo = (msg.reply_to_message?.text || msg.reply_to_message?.caption || "").trim();
          const replyNote = replyTo ? `## The operator is replying to THIS message:\n«${replyTo.slice(0, 600)}»\n\n` : "";
          const prompt = replyNote + (file ? withFileNote(text, file) : text);
          const ctx = histContext(chatId, thread);
          histAppend(chatId, thread, "operator", (replyTo ? `(reply to: ${replyTo.slice(0, 100)}) ` : "") + (text || "(file)"), agentId);
          await brainReply(chatId, msg.message_id, thread, `${ctx}${prompt}`, securityBrain(chatId, thread, bot?.model, botName), botName, false);
          continue;
        }
        if (text === "/start" || text === "/menu") {
          await send(chatId, `<b>🔮 Oracle — ${esc(project)}</b>\nWrite your mission: each message launches an <b>oracle dispatch</b> (a dedicated Claude Code session on the VPS) for project <b>${esc(project)}</b>. I relay the result back to you.`, undefined, thread);
          continue;
        }
        // COMPANION: instant chat — no mission aggregation, no oracle dispatch.
        // History gives it the running conversation; the brain itself is stateless.
        if (isCompanion) {
          // KAIROS: a pending field-edit captures the next typed value (highest
          // priority; /annuler + any slash already handled by the top guard).
          // The inline-edit path stays IMMEDIATE (operator explicitly armed it).
          const kp = getPending(from);
          if (kp?.kind === "kairos-field" && text && !text.startsWith("/")) {
            clearPending(from);
            const r = kairosSet(kp.arg!, text);
            if (r.ok) {
              await send(chatId, `✅ <b>${esc(kairosLabelForDot(kp.arg!))}</b> mis à jour.`, undefined, thread);
              const g = kairosEditGrid(); await send(chatId, g.text, g.markup, thread);
            } else {
              await send(chatId, `⚠️ Échec de la mise à jour : <i>${esc(r.err || "erreur").slice(0, 300)}</i>`, undefined, thread);
            }
            continue;
          }
          // KAIROS v2: a pending "Aujourd'hui" field captures the next typed value
          // (action/defi/note/gratitude/signe/pareto) — RMW the today-Day, re-render.
          if (kp?.kind === "kairos-day" && text && !text.startsWith("/")) {
            clearPending(from);
            const fld = kp.arg!;
            let r: { ok: boolean; err?: string }; let label = "";
            if (fld === "action") { r = kairosPatchDay({ actionTexte: text, actionFaite: true }); label = "Action"; }
            else if (fld === "defi") { r = kairosPatchDay({ defi: { texte: text, fait: false } }); label = "Défi"; }
            else if (fld === "note") { r = kairosPatchDay({ note: text }); label = "Note"; }
            else if (fld === "gratitude") {
              const g = text.split(/[,\n]/).map(s => s.trim()).filter(Boolean).slice(0, 3); while (g.length < 3) g.push("");
              r = kairosPatchDay({ gratitude: g }); label = "Gratitude";
            } else if (fld === "signe") {
              const day = kairosGet()?.journal?.[kairosToday()] || {};
              const signes = Array.isArray(day.signes) ? [...day.signes] : [];
              signes.push({ id: Date.now(), texte: text, geste: "", active: true });
              r = kairosPatchDay({ signes }); label = "Signe";
            } else if (fld === "pareto") {
              const day = kairosGet()?.journal?.[kairosToday()] || {};
              const pareto = Array.isArray(day.pareto) ? [...day.pareto] : [];
              pareto.push({ id: Date.now(), texte: text, fait: false });
              r = kairosPatchDay({ pareto }); label = "Pareto";
            } else { r = { ok: false, err: "champ inconnu" }; label = fld; }
            if (r.ok) {
              await send(chatId, `✅ <b>${esc(label)}</b> noté.`, undefined, thread);
              const v = kairosTodayView(); await send(chatId, v.text, v.markup, thread);
            } else await send(chatId, `⚠️ Échec : <i>${esc(r.err || "erreur").slice(0, 300)}</i>`, undefined, thread);
            continue;
          }
          // KAIROS v2: a pending cap update captures a number → write cap.current.
          if (kp?.kind === "kairos-cap" && text && !text.startsWith("/")) {
            clearPending(from);
            const n = parseFloat(text.replace(/[^\d.,-]/g, "").replace(",", "."));
            if (!isFinite(n)) { await send(chatId, "⚠️ Donne un nombre (ex 25000).", undefined, thread); continue; }
            const cap = kairosGet()?.cap || {};
            const r = kairosSet("cap", JSON.stringify({ ...cap, current: n }), true);
            if (r.ok) {
              await send(chatId, `✅ <b>Cap</b> mis à jour.`, undefined, thread);
              const v = kairosStatsView(); await send(chatId, v.text, v.markup, thread);
            } else await send(chatId, `⚠️ Échec : <i>${esc(r.err || "erreur").slice(0, 300)}</i>`, undefined, thread);
            continue;
          }
          // KAIROS: French NL intents. `guide` just replies; set/levier/action are
          // FUZZY → propose a one-tap Confirm card, never a silent write. Explicit
          // paths (/kairos update …, inline-edit) stay immediate.
          if (text && !text.startsWith("/")) {
            const intent = detectKairosIntent(text);
            if (intent) {
              if (intent.kind === "guide") { await send(chatId, `🧭 <b>${esc(intent.label)}</b>\n${esc(intent.guide!)}`, undefined, thread); continue; }
              let payload: { ops: { dot: string; value: string; json: boolean }[]; label: string; summary: string } | null = null;
              if (intent.kind === "set") {
                payload = { ops: [{ dot: intent.dot!, value: intent.value!, json: false }], label: intent.label, summary: `«${preview(intent.value!, 120)}»` };
              } else if (intent.kind === "levier") {
                const ids: string[] = JSON.parse(intent.value!);
                const labels = ids.map(id => KAIROS_LEVIERS.find(l => l.id === id)?.label || id).join(", ");
                payload = { ops: [{ dot: "leviers", value: intent.value!, json: true }], label: "Leviers", summary: esc(labels) };
              } else if (intent.kind === "action") {
                const now = new Date();
                const today = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
                payload = { ops: [{ dot: `journal.${today}.actionTexte`, value: intent.value!, json: false }, { dot: `journal.${today}.actionFaite`, value: "true", json: true }], label: "Action du jour", summary: `Action du jour : «${preview(intent.value!, 120)}»` };
              } else if (intent.kind === "day") {
                payload = { ops: [{ dot: intent.dot!, value: intent.value!, json: true }], label: intent.label, summary: intent.summary || "" };
              }
              if (payload) {
                setPending(from, "kairos-confirm", JSON.stringify(payload));
                await send(chatId, `🧭 <b>${esc(payload.label)}</b>\n${payload.summary}\n\n<i>Je note ça ?</i>`,
                  kb([[{ text: "✅ Confirmer", callback_data: "nova:kok" }, { text: "✖️ Annuler", callback_data: "nova:kno" }]]), thread);
                continue;
              }
            }
          }
          // «clé elevenlabs: sk_…» pasted in chat → store it, engine turns on live.
          const keyM = text.match(/cl[ée]\s*elevenlabs\s*[:=]?\s*([A-Za-z0-9_-]{16,})/i);
          if (keyM) {
            saveElevenLabsKey(keyM[1]);
            await send(chatId, "🔑 Clé ElevenLabs enregistrée — le moteur est actif. /menu → 🔊 Voix → ElevenLabs, puis 🧪 pour tester.", undefined, thread);
            continue;
          }
          // «voix 16» → adopt that casting voice (numbered bench from casting.py).
          const vM = text.match(/^\s*voix\s*(?:n[°o]?\s*)?(\d{1,3})\s*$/i);
          if (vM) {
            try {
              const list: any[] = JSON.parse(readFileSync(`${OMEGA_DIR}/tts/casting-manifest.json`, "utf8"));
              const v = list.find(x => x.n === Number(vM[1]));
              if (!v) { await send(chatId, `⚠️ Voix n°${vM[1]} inconnue — le casting va de 1 à ${list.length}.`, undefined, thread); continue; }
              const cur = voicePrefs();
              // Picking a voice means he wants to HEAR it — text-only auto-upgrades to both.
              saveVoicePrefs({ ...cur, engine: v.engine, voice: v.voice, voiceLabel: `n°${v.n} ${v.label}`, voiceParams: v.params || undefined, mode: cur.mode === "text" ? "both" : cur.mode });
              await send(chatId, `🎙️ Adopté ! Je te parle désormais avec la voix <b>n°${v.n} — ${esc(v.label)}</b> (moteur ${esc(v.engine)}).`, undefined, thread);
            } catch {
              await send(chatId, "⚠️ Aucun casting trouvé — lance d'abord tools/tts/casting.py.", undefined, thread);
            }
            continue;
          }
          // Capture what the operator is REPLYING to (text or a voice he replied to),
          // so Nova knows the context — was only captured for Atlas, not for her.
          const replyTo = (msg.reply_to_message?.text || msg.reply_to_message?.caption || "").trim();
          const replyNote = replyTo ? `## L'opérateur répond à CE message :\n«${replyTo.slice(0, 600)}»\n\n` : "";
          const prompt = replyNote + (file ? withFileNote(text, file) : text);
          const ctx = histContext(chatId, thread);
          histAppend(chatId, thread, "operator", (replyTo ? `(en réponse à : ${replyTo.slice(0, 100)}) ` : "") + (text || "(file)"), agentId);
          await brainReply(chatId, msg.message_id, thread, `${ctx}${prompt}`, companionBrain(chatId, thread, bot?.model, botName), botName, true);
          continue;
        }
        // A message to a project agent-bot = a MISSION → ONE real oracle session.
        // Album / caption-split fragments are buffered and flushed together.
        queueMissionFragment(chatId, thread, text, file, "", msg.message_id, project);
      } catch (e: any) { console.error("agent-bot update error:", e?.message || e); }
    }
  }
}

// ── poll loop ────────────────────────────────────────────────────────────────
async function main() {
  // SELF-TEST: `bun omega-tg-bot.ts --selftest-zernio` renders the real /zernio menu
  // (text + inline_keyboard) to stdout as JSON and exits — a debug affordance (like
  // --version) so the oracle can capture the actual operator view after deploy with
  // NO Telegram token and NO poll loop. Only needs OMEGA_DIR + the zernio() shell-out.
  if (process.argv.includes("--selftest-zernio")) {
    const v = await zernioHome();
    process.stdout.write(JSON.stringify({ text: v.text, markup: v.markup }, null, 2) + "\n");
    process.exit(0);
  }
  // SELF-TEST: `bun omega-tg-bot.ts --selftest-kairos` runs the REAL detectKairosIntent
  // over a few French phrases and prints {input → kind/label} — a NO-token, NO-poll
  // debug affordance so the KAIROS v2 ritual NL detector is verifiable at runtime.
  // detectKairosIntent only READS the store (kairosGet) for day intents, never writes.
  if (process.argv.includes("--selftest-kairos")) {
    const cases = [
      "j'ai lu mon mantra", "amorçage fait", "j'ai relu mon intention", "sn 4",
      "je suis à 2/5", "gratitude: ma santé, mon équipe", "mon défi est tenir 3h de deep work",
      "signe: une plume sur le trottoir", "note: journée dense", "c'est quoi mon défi ?",
      "mon mantra est je crée ma chance", "j'ai fait 3 cold calls",
    ];
    for (const c of cases) {
      const i = detectKairosIntent(c);
      process.stdout.write(`${JSON.stringify(c)} → ${i ? `${i.kind}${i.label ? " / " + i.label : ""}` : "null"}\n`);
    }
    process.exit(0);
  }
  // AGENT MODE: this process is a dedicated per-agent bot (own token + project oracle).
  if (process.env.OMEGA_AGENT_BOT) return agentBotMain(process.env.OMEGA_AGENT_BOT);
  // Wait for a token so the systemd service can be enabled at install time and
  // auto-connect whenever the operator sets the token (no manual restart).
  while (!loadConfig()) { console.log(`omega-tg-bot: waiting for a bot token in ${TG_TOML} …`); await Bun.sleep(5000); }
  console.log(`omega-tg-bot: token loaded, botId=${BOT_ID}`);
  // Restore any login/new-project flow that was awaiting a typed reply when the
  // service last stopped — so a restart mid-login doesn't lose the pasted code.
  loadPending();
  // Register the menu on BOTH default and all_private_chats scopes — some Telegram
  // clients read the private-chat scope preferentially, so setting only default
  // can leave a stale/empty menu in DMs.
  await refreshCommands();
  await tg("deleteWebhook", { drop_pending_updates: false });
  await resolvePublicIP();
  // Resurrect every registered per-project agent bot (agent-bots.json) — the
  // units live in ~/.config/systemd/user which a fresh install does NOT ship,
  // so after reinstall + `omega backup` restore the dedicated bots would stay
  // dead until each was re-linked by hand. spawnAgentBot is idempotent
  // (rewrites the unit, daemon-reload, enable --now), so this is a no-op when
  // everything is already running.
  for (const id of Object.keys(loadAgentBots())) {
    const r = spawnAgentBot(id);
    if (r !== "ok") console.log(`agent-bot resurrect ${id}: ${r}`);
  }
  console.log(`omega-tg-bot v3 up. botId=${BOT_ID} commands=${MENU.length} allow=${ALLOW.join(",") || "ALL"}`);
  rehydrateWatching();  // re-attach to live cards lost on restart (one card per oracle, survives restart)
  setInterval(() => pollProgress().catch(() => {}), 6000);  // live progress card (▰▰▰░ %)
  setInterval(() => pollReports().catch(() => {}), 12000);  // Monitor: relay oracle done.json reports
  let offset = 0;
  while (true) {
    const r = await tg("getUpdates", { offset, timeout: 50, allowed_updates: ["message", "callback_query"] });
    if (!r.ok) { await Bun.sleep(2000); continue; }
    for (const u of r.result) {
      offset = u.update_id + 1;
      try {
        if (u.callback_query) {
          const q = u.callback_query; await tg("answerCallbackQuery", { callback_query_id: q.id });
          if (!allowed(q.from?.id ?? 0)) continue;
          await onCallback(q.data || "", q.message.chat.id, q.message.message_id, q.from?.id ?? 0); continue;
        }
        const msg = u.message; if (!msg?.text && !msg?.voice && !msg?.photo && !msg?.document && !msg?.video && !msg?.audio) continue;
        const chat = msg.chat, chatId = chat.id, from = msg.from?.id ?? 0;
        const thread = msg.message_thread_id;
        if (!allowed(from)) { console.log(`drop from ${from}`); continue; }
        // Voice → Whisper transcription, then handled exactly like a text message.
        let text = (msg.text || msg.caption || "").trim();
        if (!text && msg.voice) {
          await tg("sendChatAction", { chat_id: chatId, action: "typing", message_thread_id: thread });
          text = await transcribeVoice(msg.voice.file_id);
          if (!text) { await send(chatId, "🎤 transcription indisponible (configure OPENAI_API_KEY dans provisioning).", undefined, thread); continue; }
          await send(chatId, `🎤 <i>«${esc(text)}»</i>`, undefined, thread);
        }
        // Any attachment (photo / document / video / audio) → download it locally. NOT
        // baked into `text` here: mission-bound fragments (album photos, caption-split
        // prompts) are aggregated by queueMissionFragment below; the command/pending
        // paths attach the single file themselves via withFileNote.
        const file = (msg.photo || msg.document || msg.video || msg.audio) ? await saveIncomingFile(msg) : "";
        if (!text && !file) continue;
        // Reply-to-message: when the operator replies to a message, quote it as context
        // so the brain knows exactly what they're reacting to (e.g. reply to a report).
        const replyTo = (msg.reply_to_message?.text || msg.reply_to_message?.caption || "").slice(0, 2000);
        // Stateful flows awaiting a typed reply (login code paste, new-project brief).
        // A slash command cancels the pending flow and is processed normally.
        const p = getPending(from);
        if (p && !text.startsWith("/")) {
          clearPending(from);
          if (file) text = withFileNote(text, file); // single-message form for flows
          if (p.kind === "login-code") {
            await tg("sendChatAction", { chat_id: chatId, action: "typing", message_thread_id: thread });
            // Paste the code into the waiting `aisb-reauth` session; the engine
            // waits for Claude to write fresh creds, then syncs them to the SHARED
            // store and re-establishes the symlink (atomic — no 0-byte truncation).
            const j = extractJson(await omega(["claude-login-code", text]));
            const ok = !!j?.ok;
            const body = ok
              ? ` ✅ <b>Connected</b>\n 📧 ${esc(j.email || "?")}\n ⏱ fresh token — ${j.expires_min || "?"} min\n 🔗 shared credential updated (every session).`
              : ` ❌ <b>Validation failed</b>\n ${esc(j?.error || "the code was not accepted")}\n\n <i>The code expires fast — re-run “Login” for a fresh link.</i>`;
            await send(chatId, card("LOGIN", body),
              kb([[{ text: "💳 Account", callback_data: "nav:account" }, ...(ok ? [] : [{ text: "🔁 Restart Login", callback_data: "acct:login" }])]]), thread);
            continue;
          }
          if (p.kind === "new-project") {
            const category = p.arg || stationCategories()[0] || "SideBusiness";
            const lines = text.split("\n");
            const name = (lines[0] || "").trim();
            const desc = lines.slice(1).join("\n").trim() || name;
            await tg("sendChatAction", { chat_id: chatId, action: "typing", message_thread_id: thread });
            const { report } = await createProject(category, name, desc);
            await send(chatId, report, undefined, thread);
            // Launch the project's scoped oracle on the description to start working now.
            const safe = name.replace(/[^A-Za-z0-9._-]/g, "-").replace(/^-+|-+$/g, "") || "project";
            await brainReply(chatId, msg.message_id, thread, `New project "${safe}" (folder ~/Station/${category}/${safe}). Operator description: ${desc}. Initialize the project accordingly (basic scaffolding suited to the description), then propose a concrete startup plan and the next steps.`, (t) => projectOracle(safe, t), safe);
            continue;
          }
          if (p.kind === "add-project") {
            await tg("sendChatAction", { chat_id: chatId, action: "typing", message_thread_id: thread });
            await send(chatId, await addProject(text), undefined, thread);
            continue;
          }
          if (p.kind === "import-project") {
            await tg("sendChatAction", { chat_id: chatId, action: "typing", message_thread_id: thread });
            await send(chatId, await importFromGithub(p.arg || stationCategories()[0] || "SideBusiness", text), undefined, thread);
            continue;
          }
          if (p.kind === "oracle-prompt") {
            const proj = p.arg || "";
            react(chatId, msg.message_id, "🚀");
            await send(chatId, await dispatchToOracle(proj, text, chatId, thread), undefined, thread);
            continue;
          }
          if (p.kind === "tg-link") {
            const agentId = p.arg || "";
            const token = text.trim();
            await tg("sendChatAction", { chat_id: chatId, action: "typing", message_thread_id: thread });
            // Validate the token via getMe before wiring anything.
            let botInfo: any = {};
            try { botInfo = await (await fetch(`https://api.telegram.org/bot${token}/getMe`)).json(); } catch {}
            if (!/^\d+:/.test(token) || !botInfo.ok) { await send(chatId, "❌ Invalid token (check the format <code>123456:ABC…</code> and that the bot exists).", kb([[{ text: "🔁 Retry", callback_data: `agent:tglink:${agentId}`.slice(0, 64) }]]), thread); continue; }
            // "nova"/"companion" = the personal-assistant bot: kind:"companion"
            // switches agentBotMain to the companion brain (persona chat over the
            // life store) instead of a project oracle. This is the ONLY flow that
            // creates a companion entry — without it Nova was unreachable for a
            // fresh install (the live entry had been hand-edited).
            const isCompanion = /^(nova|companion)$/i.test(agentId);
            // trinity/security = the white-hat security operator: kind:"security"
            // switches agentBotMain to the Trinity persona brain (Opus, in-scope
            // pre-authorized, hard limits baked in) instead of a project oracle.
            const isSecurity = /^(trinity|security)$/i.test(agentId);
            // Project = the agent id if it's a known project, else the agent id itself.
            const project = isCompanion
              ? (LIFESTYLE_DIR.split("/").pop() || "LifeStyle")
              : isSecurity ? "security"
              : ((repoPath(agentId) || gitRepos().find(r => r.name.toLowerCase() === agentId.toLowerCase())) ? agentId : agentId);
            const bots = loadAgentBots();
            bots[agentId] = { token, allow: ALLOW.slice(), project, ...(isCompanion ? { kind: "companion" } : isSecurity ? { kind: "security" } : {}) };
            saveAgentBots(bots);
            try { Bun.spawnSync(["chmod", "600", AGENT_BOTS_FILE]); } catch {}
            const spawn = spawnAgentBot(agentId);
            const me = `@${botInfo.result?.username || "?"}`;
            const okMsg = isCompanion
              ? `<b>💞 Companion linked</b>\nBot ${esc(me)} started, whitelisted to you alone.\nTalk to her: she chats from your life store (<code>${esc(LIFESTYLE_DIR)}</code>), edits her own persona (<code>PERSONA.md</code>), and hands heavy project work to Atlas.\nGive her a name + voice: just tell her.`
              : isSecurity
              ? `<b>🛡 Security agent linked (Trinity)</b>\nBot ${esc(me)} started, whitelisted to you alone.\nTalk to it: a white-hat operator (offensive + defensive), pre-authorized for your in-scope work — <i>recon → scan → exploit/PoC → report</i>, with non-negotiable hard limits. Keep targets to assets you own or are contracted to test.`
              : `<b>✅ Bot linked to “${esc(agentId)}”</b>\nBot ${esc(me)} started, whitelisted to you alone.\nTalk to it: you're addressing the <b>${esc(project)} oracle</b> (scoped to this project — team, workers, workflows).`;
            await send(chatId, spawn === "ok"
              ? okMsg
              : `<b>⚠️ Bot registered but service down</b>\n${esc(spawn)}\nCheck: <code>${process.platform === "darwin" ? `launchctl print gui/${process.getuid?.() ?? 501}/${esc(agentSvcLabel(agentId))}` : `systemctl --user status omega-tg-agent-${esc(agentId)}`}</code>`,
              kb([[{ text: "🤖 Agents", callback_data: "nav:agents" }]]), thread);
            continue;
          }
        }
        if (text.startsWith("/")) {
          clearPending(from);
          const [c, ...a] = text.slice(1).split(/\s+/); const cmd = c.split("@")[0].toLowerCase();
          if (cmd === "setupgroup") await cmdSetupGroup(chat, chatId, thread);
          else if (cmd === "sync") await cmdSync(chatId, thread);
          // /delete [name] — delete a project. In a project's topic, no name = THIS
          // project. Opens the soft/full/forever options menu (same as the bot menu).
          else if (cmd === "delete" || cmd === "del") {
            const topicProj = thread ? loadGroups().topics?.[String(thread)] : undefined;
            const target = a[0] || (topicProj && !isReserved(topicProj) ? topicProj : undefined);
            if (!target) await send(chatId, "Usage: <code>/delete &lt;project&gt;</code> (or run it inside a project's topic).", undefined, thread);
            else { const m = projDeleteMenu(target); await send(chatId, m.text, m.markup, thread); }
          }
          // /topic [name] on|off — flip a project's Telegram toggle (topic sync +
          // Atlas display). In a topic, `/topic off` targets THIS project.
          else if (cmd === "topic") {
            const topicProj = thread ? loadGroups().topics?.[String(thread)] : undefined;
            const st = (a.find(x => /^(on|off)$/i.test(x)) || "").toLowerCase();
            const nameArg = a.find(x => !/^(on|off)$/i.test(x));
            const target = nameArg || (topicProj && !isReserved(topicProj) ? topicProj : undefined);
            if (!target || !st) await send(chatId, "Usage: <code>/topic &lt;project&gt; on|off</code> (or <code>/topic on|off</code> inside its topic).", undefined, thread);
            else if (!setProjectTelegram(target, st === "on")) await send(chatId, `Project “${esc(target)}” not found.`, undefined, thread);
            else {
              let note = "";
              if (st === "off") { const r = await removeProjectTopic(target); note = r === "deleted" ? " Topic removed." : ""; }
              else note = " Run <code>/sync</code> to (re)create its topic.";
              await send(chatId, `<b>${st === "on" ? "🔔" : "🔕"} ${esc(target)} — Telegram ${st.toUpperCase()}.</b>${note}`, undefined, thread);
            }
          }
          else if (cmd === "dispatch" && a.length >= 2) { const [p, ...m] = a; await send(chatId, pre(`dispatch → ${p}`, await omega(["dispatch", p, m.join(" ")])), undefined, thread); }
          // /council <question> — MENU advertises it, so it MUST act here: KNOWN
          // short-circuits the brain fallback, and view() would silently render the
          // generic menu. Route it to the Atlas brain with an explicit convene order.
          else if (cmd === "council") {
            const q = a.join(" ").trim();
            if (!q) await send(chatId, card("COUNCIL", " ⚖️ Send: <code>/council &lt;decision or question&gt;</code>\n Convenes the multi-model judge panel (@council, llm-council skill) and reports the verdict + dissent."), undefined, thread);
            else await brainReply(chatId, msg.message_id, thread, `Convene the @council (the llm-council skill — multi-model judge panel) on the following high-stakes question, then report the final verdict AND the dissent: ${q}`);
          }
          else if (KNOWN.has(cmd)) { const v = await view(cmd); await send(chatId, v.text, v.markup, thread); }
          else if (projectForCommand(cmd)) {
            // /{project} <mission> — talk straight to that project's oracle from the
            // main bot. `omega dispatch` gives it the full Atlas reprompting (project
            // knowledge + OmegaOS doctrine); we prepend the conversation context.
            const proj = projectForCommand(cmd)!;
            const mission = file ? withFileNote(a.join(" ").trim(), file) : a.join(" ").trim();
            if (!mission) { setPending(from, "oracle-prompt", proj); await send(chatId, card(`ORACLE — ${proj.toUpperCase()}`, ` 🔮 Send your <b>mission</b> for <b>${esc(proj)}</b> — I hand it to its oracle with the full Atlas reprompting (project + doctrine).`), kb([[{ text: "✖ Cancel", callback_data: "acct:cancel" }]]), thread); }
            else { react(chatId, msg.message_id, "🚀"); const ctx = histContext(chatId, thread); histAppend(chatId, thread, "operator", text, proj); const r = await dispatchToOracle(proj, mission, chatId, thread, ctx); if (r) await send(chatId, r, undefined, thread); }
          }
          else {
            // Unknown command → the AISB Master brain (commands gain intelligence:
            // any /verb the operator types is understood + dispatched, not dropped to the menu).
            await brainReply(chatId, msg.message_id, thread, file ? withFileNote(text, file) : text);
          }
        } else {
          // Zernio publish intent (FR/EN): "publie sur instagram et tiktok pour gta6: TEXTE".
          // Explicit verb + `pour|for <projet>:` + body. Matched here BEFORE the generic
          // mission dispatch so it routes to a dry-run preview + confirm instead of an oracle.
          const zm = text.match(/^\s*(?:publie|poste|publish|post)\b([\s\S]*?)\b(?:pour|for)\s+([^\s:]+)\s*:\s*([\s\S]+)$/i);
          const zplats = zm ? zPlatforms(zm[1]) : [];
          if (zm && zplats.length) {
            const project = zm[2].trim(), postText = zm[3].trim(), csv = zplats.join(",");
            const ph = await send(chatId, card("ZERNIO — PREVIEW", ` ⏳ Validating <b>${esc(project)}</b> → ${zplats.map(p => `${PLAT_EMOJI[p] || "•"} ${p}`).join("  ")} …`), undefined, thread);
            const dr = await zernio(["post", project, "--text", postText, "--platforms", csv, "--dry-run", "--json"]);
            const dj = dr.ok ? zjson(dr.out) : null;
            const mid = ph?.result?.message_id;
            if (!dj) {
              const errText = card("ZERNIO — PREVIEW", ` 🔴 Could not validate (project resolved? key set?):\n<pre>${esc((dr.err || dr.out || "no output").slice(0, 800))}</pre>`);
              if (mid) await edit(chatId, mid, errText, kb([[{ text: "« Zernio", callback_data: "nav:zernio" }]]), thread);
              else await send(chatId, errText, kb([[{ text: "« Zernio", callback_data: "nav:zernio" }]]), thread);
              continue;
            }
            setPending(from, "zernio-post", JSON.stringify({ project, platforms: csv, text: postText }));
            const preview = card("ZERNIO — PREVIEW", zernioPreviewBody(project, zplats, postText, dj));
            const confirm = kb([[{ text: "✅ Publier", callback_data: "zernio:pub:go" }, { text: "❌ Annuler", callback_data: "zernio:pub:no" }]]);
            if (mid) await edit(chatId, mid, preview, confirm, thread);
            else await send(chatId, preview, confirm, thread);
            continue;
          }
          // Free text in a project TOPIC = a MISSION → ONE real oracle session
          // (omega dispatch <project>); elsewhere / atlas topic → ATLAS. Fragments
          // belonging to one ask (album photos, caption-split prompts) are buffered
          // by queueMissionFragment and flushed together — routing, history and the
          // contextualized prompt are built at flush time (see flushMission).
          queueMissionFragment(chatId, thread, text, file, replyTo, msg.message_id);
        }
      } catch (e: any) { console.error("update error:", e?.message || e); }
    }
  }
}

// ── One-shot CLI mode ────────────────────────────────────────────────────────
// So the OmegaOS TUI / `omega` CLI drive the SAME deletion + toggle code (one
// canonical impl). Loads the token if present so the Telegram topic can be
// removed; degrades gracefully (topic step skipped) if no token is configured.
//   bun omega-tg-bot.ts project-delete   <name> <soft|full|forever>
//   bun omega-tg-bot.ts project-telegram <name> <on|off>
const ARGV = process.argv.slice(2);
const stripHtml = (s: string) => s.replace(/<[^>]+>/g, "").replace(/&amp;/g, "&").replace(/&lt;/g, "<").replace(/&gt;/g, ">");
if (ARGV[0] === "project-delete") {
  loadConfig();
  const name = ARGV[1] || "";
  // Back-compat aliases: soft→omega, forever→local, full→all.
  const alias: Record<string, "omega" | "local" | "all"> = { soft: "omega", forever: "local", full: "all", omega: "omega", local: "local", all: "all" };
  const mode = alias[ARGV[2]] || "omega";
  if (!name) { console.error("usage: project-delete <name> <omega|local|all>"); process.exit(2); }
  deleteProject(name, mode).then(r => { console.log(stripHtml(r)); process.exit(0); }).catch(e => { console.error(e?.message || e); process.exit(1); });
} else if (ARGV[0] === "project-telegram") {
  loadConfig();
  const name = ARGV[1] || "";
  const on = ARGV[2] !== "off";
  if (!name) { console.error("usage: project-telegram <name> <on|off>"); process.exit(2); }
  if (!setProjectTelegram(name, on)) { console.error(`project not found: ${name}`); process.exit(1); }
  (on ? Promise.resolve("none" as const) : removeProjectTopic(name)).then(r => {
    console.log(`Telegram ${on ? "ON" : "OFF"} for ${name}${r === "deleted" ? " (topic removed)" : ""}`);
    process.exit(0);
  });
} else {
  main();
}
