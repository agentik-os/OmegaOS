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
import { readFileSync, writeFileSync, existsSync } from "node:fs";
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
// Image input: a photo (or image document) is downloaded to ${OMEGA_DIR}/state/tg-media/
// so the dispatched oracle / Atlas can open it with the Read tool. Returns the local
// path, or "" when the message carries no image / the download failed. Without this,
// a photo+caption message has no `.text` and was silently DROPPED by both poll loops
// (the operator's mission never reached any oracle).
async function saveIncomingImage(msg: any): Promise<string> {
  try {
    const photo = Array.isArray(msg?.photo) && msg.photo.length ? msg.photo[msg.photo.length - 1] : undefined; // last = largest size
    const doc = /^image\//.test(msg?.document?.mime_type || "") ? msg.document : undefined;
    const fileId = photo?.file_id || doc?.file_id; if (!fileId) return "";
    const gf = await tg("getFile", { file_id: fileId });
    const fp = gf?.result?.file_path; if (!fp) return "";
    const ext = (fp.match(/\.[A-Za-z0-9]+$/) || [".jpg"])[0];
    const dest = `${OMEGA_DIR}/state/tg-media/tg-${msg.chat?.id}-${msg.message_id}${ext}`;
    const data = await (await fetch(`https://api.telegram.org/file/bot${TOKEN}/${fp}`)).arrayBuffer();
    await Bun.write(dest, data); // creates parent dirs
    return dest;
  } catch (e: any) { console.error("saveIncomingImage:", e?.message || e); return ""; }
}
// Mission text for a message that carries an image: caption (or a default) + where
// the image lives on the VPS, so the receiving Claude session opens it with Read.
function withImageNote(text: string, img: string): string {
  return `${text || "Analyze the attached image and act on it."}\n\n## Attached image\nSaved on the VPS at: ${img}\nOpen it with the Read tool and analyze it as part of this mission.`;
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
type AgentBot = { token: string; allow: number[]; project: string };
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
  // Message gone (e.g. placeholder deleted): post a fresh message, keeping topic context.
  return send(chat, text, markup, thread);
}

// ── omega CLI ────────────────────────────────────────────────────────────────
async function omega(args: string[]): Promise<string> {
  try { const r = await $`${OMEGA} ${args}`.quiet().nothrow(); const o = (r.stdout.toString() + r.stderr.toString()).trim(); return o || `(no output, exit ${r.exitCode})`; }
  catch (e: any) { return `error: ${e?.message || e}`; }
}
const esc = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

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
  const stash = (html: string) => ` ${codes.push(html) - 1} `;
  let s = src.replace(/```[a-zA-Z0-9]*\n?([\s\S]*?)```/g, (_m, c) => stash(`<pre>${esc(String(c).replace(/\n$/, ""))}</pre>`));
  s = s.replace(/`([^`\n]+)`/g, (_m, c) => stash(`<code>${esc(String(c))}</code>`));
  s = esc(s);
  s = s.replace(/\[([^\]\n]+)\]\((https?:\/\/[^\s)]+)\)/g, '<a href="$2">$1</a>');     // links
  s = s.replace(/^[ \t]*#{1,6}[ \t]+(.+)$/gm, "<b>$1</b>");                              // headers → bold
  s = s.replace(/\*\*([^\n*]+)\*\*/g, "<b>$1</b>").replace(/__([^\n_]+)__/g, "<b>$1</b>"); // bold
  s = s.replace(/~~([^\n~]+)~~/g, "<s>$1</s>");                                          // strikethrough
  s = s.replace(/(^|[^*\w])\*([^\n*]+)\*(?!\w)/g, "$1<i>$2</i>");                        // italic *…*
  s = s.replace(/(^|[^_\w])_([^\n_]+)_(?!\w)/g, "$1<i>$2</i>");                          // italic _…_
  s = s.replace(/^[ \t]*[-*+][ \t]+/gm, "• ");                                          // bullets
  return s.replace(/ (\d+) /g, (_m, i) => (codes[+i] !== undefined ? codes[+i] : _m));
}

// ── Project management: "add a project" = make it MANAGED (dashboard + oracle + topic)
const MC_CONFIG = `${OMEGA_DIR}/repos/omega-mc/config/omega-mc.yaml`;
// Add a project's dedicated oracle to the Mission-Control roster (idempotent) so it
// shows in the dashboard like the 13 managers + the atlas. omega-mc hot-reloads it.
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
async function addProject(name: string): Promise<string> {
  const dash = mcRegister(name);
  const pdir = repoPath(name) || "";
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
  return raw.split("\n").filter(c => c && c !== "OmegaOS" && !c.startsWith("."));
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
// talks to. "AISB" is the TEAM (13 Matrix manager agents + one dedicated oracle
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
let ATLAS_PROMPT = "";
try { ATLAS_PROMPT = readFileSync(`${OMEGA_DIR}/agents/aisb-atlas.md`, "utf8"); }
catch { try { ATLAS_PROMPT = readFileSync(`${OMEGA_DIR}/agents/aisb-master.md`, "utf8"); } catch {} }
// Live OmegaOS doctrine (Laws + operational Rules + orchestration + audits) pulled
// from the SINGLE source (`omega rules context <scope>`) and injected into every
// brain — so Atlas and the project oracles always know how OmegaOS is
// orchestrated and which rules/audits to respect, with NO re-explaining per prompt.
function doctrine(scope: string): string { try { return Bun.spawnSync([OMEGA, "rules", "context", scope]).stdout.toString().trim(); } catch { return ""; } }
const ATLAS_DOCTRINE = doctrine("master");
const ORACLE_DOCTRINE = doctrine("oracle");
const IDENTITY =
  "You are ATLAS of OmegaOS — the boss the operator talks to here on Telegram. " +
  "'AISB' is your TEAM, not your name: the 13 Matrix manager agents (oracle, morpheus, seraph, keymaker, niobe, smith, architect, merovingian, neo, zion, link, construct, pythia) plus one dedicated oracle per project. " +
  "You DIRECT them — dispatch to the right manager/oracle, or act directly with full VPS control. " +
  "When asked who you are, answer clearly: you are Atlas, directing the AISB team and the project oracles. Speak in the first person as Atlas.\n\n";
// One funnel for every headless-claude brain call (Atlas + the project oracles):
// lazy binary resolution, a JS-level 900s watchdog (proc.kill on expiry — portable,
// unlike GNU `timeout`), captured stdout/stderr, and the shared not-found /
// empty-output diagnostics. `who` labels the operator-facing messages.
async function runClaude(text: string, systemPrompt: string, addDir: string, who: string, cwd?: string): Promise<string> {
  const claude = resolveClaude();
  if (!claude) {
    return "Claude Code not found — install + log in on this machine:  omega install claude  then  claude  →  /login";
  }
  try {
    const proc = Bun.spawn([claude, "-p", text, "--append-system-prompt", systemPrompt, "--add-dir", addDir, "--dangerously-skip-permissions"], {
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
        }, CLAUDE_TIMEOUT_MS);
      })]);
      if (!r) return `(${who}: claude timed out after ${CLAUDE_TIMEOUT_MS / 1000}s and was killed — try again or split the request.)`;
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
  // (--add-dir /), permissions auto-approved. It dispatches to the 13 managers /
  // project oracles (omega dispatch) or acts directly. runClaude guards a stuck run.
  return runClaude(text, IDENTITY + ATLAS_PROMPT + "\n\n" + ATLAS_DOCTRINE, "/", "Atlas");
}

// ── Project oracle: an agent-bot's brain. Headless Claude SCOPED to one project —
// full project knowledge, commands the team (omega dispatch / workers / workflows)
// for THAT project only, and refuses to touch any other project.
let ORACLE_PERSONA = ""; try { ORACLE_PERSONA = readFileSync(`${OMEGA_DIR}/agents/aisb/oracle.md`, "utf8"); } catch {}
async function projectOracle(project: string, text: string): Promise<string> {
  const dir = repoPath(project) || gitRepos().find(r => r.name.toLowerCase() === project.toLowerCase())?.path || `${homedir()}/Station`;
  const scope =
    `You are the ORACLE of the project "${project}" — its dedicated orchestrator. Your ENTIRE world is this project at ${dir}: you have full knowledge of its code, history and state, and you orchestrate ONLY this project. ` +
    `You command the AISB team FOR ${project}: dispatch missions with \`omega dispatch ${project} "<mission>"\` (spawns oracle-${project}-<n> + workers/workflows), and use the 13 Matrix managers, workers and dynamic workflows — always in service of ${project} and nothing else. ` +
    `ORCHESTRATE, don't grind: for anything non-trivial, break it into a DYNAMIC WORKFLOW (fan-out → adversarially verify → synthesize) and/or workers/sub-tasks, each driven by a SMALL goal to reach (R-ORCH / R-GOAL). Define the success goal first, then dispatch and verify. ` +
    `STRICT SCOPE: never work on, modify, or discuss another project. If asked about anything outside ${project}, say it is out of scope and refocus on ${project}. Speak in the first person as the ${project} oracle.\n\n`;
  return runClaude(text, scope + ORACLE_PERSONA + "\n\n" + ORACLE_DOCTRINE, dir, `The ${project} oracle`, dir);
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
type Watch = { chat: number; thread?: number; mission: string; ts: number; oracle: string; project: string; msgId?: number };
const watching: Watch[] = [];
const reported = new Set<string>();
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
  try { writeFileSync(`${OMEGA_DIR}/state/${oracle}.progress.json`, JSON.stringify({ chat, thread: thread ?? null, msgId: msgId ?? null, project, oracle, mission, done: 0, total: 0, tasks: [] })); } catch {}
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
type Fragments = { texts: string[]; imgs: string[]; quoted: string; lastMsgId: number; timer?: ReturnType<typeof setTimeout> };
const fragments = new Map<string, Fragments>();
function queueMissionFragment(chat: number, thread: number | undefined, text: string, img: string, replyTo: string, msgId: number, fixedProject?: string) {
  const key = `${chat}:${thread ?? 0}:${fixedProject || ""}`;
  const f = fragments.get(key) || { texts: [], imgs: [], quoted: "", lastMsgId: msgId };
  if (text) f.texts.push(text);
  if (img) f.imgs.push(img);
  if (replyTo && !f.quoted) f.quoted = replyTo;
  f.lastMsgId = msgId;
  react(chat, msgId, "👀"); // received & buffering — the 🚀 lands at flush time
  if (f.timer) clearTimeout(f.timer);
  f.timer = setTimeout(() => { fragments.delete(key); flushMission(chat, thread, f, fixedProject).catch((e: any) => console.error("flushMission:", e?.message || e)); }, AGGREGATE_MS);
  fragments.set(key, f);
}
async function flushMission(chat: number, thread: number | undefined, f: Fragments, fixedProject?: string) {
  let text = f.texts.join("\n\n");
  if (f.imgs.length) {
    const many = f.imgs.length > 1;
    text = `${text || `Analyze the attached image${many ? "s" : ""} and act on ${many ? "them" : "it"}.`}\n\n## Attached image${many ? "s" : ""}\n${f.imgs.map((p) => `- ${p}`).join("\n")}\nOpen ${many ? "them" : "it"} with the Read tool and analyze ${many ? "them" : "it"} as part of this mission.`;
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
    try { p = JSON.parse(readFileSync(`${OMEGA_DIR}/state/${w.oracle}.progress.json`, "utf8")); } catch {}
    if (!p) continue;
    await edit(w.chat, w.msgId, progressCard(w.project, w.oracle, w.mission, p), undefined, w.thread);
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
    const proj = String(d.project || "").toLowerCase();
    const idx = watching.findIndex(w => finishedTs >= w.ts - 5000 && (w.oracle === d.oracle || (proj && w.oracle.toLowerCase().includes(proj))));
    if (idx < 0) continue;
    const w = watching[idx]; reported.add(f); watching.splice(idx, 1);
    const st = d.status || "done";
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
    try { const pj = JSON.parse(readFileSync(`${OMEGA_DIR}/state/${d.oracle || w.oracle}.progress.json`, "utf8")); plist = pj.tasks; pdone = pj.done || 0; ptot = pj.total || 0; } catch {}
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
    try { writeFileSync(`${f}.notified`, ""); } catch {}
    try { Bun.spawnSync(["rm", "-f", `${OMEGA_DIR}/state/${d.oracle || w.oracle}.progress.json`]); } catch {}
  }
}

// ── brain UX: instant ack + live "thinking" placeholder, then formatted reply ──
async function react(chat: number, msgId: number, emoji: string) {
  try { await tg("setMessageReaction", { chat_id: chat, message_id: msgId, reaction: [{ type: "emoji", emoji }] }); } catch {}
}
// One funnel for every brain call: 🤔 reaction (seen it) + a live placeholder, run
// the Master, then edit the placeholder with HTML-formatted output + ✅ reaction.
async function brainReply(chat: number, userMsgId: number, thread: number | undefined, prompt: string, brain: (t: string) => Promise<string> = master, label = "Atlas") {
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
    .then(out => {
      stop();
      histAppend(chat, thread, "assistant", out, label.toLowerCase()); // persist the reply (+ MC mirror)
      let html: string; try { html = mdToHtml(out); } catch { html = out; } // bad markup → raw text
      return phId ? edit(chat, phId, html, undefined, thread) : send(chat, html, undefined, thread);
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
type Pending = { kind: "login-code" | "new-project" | "add-project" | "import-project" | "tg-link" | "oracle-prompt"; ts: number; arg?: string };
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
  const out = await omega(["projects"]);
  return out.split("\n").map(l => l.trim()).filter(l => /^[A-Za-z0-9]/.test(l) && !/no projects|^Tip:|discovered|containing/i.test(l)).map(l => l.split(/\s+/)[0]);
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
const KNOWN = new Set<string>([...MENU.map(([c]) => c), "setupgroup", "sync", "dispatch"]);
function menuKb() {
  return kb([
    [{ text: "📖 Guide — how it works", callback_data: "nav:guide" }],
    [{ text: "🤖 Agents", callback_data: "nav:agents" }, { text: "🖥 Dashboard", callback_data: "nav:dashboard" }],
    [{ text: "📊 Status", callback_data: "nav:status" }, { text: "🗂 Sessions", callback_data: "nav:sessions" }],
    [{ text: "📁 Projects", callback_data: "nav:projects" }, { text: "🔍 Audits", callback_data: "nav:audits" }],
    [{ text: "💳 Account", callback_data: "nav:account" }, { text: "🧠 Model", callback_data: "nav:model" }],
    [{ text: "🧩 Skills", callback_data: "nav:skills" }, { text: "🚀 Dispatch", callback_data: "nav:dispatch" }],
    [{ text: "👥 Group hub", callback_data: "nav:setupgroup" }, { text: "🧹 Clean", callback_data: "nav:clean" }],
  ]);
}
const menuText = card("OMEGAOS — ACTION HUB", " Tap an action. Each one runs on your server via the <code>omega</code> CLI.");

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

// ── views ────────────────────────────────────────────────────────────────────
async function view(name: string): Promise<{ text: string; markup: any }> {
  switch (name) {
    case "menu": case "help": case "commands": return { text: menuText, markup: menuKb() };
    case "start": case "guide": return { text: await guideCard(), markup: kb([[{ text: "📋 Open menu", callback_data: "nav:menu" }], [{ text: "🚀 Dispatch", callback_data: "nav:dispatch" }, { text: "💳 Account", callback_data: "nav:account" }]]) };
    case "agents": {
      const ags = await mcAgents();
      if (!ags.length) return { text: card("AISB AGENTS", " ⚠️ Dashboard unreachable. Start it: <code>omega-mc-up</code>."), markup: kb([[back()]]) };
      const rows: Btn[][] = [];
      for (let i = 0; i < ags.length; i += 2) rows.push(ags.slice(i, i + 2).map(a => ({ text: a.id.slice(0, 28), callback_data: `agent:info:${a.id}`.slice(0, 64) })));
      return { text: card(`AISB AGENTS — ${ags.length}`, " Tap an agent for its role. To talk to it, use its dedicated bot (see /dashboard)."), markup: kb([...rows, [back()]]) };
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
    case "skills": return { text: pre("Skills", Bun.spawnSync(["ls", "-1", `${OMEGA_DIR}/skills`]).stdout.toString().trim() || "(none)"), markup: kb([[back()]]) };
    case "dispatch": return { text: card("DISPATCH", " Send: <code>/dispatch &lt;project&gt; &lt;mission&gt;</code>\n Launches a dedicated oracle on the VPS."), markup: kb([[{ text: "📁 Projects", callback_data: "nav:projects" }], [back()]]) };
    case "setupgroup": return { text: card("GROUP HUB", " Run <code>/setupgroup</code> <b>in a supergroup</b> where this bot is <b>admin</b> (Topics enabled). It registers the group as the project hub, then <code>/sync</code> maps each project to a topic."), markup: kb([[back()]]) };
    case "sync": { const g = loadGroups(); return { text: card("SYNC", g.hub ? " Hub registered. Run <code>/sync</code> in it to map projects → topics." : " No hub yet — run <code>/setupgroup</code> in your supergroup first."), markup: kb([[back()]]) }; }
    case "killall": return { text: card("KILL ALL SESSIONS?", " 🛑 Kills every session.\n <i>Keeps the infra (Home/System, bridge, master).</i>"), markup: kb([[{ text: "✅ Yes", callback_data: "do:killall" }], [{ text: "✖ Cancel", callback_data: "nav:menu" }]]) };
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
    // Auto-detect projects (top-level git repos under Station) and offer one button each.
    const repos = gitRepos();
    if (!repos.length) { setPending(from, "add-project"); return edit(chat, msgId, "<b>📁 Manage a project</b>\nNo project auto-detected — send the <b>name</b> of the project to manage.", kb([[{ text: "✖ Cancel", callback_data: "acct:cancel" }], [back("projects")]])); }
    const rows: Btn[][] = [];
    for (let i = 0; i < repos.length; i += 2) rows.push(repos.slice(i, i + 2).map(r => ({ text: `➕ ${r.name}`.slice(0, 28), callback_data: `proj:reg:${r.name}`.slice(0, 64) })));
    return edit(chat, msgId, `<b>📁 Add a project</b>\n${repos.length} project(s) detected under Station — tap a button to manage it (dedicated oracle + dashboard + topic).`, kb([...rows, [{ text: "✍️ Other (type the name)", callback_data: "proj:addname" }], [back("projects")]]));
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
  if (ns === "do" && action === "killall") return edit(chat, msgId, pre("kill-all", await omega(["kill-all"])), kb([[back("clean")]]));
  if (ns === "do" && action === "clean") return edit(chat, msgId, pre("cleanup", await omega(["cleanup"])), kb([[back("clean")]]));
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
    return edit(chat, msgId, `<b>🔗 Link a Telegram bot — ${esc(arg)}</b>\n1) Create a bot via @BotFather (or reuse one).\n2) Send me its <b>token</b> here (format <code>123456:ABC…</code>).\n\nThe bot will be <b>whitelisted to you alone</b>, and when you talk to it you'll be addressing the oracle of project <b>${esc(arg)}</b> (scoped to this project only).`, kb([[{ text: "✖ Cancel", callback_data: "acct:cancel" }], [back("agents")]]));
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
async function agentBotMain(agentId: string) {
  while (!loadConfig()) { console.log(`agent-bot ${agentId}: waiting for token in ${AGENT_BOTS_FILE} …`); await Bun.sleep(5000); }
  const project = loadAgentBots()[agentId]?.project || agentId;
  await tg("setMyCommands", { commands: [{ command: "start", description: `Talk to the ${project} project oracle` }] });
  await tg("deleteWebhook", { drop_pending_updates: false });
  console.log(`agent-bot up: ${agentId} → project ${project}, botId=${BOT_ID}, allow=${ALLOW.join(",")}`);
  setInterval(() => pollProgress().catch(() => {}), 6000);  // live progress card (▰▰▰░ %)
  setInterval(() => pollReports().catch(() => {}), 12000);  // Monitor: relay oracle done.json
  let offset = 0;
  while (true) {
    const r = await tg("getUpdates", { offset, timeout: 50, allowed_updates: ["message"] });
    if (!r.ok) { await Bun.sleep(2000); continue; }
    for (const u of r.result) {
      offset = u.update_id + 1;
      try {
        const msg = u.message; if (!msg?.text && !msg?.photo && !msg?.document) continue;
        const chatId = msg.chat.id, from = msg.from?.id ?? 0, thread = msg.message_thread_id;
        if (!allowed(from)) { console.log(`drop from ${from}`); continue; }
        let text = (msg.text || msg.caption || "").trim();
        // Photo / image document → download it locally; aggregated with the text below.
        const img = (msg.photo || msg.document) ? await saveIncomingImage(msg) : "";
        if (!text && !img) continue;
        if (text === "/start" || text === "/menu") { await send(chatId, `<b>🔮 Oracle — ${esc(project)}</b>\nWrite your mission: each message launches an <b>oracle dispatch</b> (a dedicated Claude Code session on the VPS) for project <b>${esc(project)}</b>. I relay the result back to you.`, undefined, thread); continue; }
        // A message to a project agent-bot = a MISSION → ONE real oracle session.
        // Album / caption-split fragments are buffered and flushed together.
        queueMissionFragment(chatId, thread, text, img, "", msg.message_id, project);
      } catch (e: any) { console.error("agent-bot update error:", e?.message || e); }
    }
  }
}

// ── poll loop ────────────────────────────────────────────────────────────────
async function main() {
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
        const msg = u.message; if (!msg?.text && !msg?.voice && !msg?.photo && !msg?.document) continue;
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
        // Photo / image document → download it locally. NOT baked into `text` here:
        // mission-bound fragments (album photos, caption-split prompts) are
        // aggregated by queueMissionFragment below; the command/pending paths
        // attach the single image themselves via withImageNote.
        const img = (msg.photo || msg.document) ? await saveIncomingImage(msg) : "";
        if (!text && !img) continue;
        // Reply-to-message: when the operator replies to a message, quote it as context
        // so the brain knows exactly what they're reacting to (e.g. reply to a report).
        const replyTo = (msg.reply_to_message?.text || msg.reply_to_message?.caption || "").slice(0, 2000);
        // Stateful flows awaiting a typed reply (login code paste, new-project brief).
        // A slash command cancels the pending flow and is processed normally.
        const p = getPending(from);
        if (p && !text.startsWith("/")) {
          clearPending(from);
          if (img) text = withImageNote(text, img); // single-message form for flows
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
            // Project = the agent id if it's a known project, else the agent id itself.
            const project = (repoPath(agentId) || gitRepos().find(r => r.name.toLowerCase() === agentId.toLowerCase())) ? agentId : agentId;
            const bots = loadAgentBots();
            bots[agentId] = { token, allow: ALLOW.slice(), project };
            saveAgentBots(bots);
            try { Bun.spawnSync(["chmod", "600", AGENT_BOTS_FILE]); } catch {}
            const spawn = spawnAgentBot(agentId);
            const me = `@${botInfo.result?.username || "?"}`;
            await send(chatId, spawn === "ok"
              ? `<b>✅ Bot linked to “${esc(agentId)}”</b>\nBot ${esc(me)} started, whitelisted to you alone.\nTalk to it: you're addressing the <b>${esc(project)} oracle</b> (scoped to this project — team, workers, workflows).`
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
          else if (KNOWN.has(cmd)) { const v = await view(cmd); await send(chatId, v.text, v.markup, thread); }
          else if (projectForCommand(cmd)) {
            // /{project} <mission> — talk straight to that project's oracle from the
            // main bot. `omega dispatch` gives it the full Atlas reprompting (project
            // knowledge + OmegaOS doctrine); we prepend the conversation context.
            const proj = projectForCommand(cmd)!;
            const mission = img ? withImageNote(a.join(" ").trim(), img) : a.join(" ").trim();
            if (!mission) { setPending(from, "oracle-prompt", proj); await send(chatId, card(`ORACLE — ${proj.toUpperCase()}`, ` 🔮 Send your <b>mission</b> for <b>${esc(proj)}</b> — I hand it to its oracle with the full Atlas reprompting (project + doctrine).`), kb([[{ text: "✖ Cancel", callback_data: "acct:cancel" }]]), thread); }
            else { react(chatId, msg.message_id, "🚀"); const ctx = histContext(chatId, thread); histAppend(chatId, thread, "operator", text, proj); const r = await dispatchToOracle(proj, mission, chatId, thread, ctx); if (r) await send(chatId, r, undefined, thread); }
          }
          else {
            // Unknown command → the AISB Master brain (commands gain intelligence:
            // any /verb the operator types is understood + dispatched, not dropped to the menu).
            await brainReply(chatId, msg.message_id, thread, img ? withImageNote(text, img) : text);
          }
        } else {
          // Free text in a project TOPIC = a MISSION → ONE real oracle session
          // (omega dispatch <project>); elsewhere / atlas topic → ATLAS. Fragments
          // belonging to one ask (album photos, caption-split prompts) are buffered
          // by queueMissionFragment and flushed together — routing, history and the
          // contextualized prompt are built at flush time (see flushMission).
          queueMissionFragment(chatId, thread, text, img, replyTo, msg.message_id);
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
