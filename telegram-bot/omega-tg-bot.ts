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

function readKV(path: string, re: RegExp): Record<string, string> {
  const out: Record<string, string> = {};
  try { for (const l of readFileSync(path, "utf8").split("\n")) { const m = l.match(re); if (m) out[m[1]] = m[2].replace(/^"|"$/g, ""); } } catch {}
  return out;
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
type Groups = { hub?: number; isForum?: boolean; topics?: Record<string, string>; atlas_topic?: number };
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
async function send(chat: number, text: string, markup?: any, thread?: number): Promise<any> {
  const body: any = { chat_id: chat, text: text.slice(0, 4096), parse_mode: "HTML", disable_web_page_preview: true, reply_markup: markup, message_thread_id: thread };
  const r = await tg("sendMessage", body);
  // HTML parse error (unbalanced tag from model output) → retry as plain text so a
  // message is NEVER silently dropped.
  if (!r.ok) return tg("sendMessage", { ...body, parse_mode: undefined });
  return r;
}
async function edit(chat: number, msgId: number, text: string, markup?: any, thread?: number): Promise<any> {
  const body: any = { chat_id: chat, message_id: msgId, text: text.slice(0, 4096), parse_mode: "HTML", disable_web_page_preview: true, reply_markup: markup };
  const r = await tg("editMessageText", body);
  if (!r.ok) {
    const r2 = await tg("editMessageText", { ...body, parse_mode: undefined });
    // Last resort (e.g. placeholder deleted): post a fresh message, keeping topic context.
    if (!r2.ok) await send(chat, text, markup, thread);
  }
  return r;
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
// Branded card: Ω-ruled header + body (+ optional ruled footer). `title` is plain text.
const card = (title: string, body: string, footer?: string) =>
  `${RULE}\n<b>Ω  ${esc(title)}</b>\n${RULE}\n${body}` + (footer ? `\n${RULE}\n${footer}` : "");
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
  let topicLine = "⚠️ Topic en attente — passe le groupe en <b>supergroupe + Topics activés</b> et ajoute le bot <b>admin (Manage Topics)</b>, puis /setupgroup et /sync.";
  if (g.hub && g.isForum) {
    const r = await tg("createForumTopic", { chat_id: g.hub, name: name.slice(0, 128) });
    if (r.ok) { g.topics ||= {}; g.topics[String(r.result.message_thread_id)] = name; saveGroups(g); recordProject(name, pdir, undefined, r.result.message_thread_id); topicLine = "✅ Topic Telegram créé dans le groupe."; }
    else topicLine = `⚠️ Topic non créé : <i>${esc(r.description || "erreur")}</i>.${/rights/i.test(r.description || "") ? " Active la permission <b>« Gérer les sujets »</b> pour le bot (admin du groupe)." : ""}`;
  }
  const dashLine = dash === "added" ? "ajouté ✅" : dash === "exists" ? "déjà présent ✅" : "non écrit ⚠️ (omega-mc config introuvable)";
  return `<b>📁 Projet « ${esc(name)} » géré</b>\n` +
    `• Oracle dédié (multi-session) : <code>omega dispatch ${esc(name)}</code> ✅\n` +
    `• Dashboard Mission Control : ${dashLine}\n` +
    `• ${topicLine}\n\n<i>Parle du projet dans son topic (ou ici) — Atlas connaît le contexte et dirige son oracle.</i>`;
}

// ── Managed projects = the SHARED registry the OmegaOS TUI (Project menu / oracle
// dispatch picker) reads: ~/.omega/projects.json (the Rust ProjectRegistry). Telegram
// writes HERE so Telegram, the TUI menu, and sessions stay in sync (single source of
// truth). Shape: { projects: [{ name, path, telegram_topic_id, oracle_session, … }] }.
const PROJECTS_FILE = `${OMEGA_DIR}/projects.json`;
type RegProject = { name: string; path: string; icon?: string | null; telegram_topic_id?: number | null; oracle_session?: string | null; git_email?: string | null; created_at: string };
function loadRegistry(): { projects: RegProject[] } { try { const r = JSON.parse(readFileSync(PROJECTS_FILE, "utf8")); return Array.isArray(r?.projects) ? r : { projects: [] }; } catch { return { projects: [] }; } }
function saveRegistry(r: { projects: RegProject[] }) { try { writeFileSync(PROJECTS_FILE, JSON.stringify(r, null, 2)); } catch {} }
// View shape kept stable: { name: { dir, category(derived), topic } }.
function loadProjects(): Record<string, { dir: string; category: string; topic?: number | null }> {
  const out: Record<string, { dir: string; category: string; topic?: number | null }> = {};
  for (const p of loadRegistry().projects) out[p.name] = { dir: p.path, category: p.path.split("/Station/")[1]?.split("/")[0] || "", topic: p.telegram_topic_id ?? null };
  return out;
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
// Delete a managed project. mode "soft" = remove from OmegaOS (Telegram topic +
// dashboard roster + registry + agent-bot); the code (GitHub + local folder) stays.
// mode "full" = soft + delete the GitHub repo (irreversible). Never touches local files
// unless asked; the local folder is left in place (operator can rm manually).
async function deleteProject(name: string, mode: "soft" | "full"): Promise<string> {
  const steps: string[] = [];
  const id = projId(name);
  // 1. Telegram topic
  const tid = projTopicId(name);
  if (tid != null) {
    const g = loadGroups();
    const r = await tg("deleteForumTopic", { chat_id: g.hub, message_thread_id: tid });
    if (r.ok || /not found|thread not found/i.test(r.description || "")) { delete g.topics![String(tid)]; saveGroups(g); steps.push("💬 Topic Telegram : supprimé ✅"); }
    else steps.push(`💬 Topic : ⚠️ ${esc(r.description || "échec")}`);
  } else steps.push("💬 Topic : (aucun)");
  // 2. Dashboard roster
  steps.push(mcUnregister(name) ? "🤖 Agent dashboard : retiré ✅" : "🤖 Agent dashboard : (absent)");
  // 3. Agent-bot service (if one was associated)
  const bots = loadAgentBots();
  if (bots[id] || bots[name]) {
    delete bots[id]; delete bots[name]; saveAgentBots(bots);
    Bun.spawnSync(["systemctl", "--user", "disable", "--now", `omega-tg-agent-${id}.service`]);
    steps.push("🔗 Bot agent dédié : arrêté + retiré ✅");
  }
  // 4. Shared registry (TUI menu stops seeing it too)
  removeProject(name);
  steps.push("📋 Registre projets (TUI + Telegram) : retiré ✅");
  // 5. GitHub repo (full only — irreversible)
  if (mode === "full") {
    const dir = repoPath(name) || loadProjects()[name]?.dir;
    let slug = "";
    if (dir) { const r = Bun.spawnSync(["bash", "-lc", `git -C ${dir} remote get-url origin 2>/dev/null`]).stdout.toString().trim(); slug = (r.match(/[:/]([^/]+\/[^/]+?)(?:\.git)?$/) || [])[1] || ""; }
    if (slug) {
      const del = Bun.spawnSync(["bash", "-lc", `gh repo delete ${slug} --yes 2>&1`]);
      steps.push(del.exitCode === 0 ? `🐙 Repo GitHub <code>${esc(slug)}</code> : SUPPRIMÉ ✅` : `🐙 GitHub : ⚠️ ${esc((del.stdout.toString() + del.stderr.toString()).trim().slice(0, 120))}`);
    } else steps.push("🐙 GitHub : ⚠️ remote introuvable (rien supprimé)");
    steps.push("📁 Dossier local : <b>conservé</b> (supprime-le à la main si besoin).");
  }
  return `<b>🗑 Projet « ${esc(name)} » supprimé (${mode === "full" ? "complet" : "OmegaOS only"})</b>\n${steps.join("\n")}`;
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
  steps.push(mk.stdout.toString().includes("ok") ? `📁 Dossier + git : <code>${dir}</code>` : `📁 Dossier : ⚠️ ${esc(mk.stderr.toString().slice(0, 120))}`);
  const dash = mcRegister(safe);
  recordProject(safe, dir, category);
  steps.push(`🤖 Agent oracle (dashboard) : ${dash === "added" ? "créé ✅" : dash === "exists" ? "déjà là ✅" : "⚠️"}`);
  const g = loadGroups();
  if (g.hub && g.isForum) {
    const r = await tg("createForumTopic", { chat_id: g.hub, name: safe.slice(0, 128) });
    if (r.ok) { g.topics ||= {}; g.topics[String(r.result.message_thread_id)] = safe; saveGroups(g); recordProject(safe, dir, undefined, r.result.message_thread_id); steps.push("💬 Topic Telegram : créé ✅"); }
    else steps.push(`💬 Topic Telegram : ⚠️ ${esc(r.description || "échec")}${/rights/i.test(r.description || "") ? " — active « Gérer les sujets » pour le bot" : ""}`);
  } else steps.push("💬 Topic Telegram : en attente (groupe forum + bot admin)");
  return { dir, report: `<b>🚀 Projet « ${esc(safe)} » créé dans ${esc(category)}</b>\n${steps.join("\n")}` };
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
  const path = repoPath(name); if (!path) return `repo introuvable: ${name}`;
  return gitOp(path, ["pull", "--ff-only"]);
}
function gitStatus(name: string): string {
  const path = repoPath(name); if (!path) return `repo introuvable: ${name}`;
  const branch = gitOp(path, ["rev-parse", "--abbrev-ref", "HEAD"]);
  return `branch: ${branch}\n` + gitOp(path, ["status", "-sb"]);
}
// add -A → commit → push. Safe when there's nothing to commit (push still runs).
function gitPush(name: string): string {
  const path = repoPath(name); if (!path) return `repo introuvable: ${name}`;
  gitOp(path, ["add", "-A"]);
  const commit = gitOp(path, ["commit", "-m", "update from Telegram (Atlas)"]);
  const push = gitOp(path, ["push"]);
  const cLine = /nothing to commit/.test(commit) ? "rien à committer" : (commit.split("\n").pop() || "commit ok");
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
const CLAUDE = process.env.CLAUDE_BIN || `${homedir()}/.local/bin/claude`;
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
async function master(text: string): Promise<string> {
  // Headless Claude AS ATLAS, full VPS control: every tool, whole-FS
  // (--add-dir /), permissions auto-approved. It dispatches to the 13 managers /
  // project oracles (omega dispatch) or acts directly. timeout guards a stuck run.
  try {
    const r = await $`timeout 900 ${CLAUDE} -p ${text} --append-system-prompt ${IDENTITY + ATLAS_PROMPT + "\n\n" + ATLAS_DOCTRINE} --add-dir / --dangerously-skip-permissions`
      .env({ ...process.env, OMEGA_DIR }).quiet().nothrow();
    const o = r.stdout.toString().trim();
    return o || "(Atlas n'a rien renvoyé — réessaie ou utilise /menu)";
  } catch (e: any) { return "Atlas error: " + (e?.message || e); }
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
  try {
    const r = await $`timeout 900 ${CLAUDE} -p ${text} --append-system-prompt ${scope + ORACLE_PERSONA + "\n\n" + ORACLE_DOCTRINE} --add-dir ${dir} --dangerously-skip-permissions`
      .cwd(dir).env({ ...process.env, OMEGA_DIR }).quiet().nothrow();
    const o = r.stdout.toString().trim();
    return o || `(L'oracle ${project} n'a rien renvoyé — réessaie.)`;
  } catch (e: any) { return `Oracle ${project} error: ${e?.message || e}`; }
}

// Provision a per-agent Telegram bot as its own systemd service (AGENT MODE). The
// token lives only in agent-bots.json (mode 600), never in the unit file.
function spawnAgentBot(agentId: string): string {
  try {
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
ExecStart=/usr/local/bin/bun %h/.omega/telegram-bot/omega-tg-bot.ts
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

// ── Dispatch to a REAL oracle session (the canonical path for project work). A
// message from a project topic / a project agent-bot / the "Parler à l'oracle"
// button is a MISSION for the VPS, not a chat: it spawns `omega dispatch <project>`
// → a visible Claude Code oracle session (its own mission; it delegates to dynamic
// workflows / workers / audit-review). The Monitor watches done.json and relays the
// result. The bot NEVER does project work itself (no headless brain).
type Watch = { chat: number; thread?: number; mission: string; ts: number; oracle: string };
const watching: Watch[] = [];
const reported = new Set<string>();
async function dispatchToOracle(project: string, mission: string, chat: number, thread: number | undefined): Promise<string> {
  const out = await omega(["dispatch", project, mission]);
  // Detect SUCCESS by the dispatch confirmation line — NOT by scanning the whole
  // output for "error"/"not found", because `omega dispatch` echoes the mission text
  // (`Mission: <text>`) and a prompt like "fix the error" would false-trigger failure.
  const m = out.match(/Oracle dispatched:?\s*(oracle-[A-Za-z0-9._-]+)/) || out.match(/oracle=(oracle-[A-Za-z0-9._-]+)/);
  if (!m) return card(`DISPATCH ${project.toUpperCase()} — ÉCHEC`, ` ❌ <pre>${esc(out).slice(0, 600)}</pre>`);
  const oracle = m[1];
  watching.push({ chat, thread, mission, ts: Date.now(), oracle });
  return card("🚀 ORACLE LANCÉ",
    ` 🔮 <code>${esc(oracle)}</code>\n 🎯 ${esc(mission).slice(0, 220)}\n\n <i>La session tourne sur le VPS (menu Session / dashboard MC). Je remonte le résultat dès que c'est fini.</i>`);
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
    const icon = st === "done_clean" ? "✅" : st === "failed" ? "❌" : st === "blocked" ? "🚧" : "⏹";
    const ship = d.ship?.commit ? `\n 📦 commit <code>${esc(d.ship.commit)}</code>` : "";
    const url = d.ship?.deploy_url ? `\n 🌐 ${esc(d.ship.deploy_url)}` : "";
    const head = ` ${icon} <code>${esc(d.oracle || w.oracle)}</code>\n 🎯 ${esc(w.mission).slice(0, 200)}`;
    await send(w.chat, card(`ORACLE — ${String(d.project || "").toUpperCase() || "MISSION"}`,
      `${head}\n\n${esc(d.summary || "(pas de résumé)").slice(0, 2500)}${ship}${url}`), undefined, w.thread);
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
  const ph = await tg("sendMessage", { chat_id: chat, parse_mode: "HTML", message_thread_id: thread, text: `🧠 <i>${label} réfléchit…</i>` });
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
      edit(chat, phId, `${frames[tick % frames.length]} <i>${label} réfléchit${dots}</i>  <code>${secs}s</code>`, undefined, thread);
    }
    tick++;
  }, 3000);
  const stop = () => { done = true; clearInterval(beat); };
  // Fire-and-forget BY DESIGN: the poll loop must never block on a 900s brain run.
  // The chain below always lands a final message (success OR error) for the operator.
  brain(prompt)
    .then(out => {
      stop();
      let html: string; try { html = mdToHtml(out); } catch { html = out; } // bad markup → raw text
      return phId ? edit(chat, phId, html, undefined, thread) : send(chat, html, undefined, thread);
    })
    .then(() => react(chat, userMsgId, "✅"))
    .catch(async () => {
      stop();
      react(chat, userMsgId, "⚠️");
      const m = "⚠️ AISB a rencontré une erreur — réessaie.";
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
type Pending = { kind: "login-code" | "new-project" | "add-project" | "tg-link" | "oracle-prompt"; ts: number; arg?: string };
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
  try { const c = JSON.parse(await oauth(["check"])); token = c.valid ? `valide (${c.remaining_min} min restantes)` : "⚠️ EXPIRÉ — clique « Login »"; } catch {}
  const usage = await omega(["usage"]);
  const tokenOk = /valide/i.test(token);
  return card("COMPTE CLAUDE (AISB)",
    ` 📧 ${esc(email)}\n 🎫 abo : ${esc(sub)}\n ${tokenOk ? "🟢" : "🔴"} token : ${esc(token)}`,
    `📊 <b>USAGE TOKENS</b>\n<pre>${esc(usage).slice(0, 1500)}</pre>`);
}

async function serviceAccounts(): Promise<string> {
  const env = readKV(`${OMEGA_DIR}/provisioning/services.env`, /^\s*export\s+([A-Z_]+)\s*=\s*"?([^"]*)"?\s*$/);
  const row = (label: string, key: string) => `${env[key] ? "✅" : "❌"} ${label}${env[key] ? "" : " — token manquant"}`;
  const staticTable = `<b>👤 Comptes de services (provisioning)</b>\n` +
    `${row("Vercel", "VERCEL_TOKEN")}\n${row("Convex", "CONVEX_TEAM_TOKEN")}\n${row("GitHub", "GITHUB_TOKEN")}\n` +
    `${row("Stripe", "STRIPE_SECRET_KEY")}\nClerk: ${esc(env.CLERK_PROVISION_MODE || "?")}\n\n` +
    `<i>Les ❌ requièrent ton token. Renseigne-les via le wizard Provisioning (TUI) ou édite ~/.omega/provisioning/services.env.</i>`;
  // Live probe of which accounts actually authenticate, when the CLI supports it
  // (graceful no-op on older binaries without `omega provision verify`).
  const probe = await omega(["provision", "verify", "default"]);
  const hasProbe = probe && !/error|unrecognized|unexpected argument|USAGE:|no output|not found/i.test(probe);
  return hasProbe ? `<b>🔎 Vérification live des comptes</b>\n<pre>${esc(probe).slice(0, 1400)}</pre>\n\n${staticTable}` : staticTable;
}

// Login / Re-auth — drives the real `claude /login` via the shared `omega
// claude-login` engine (spawns the visible `aisb-reauth` session, captures the
// authorize URL). UX: a "⏳ en cours" card while the URL generates (~15s — real
// OAuth, not instant), then the SAME message is replaced by a designed card with
// the link as a tappable button. Pasting the callback code back runs
// `omega claude-login-code`, which writes fresh creds to the SHARED store.
const TITLE_LOGIN = (s: boolean) => (s ? "CHANGER DE COMPTE" : "LOGIN / RE-AUTH");
async function startLogin(chat: number, msgId: number, from: number, switchAcct: boolean) {
  // 1) Waiting card (the wait is normal — house OAuth, browser-less).
  await edit(chat, msgId, card(TITLE_LOGIN(switchAcct),
    " ⏳ <b>Connexion en cours…</b>\n Génération du lien d'autorisation Claude.\n <i>~15 s — c'est l'auth OAuth, c'est normal.</i>"),
    kb([[back("account")]]));
  // 2) Drive the engine, pull the URL out of its JSON.
  const j = extractJson(await omega(["claude-login"]));
  const url: string = j?.url || "";
  if (!j?.ok || !/^https?:\/\//.test(url))
    return edit(chat, msgId, card(TITLE_LOGIN(switchAcct),
      ` ❌ <b>Lien non généré.</b>\n <i>Réessaie dans un instant.</i>`),
      kb([[{ text: "🔄 Réessayer", callback_data: "acct:login" }], [back("account")]]));
  // 3) Replace the waiting card with the designed link card + button.
  setPending(from, "login-code");
  await edit(chat, msgId, card(TITLE_LOGIN(switchAcct),
    ` 🔗 <b>1.</b> Ouvre le lien et autorise${switchAcct ? " <b>avec l'autre compte</b>" : " avec ton compte Max"}.\n` +
    ` 🔑 <b>2.</b> Copie le <b>code</b> de la page de callback et <b>colle-le ici</b> (prochain message).`,
    "<i>Un seul login pour tout OmegaOS — le credential est partagé entre toutes les sessions.</i>"),
    kb([[{ text: "🔐 Ouvrir & autoriser", url }], [{ text: "✖ Annuler", callback_data: "acct:cancel" }]]));
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
  ["killall", "Kill all sessions (keeps infra)"],
  ["clean", "Cleanup stray sessions + state"],
  ["help", "Show the action hub"],
];
// Commands with a dedicated button view/handler. Anything NOT here is routed to
// the AISB Master brain instead of falling back to the menu (intelligent commands).
const KNOWN = new Set<string>([...MENU.map(([c]) => c), "setupgroup", "sync", "dispatch"]);
function menuKb() {
  return kb([
    [{ text: "🤖 Agents", callback_data: "nav:agents" }, { text: "🖥 Dashboard", callback_data: "nav:dashboard" }],
    [{ text: "📊 Status", callback_data: "nav:status" }, { text: "🗂 Sessions", callback_data: "nav:sessions" }],
    [{ text: "📁 Projects", callback_data: "nav:projects" }, { text: "🔍 Audits", callback_data: "nav:audits" }],
    [{ text: "💳 Account", callback_data: "nav:account" }, { text: "🧠 Model", callback_data: "nav:model" }],
    [{ text: "🧩 Skills", callback_data: "nav:skills" }, { text: "🚀 Dispatch", callback_data: "nav:dispatch" }],
    [{ text: "👥 Group hub", callback_data: "nav:setupgroup" }, { text: "🧹 Clean", callback_data: "nav:clean" }],
    [{ text: "🛑 Kill all", callback_data: "nav:killall" }],
  ]);
}
const menuText = card("OMEGAOS — ACTION HUB", " Tape une action. Chacune tourne sur ton serveur via le CLI <code>omega</code>.");

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
    body += `\n\n ❗ <b>À CORRIGER</b>\n` + warns.map(w => `  ${dot(sev(w.value))} <b>${esc(w.label)}</b> — ${esc(cleanDetail(w.value))}`).join("\n");
  const details = checks.map(c => ` ${dot(c.ok ? "ok" : sev(c.value))} <b>${esc(c.label.toUpperCase())}</b>  ${esc(cleanDetail(c.value))}`).join("\n");
  body += `\n\n<blockquote expandable>▾ ${total} checks système\n${details}</blockquote>`;
  return `${RULE}\n   Ω  O M E G A O S\n${RULE}\n${body}\n${RULE}`;
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
async function modelProviderView(provider: string, banner = ""): Promise<{ text: string; markup: any }> {
  const [models, cur] = await Promise.all([listModels(provider), currentModel(provider)]);
  const rows: Btn[][] = [];
  for (let i = 0; i < models.length; i += 2)
    rows.push(models.slice(i, i + 2).map(m => ({ text: `${m === cur ? "✓ " : ""}${m}`.slice(0, 28), callback_data: `model:set:${provider}:${m}`.slice(0, 64) })));
  const body = (banner ? banner + "\n\n" : "") + (models.length
    ? ` Actuel : <code>${esc(cur || "défaut")}</code>\n Tape un modèle pour l'activer.`
    : ` Aucun modèle catalogué. Configure : <code>omega config set ${esc(provider)}.model …</code>`);
  return { text: card(`MODÈLE — ${provider.toUpperCase()}`, body), markup: kb([...rows, [{ text: "« Providers", callback_data: "nav:model" }]]) };
}

// ── views ────────────────────────────────────────────────────────────────────
async function view(name: string): Promise<{ text: string; markup: any }> {
  switch (name) {
    case "menu": case "help": case "commands": return { text: menuText, markup: menuKb() };
    case "agents": {
      const ags = await mcAgents();
      if (!ags.length) return { text: card("AISB AGENTS", " ⚠️ Dashboard injoignable. Démarre-le : <code>omega-mc-up</code>."), markup: kb([[back()]]) };
      const rows: Btn[][] = [];
      for (let i = 0; i < ags.length; i += 2) rows.push(ags.slice(i, i + 2).map(a => ({ text: a.id.slice(0, 28), callback_data: `agent:info:${a.id}`.slice(0, 64) })));
      return { text: card(`AISB AGENTS — ${ags.length}`, " Tape un agent pour son rôle. Pour lui parler, utilise son bot dédié (voir /dashboard)."), markup: kb([...rows, [back()]]) };
    }
    case "dashboard": {
      await resolvePublicIP();
      const { url } = dashboardURL();
      const rows: Btn[][] = [];
      if (url) rows.push([{ text: "👉 Cliquez ici pour ouvrir", url }]);
      rows.push([{ text: "🔑 Révéler le password", callback_data: "dash:pw" }]);
      rows.push([back()]);
      const body = url
        ? ` <code>${esc(url)}</code>\n\n Tape « 👉 Ouvrir » pour le dashboard, puis « 🔑 Révéler » pour le mot de passe.`
        : ` ⚠️ IP publique non résolue — réessaie, ou active Tailscale pour un accès sécurisé.`;
      return { text: card("MISSION CONTROL", body), markup: kb(rows) };
    }
    case "status": return { text: statusCard(await omega(["doctor"])), markup: kb([[{ text: "🛠 Fix it", callback_data: "status:fix" }, { text: "🔄 Refresh", callback_data: "nav:status" }], [back()]]) };
    case "sessions": {
      const names = await sessionNames();
      const rows = names.slice(0, 12).map(s => [{ text: `📊 ${s}`.slice(0, 30), callback_data: `sess:status:${s}`.slice(0, 64) }, { text: "🛑 Kill", callback_data: `sess:kill:${s}`.slice(0, 64) }]);
      const list = names.length ? names.map(s => ` 🟢 <code>${esc(s)}</code>`).join("\n") : " <i>Aucune session active.</i>";
      return { text: card(`SESSIONS — ${names.length}`, list), markup: kb([...rows, [{ text: "🔄 Refresh", callback_data: "nav:sessions" }, back()]]) };
    }
    case "projects": {
      const mp = loadProjects();
      const names = Object.keys(mp).sort();
      const list = names.length
        ? names.map(n => `• <b>${esc(n)}</b> <i>(${esc(mp[n].category || "?")})</i>`).join("\n")
        : "<i>Aucun projet géré pour l'instant — ajoute-en un (📁) ou crée-en un (➕).</i>";
      const rows: Btn[][] = [];
      for (let i = 0; i < names.length; i += 2) rows.push(names.slice(i, i + 2).map(n => ({ text: `📦 ${n}`.slice(0, 28), callback_data: `proj:open:${n}`.slice(0, 64) })));
      return { text: card(`PROJETS — ${names.length}`, list), markup: kb([...rows, [{ text: "➕ New", callback_data: "proj:new" }, { text: "📁 Add existing", callback_data: "proj:add" }], [{ text: "🔧 Git", callback_data: "git:list" }, { text: "🔁 Sync", callback_data: "nav:sync" }], [back()]]) };
    }
    case "audits": {
      const ids = await auditIds(); const rows: Btn[][] = [];
      for (let i = 0; i < ids.length; i += 2) rows.push(ids.slice(i, i + 2).map(a => ({ text: a.slice(0, 28), callback_data: `aud:run:${a}`.slice(0, 64) })));
      return { text: card("QUALITY ARSENAL", ` ${ids.length} audits disponibles — tape-en un pour le lancer.`), markup: kb([...rows, [back()]]) };
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
      const body = ` Sessions omega tournent sur :\n <b>claude</b> · <code>${esc(active || "défaut")}</code>\n\n Choisis un provider pour voir et changer son modèle.`;
      return { text: card("MODÈLE / PROVIDERS", body), markup: kb([...rows, [{ text: "🔄 Refresh", callback_data: "nav:model" }, back()]]) };
    }
    case "skills": return { text: pre("Skills", Bun.spawnSync(["ls", "-1", `${OMEGA_DIR}/skills`]).stdout.toString().trim() || "(none)"), markup: kb([[back()]]) };
    case "dispatch": return { text: card("DISPATCH", " Envoie : <code>/dispatch &lt;projet&gt; &lt;mission&gt;</code>\n Lance un oracle dédié sur le VPS."), markup: kb([[{ text: "📁 Projects", callback_data: "nav:projects" }], [back()]]) };
    case "setupgroup": return { text: card("GROUP HUB", " Lance <code>/setupgroup</code> <b>dans un supergroupe</b> où ce bot est <b>admin</b> (Topics activés). Ça enregistre le groupe comme hub projets, puis <code>/sync</code> mappe chaque projet sur un topic."), markup: kb([[back()]]) };
    case "sync": { const g = loadGroups(); return { text: card("SYNC", g.hub ? " Hub enregistré. Lance <code>/sync</code> dedans pour mapper projets → topics." : " Pas encore de hub — lance <code>/setupgroup</code> dans ton supergroupe d'abord."), markup: kb([[back()]]) }; }
    case "killall": return { text: card("KILL ALL SESSIONS ?", " 🛑 Tue toutes les sessions.\n <i>Garde l'infra (Home/System, bridge, master).</i>"), markup: kb([[{ text: "✅ Oui", callback_data: "do:killall" }], [{ text: "✖ Annuler", callback_data: "nav:menu" }]]) };
    case "clean": return { text: card("CLEANUP ?", " 🧹 Tue les sessions orphelines + purge le state.\n <i>Ne touche jamais à l'infra.</i>"), markup: kb([[{ text: "✅ Oui", callback_data: "do:clean" }], [{ text: "✖ Annuler", callback_data: "nav:menu" }]]) };
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
    if (!warns.length) return edit(chat, msgId, card("OMEGAOS — FIX IT", " ✅ Rien à corriger — tout est vert."), kb([[{ text: "« Status", callback_data: "nav:status" }]]));
    const mission = `Auto-heal OmegaOS. \`omega doctor\` signale ces problèmes — diagnostique la cause racine et corrige chacun (tu peux t'appuyer sur \`omega doctor --fix\` pour les correctifs mécaniques), puis vérifie avec \`omega doctor\` que tout repasse au vert :\n` + warns.map(w => `- ${w}`).join("\n");
    // Make `omega dispatch OmegaOS` resolve: the OS repo isn't auto-discovered
    // (it's a sibling of the container dirs), so register it in the shared
    // registry first (idempotent — recordProject upserts by path/name).
    recordProject("OmegaOS", repoPath("OmegaOS") || `${homedir()}/Station/OmegaOS`);
    const out = await dispatchToOracle("OmegaOS", mission, chat, undefined);
    return edit(chat, msgId, out, kb([[{ text: "« Status", callback_data: "nav:status" }]]));
  }
  if (ns === "model" && action === "prov") { const v = await modelProviderView(arg); return edit(chat, msgId, v.text, v.markup); }
  if (ns === "model" && action === "set") {
    // arg = "provider:model" — model may contain "/" (openrouter ids), never ":".
    const i = arg.indexOf(":"); const provider = arg.slice(0, i); const model = arg.slice(i + 1);
    const res = await omega(["config", "set", `${provider}.model`, model]);
    const okOmega = /^\[\+\] Set/m.test(res);
    let dash = "";
    if (provider === "claude") {
      const full = CLAUDE_FULL_ID[model] || model;
      const wrote = mcSetDefaultModel(full);
      dash = `\n 🖥 Dashboard defaults : ${wrote ? `<code>${esc(full)}</code> ✅ <i>(hot-reload ~3s)</i>` : "inchangé"}`;
    }
    const banner = ` ${okOmega ? "✅" : "⚠️"} <b>${esc(provider)}</b> → <code>${esc(model)}</code>\n ⚙️ Sessions omega : ${okOmega ? "✅" : "⚠️ " + esc(res.slice(0, 80))}${dash}`;
    const v = await modelProviderView(provider, banner);
    return edit(chat, msgId, v.text, v.markup);
  }
  if (ns === "dash" && action === "pw") {
    const { pw } = dashboardURL();
    if (!pw) return;
    // Reveal in a copyable code block, then auto-delete after 30s (so it never lingers in chat history).
    const m = await tg("sendMessage", { chat_id: chat, parse_mode: "HTML", text: `🔑 <b>Password du dashboard</b>\n(tape dessus pour copier — s'efface dans 30s)\n\n<code>${esc(pw)}</code>` });
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
    return edit(chat, msgId, "<b>➕ Nouveau projet</b>\nDans quel dossier (catégorie) sous Station ?", kb([...rows, [back("projects")]]));
  }
  if (ns === "proj" && action === "newcat") {
    setPending(from, "new-project", arg);
    return edit(chat, msgId, `<b>➕ Nouveau projet — ${esc(arg)}</b>\nEnvoie en <b>un message</b> :\n• <b>1ère ligne</b> = nom du projet\n• <b>lignes suivantes</b> = description (ce que c'est, ce qu'on veut faire)\n\nJe crée le dossier + git, l'oracle dédié, le topic, puis je <b>lance l'oracle</b> sur ta description pour démarrer direct.`, kb([[{ text: "✖ Annuler", callback_data: "acct:cancel" }], [back("projects")]]));
  }
  if (ns === "proj" && action === "add") {
    // Auto-detect projects (top-level git repos under Station) and offer one button each.
    const repos = gitRepos();
    if (!repos.length) { setPending(from, "add-project"); return edit(chat, msgId, "<b>📁 Gérer un projet</b>\nAucun projet auto-détecté — envoie le <b>nom</b> du projet à gérer.", kb([[{ text: "✖ Annuler", callback_data: "acct:cancel" }], [back("projects")]])); }
    const rows: Btn[][] = [];
    for (let i = 0; i < repos.length; i += 2) rows.push(repos.slice(i, i + 2).map(r => ({ text: `➕ ${r.name}`.slice(0, 28), callback_data: `proj:reg:${r.name}`.slice(0, 64) })));
    return edit(chat, msgId, `<b>📁 Ajouter un projet</b>\n${repos.length} projet(s) détecté(s) sous Station — tape un bouton pour le gérer (oracle dédié + dashboard + topic).`, kb([...rows, [{ text: "✍️ Autre (taper le nom)", callback_data: "proj:addname" }], [back("projects")]]));
  }
  if (ns === "proj" && action === "reg") return edit(chat, msgId, await addProject(arg), kb([[{ text: "📁 Ajouter un autre", callback_data: "proj:add" }, { text: "📋 Projets", callback_data: "nav:projects" }], [back("projects")]]));
  if (ns === "proj" && action === "addname") { setPending(from, "add-project"); return edit(chat, msgId, "<b>📁 Gérer un projet</b>\nEnvoie le <b>nom</b> du projet à gérer.", kb([[{ text: "✖ Annuler", callback_data: "acct:cancel" }], [back("projects")]])); }
  if (ns === "proj" && action === "open") {
    const mp = loadProjects()[arg];
    // One coherent primary action: dispatch a mission to the project's dedicated
    // oracle (proj:oracle → `omega dispatch <project>` — a real tracked oracle
    // session whose result is relayed back). "Talk to oracle" was a misnomer (it's
    // a fire-and-track dispatch, not a live chat) and duplicated this exact button.
    return edit(chat, msgId, `<b>📦 ${esc(arg)}</b>${mp ? `\n<i>${esc(mp.category || "")}</i> · <code>${esc(mp.dir || "")}</code>` : ""}`, kb([
      [{ text: "🚀 Dispatch mission", callback_data: `proj:oracle:${arg}`.slice(0, 64) }],
      [{ text: "🔧 Git", callback_data: `git:menu:${arg}`.slice(0, 64) }, { text: "🗑 Supprimer", callback_data: `proj:del:${arg}`.slice(0, 64) }],
      [back("projects")],
    ]));
  }
  if (ns === "proj" && action === "del") {
    return edit(chat, msgId, `<b>🗑 Supprimer « ${esc(arg)} »</b>\nChoisis :\n• <b>OmegaOS only</b> — retire le topic Telegram, l'agent du dashboard et le registre. <i>Le code (GitHub + dossier) reste.</i>\n• <b>Complet</b> — tout ça <b>+ supprime le repo GitHub</b> (irréversible). Le dossier local est conservé.`, kb([
      [{ text: "🧹 OmegaOS only", callback_data: `proj:delsoft:${arg}`.slice(0, 64) }],
      [{ text: "💥 Complet (+ GitHub)", callback_data: `proj:delfull:${arg}`.slice(0, 64) }],
      [{ text: "✖ Annuler", callback_data: `proj:open:${arg}`.slice(0, 64) }],
    ]));
  }
  if (ns === "proj" && action === "delsoft") return edit(chat, msgId, await deleteProject(arg, "soft"), kb([[{ text: "📋 Projets", callback_data: "nav:projects" }]]));
  if (ns === "proj" && action === "delfull") {
    // Extra confirmation for the irreversible GitHub deletion.
    return edit(chat, msgId, `<b>💥 Suppression COMPLÈTE de « ${esc(arg)} »</b>\n⚠️ Ça <b>supprime le repo GitHub</b> (irréversible) en plus du reste. Sûr ?`, kb([
      [{ text: "💥 Oui, tout supprimer", callback_data: `proj:delfullgo:${arg}`.slice(0, 64) }],
      [{ text: "✖ Annuler", callback_data: `proj:open:${arg}`.slice(0, 64) }],
    ]));
  }
  if (ns === "proj" && action === "delfullgo") return edit(chat, msgId, await deleteProject(arg, "full"), kb([[{ text: "📋 Projets", callback_data: "nav:projects" }]]));
  if (ns === "proj" && action === "oracle") {
    setPending(from, "oracle-prompt", arg);
    return edit(chat, msgId, `<b>🔮 Oracle — ${esc(arg)}</b>\nEnvoie ton <b>prompt / ta mission</b>. Je le donne à l'oracle dédié de <b>${esc(arg)}</b> (reprompting complet : connaissance projet + toute la doctrine OmegaOS — orchestration, dynamic workflows, workers, goals, audits) — scopé à ce projet.`, kb([[{ text: "✖ Annuler", callback_data: "acct:cancel" }], [{ text: "« Projet", callback_data: `proj:open:${arg}`.slice(0, 64) }]]));
  }
  if (ns === "git" && action === "list") {
    const repos = gitRepos();
    if (!repos.length) return edit(chat, msgId, "<b>🔧 Git</b>\nAucun repo git trouvé sous la racine projets.", kb([[back("projects")]]));
    const rows: Btn[][] = [];
    for (let i = 0; i < repos.length; i += 2) rows.push(repos.slice(i, i + 2).map(r => ({ text: `📦 ${r.name}`.slice(0, 28), callback_data: `git:menu:${r.name}`.slice(0, 64) })));
    return edit(chat, msgId, `<b>🔧 Git — ${repos.length} repo(s)</b>\nChoisis un projet pour pull / add+push / status.`, kb([...rows, [back("projects")]]));
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
  if (ns === "acct" && action === "cancel") { clearPending(from); return edit(chat, msgId, "Annulé.", kb([[back("account")]])); }
  if (ns === "do" && action === "killall") return edit(chat, msgId, pre("kill-all", await omega(["kill-all"])), kb([[back("menu")]]));
  if (ns === "do" && action === "clean") return edit(chat, msgId, pre("cleanup", await omega(["cleanup"])), kb([[back("menu")]]));
  if (ns === "agent" && action === "info") { const a = (await mcAgents()).find(x => x.id === arg); return edit(chat, msgId, `<b>🤖 ${esc(arg)}</b>\n${esc(a?.description || "(no description)")}\n\n<i>Associe un bot Telegram dédié à cet agent — tu lui parleras directement (scopé à son projet).</i>`, kb([[{ text: "🔗 Associer Telegram", callback_data: `agent:tglink:${arg}`.slice(0, 64) }], [back("agents")]])); }
  if (ns === "agent" && action === "tglink") {
    setPending(from, "tg-link", arg);
    return edit(chat, msgId, `<b>🔗 Associer un bot Telegram — ${esc(arg)}</b>\n1) Crée un bot via @BotFather (ou réutilises-en un).\n2) Envoie-moi son <b>token</b> ici (format <code>123456:ABC…</code>).\n\nLe bot sera <b>whitelisté à toi seul</b>, et quand tu lui parleras tu t'adresseras à l'oracle du projet <b>${esc(arg)}</b> (scopé à ce projet uniquement).`, kb([[{ text: "✖ Annuler", callback_data: "acct:cancel" }], [back("agents")]]));
  }
  return edit(chat, msgId, menuText, menuKb());
}

// ── group setup: verify the bot is admin, register the supergroup as hub ─────
async function cmdSetupGroup(chat: any, chatId: number, thread?: number) {
  if (chat.type !== "group" && chat.type !== "supergroup") return send(chatId, "⚠️ Lance <code>/setupgroup</code> <b>dans le groupe</b> (un supergroupe avec les Sujets/Topics activés).", undefined, thread);
  // getChat to read the live is_forum (the message's chat object can be stale).
  const info = await tg("getChat", { chat_id: chatId });
  const isForum = info.ok ? !!info.result?.is_forum : !!chat.is_forum;
  const admins = await tg("getChatAdministrators", { chat_id: chatId });
  const me = admins.ok ? admins.result.find((a: any) => a.user?.id === BOT_ID) : null;
  if (!me) return send(chatId, "⚠️ Je ne suis <b>pas admin</b> ici. Ajoute-moi <b>administrateur</b> avec la permission <b>« Gérer les sujets »</b> (Manage Topics), puis relance <code>/setupgroup</code>.", undefined, thread);
  const canTopics = me.status === "creator" || me.can_manage_topics === true;
  const g = loadGroups(); g.hub = chatId; g.isForum = isForum; g.topics ||= {}; saveGroups(g);
  let msg = "✅ Groupe enregistré comme <b>hub projets</b>.";
  if (!isForum) msg += "\n⚠️ Les <b>Sujets/Topics ne sont pas activés</b> — active-les dans les réglages du groupe, puis relance /setupgroup.";
  else if (!canTopics) msg += "\n⚠️ Il me manque la permission <b>« Gérer les sujets »</b> : ouvre mes droits admin (toi → groupe → admins → ce bot) et active <b>Manage Topics</b>, puis lance <code>/sync</code>.";
  else msg += "\n✅ Topics activés + droits OK — lance <code>/sync</code> pour créer un topic par projet.";
  return send(chatId, msg, undefined, thread);
}

// ── sync: one forum topic per project; route topic messages to its oracle ────
async function cmdSync(chatId: number, thread?: number) {
  const g = loadGroups();
  if (!g.hub) return send(chatId, "⚠️ Pas de hub — lance d'abord <code>/setupgroup</code> dans ton supergroupe.", undefined, thread);
  if (!g.isForum) return send(chatId, "⚠️ Les Sujets/Topics ne sont pas activés — active-les, relance /setupgroup, puis /sync.", undefined, thread);
  g.topics ||= {};
  // Ensure the Atlas topic exists — where reports that don't belong to a project
  // (OmegaOS-self, cross-project) are posted instead of the operator DM.
  if (!g.atlas_topic || !Object.keys(g.topics).includes(String(g.atlas_topic))) {
    const existing = Object.entries(g.topics).find(([, n]) => String(n).toLowerCase() === "atlas")?.[0];
    if (existing) { g.atlas_topic = Number(existing); }
    else {
      const ar = await tg("createForumTopic", { chat_id: g.hub, name: "Atlas 🎩", icon_color: 7322096 });
      if (ar.ok) { g.atlas_topic = ar.result.message_thread_id; g.topics[String(ar.result.message_thread_id)] = "atlas"; saveGroups(g); }
    }
  }
  const mp = loadProjects();
  const names = Object.keys(mp);
  if (!names.length) return send(g.hub, "Aucun projet géré — ajoute (📁) ou crée (➕) un projet, puis /sync.", undefined, thread);
  let made = 0; let recreated = 0; let err = "";
  for (const p of names) {
    // Reverse-lookup the project's currently-mapped topic id (if any).
    const mappedTid = Object.entries(g.topics).find(([, n]) => String(n).toLowerCase() === p.toLowerCase())?.[0];
    if (mappedTid) {
      // Verify the topic STILL EXISTS on Telegram — a no-op rename probes it. If it
      // was deleted in the group, recreate it (this is the resilience the operator wants).
      const probe = await tg("editForumTopic", { chat_id: g.hub, message_thread_id: Number(mappedTid), name: p.slice(0, 128) });
      if (probe.ok) continue; // topic alive → keep
      if (/rights|manage/i.test(probe.description || "")) { err = probe.description || "droits"; break; }
      delete g.topics[mappedTid]; // stale mapping (topic deleted) → drop + recreate below
    }
    const r = await tg("createForumTopic", { chat_id: g.hub, name: p.slice(0, 128) });
    if (r.ok) { g.topics[String(r.result.message_thread_id)] = p; recordProject(p, mp[p].dir || "", undefined, r.result.message_thread_id); if (mappedTid) recreated++; else made++; }
    else { err = r.description || "échec"; break; }
  }
  saveGroups(g);
  if (err) return send(g.hub, `⚠️ Sync interrompu : <i>${esc(err)}</i>.${/rights|manage/i.test(err) ? "\nActive la permission <b>« Gérer les sujets »</b> pour le bot (admin du groupe), puis relance /sync." : ""}\n(${made} créé(s), ${recreated} recréé(s) avant l'arrêt)`, undefined, thread);
  return send(g.hub, `🔁 Sync OK. ${made} nouveau(x) topic(s)${recreated ? `, ${recreated} recréé(s) (topics supprimés détectés)` : ""} ; ${Object.keys(g.topics).length} topic(s) projet au total. Les messages dans le topic d'un projet sont routés vers son oracle.`, undefined, thread);
}

// ── AGENT MODE poll loop: a per-agent bot. Whitelisted to the operator; every
// message goes straight to that project's scoped oracle (no menu, no other project).
async function agentBotMain(agentId: string) {
  while (!loadConfig()) { console.log(`agent-bot ${agentId}: waiting for token in ${AGENT_BOTS_FILE} …`); await Bun.sleep(5000); }
  const project = loadAgentBots()[agentId]?.project || agentId;
  await tg("setMyCommands", { commands: [{ command: "start", description: `Parler à l'oracle du projet ${project}` }] });
  await tg("deleteWebhook", { drop_pending_updates: false });
  console.log(`agent-bot up: ${agentId} → project ${project}, botId=${BOT_ID}, allow=${ALLOW.join(",")}`);
  setInterval(() => pollReports().catch(() => {}), 15000); // Monitor: relay oracle done.json
  let offset = 0;
  while (true) {
    const r = await tg("getUpdates", { offset, timeout: 50, allowed_updates: ["message"] });
    if (!r.ok) { await Bun.sleep(2000); continue; }
    for (const u of r.result) {
      offset = u.update_id + 1;
      try {
        const msg = u.message; if (!msg?.text) continue;
        const chatId = msg.chat.id, from = msg.from?.id ?? 0, text = msg.text.trim(), thread = msg.message_thread_id;
        if (!allowed(from)) { console.log(`drop from ${from}`); continue; }
        if (text === "/start" || text === "/menu") { await send(chatId, `<b>🔮 Oracle — ${esc(project)}</b>\nÉcris ta mission : chaque message lance un <b>dispatch oracle</b> (session Claude Code dédiée sur le VPS) pour le projet <b>${esc(project)}</b>. Je te remonte le résultat.`, undefined, thread); continue; }
        // A message to a project agent-bot = a MISSION → dispatch a real oracle session.
        react(chatId, msg.message_id, "🚀");
        await send(chatId, await dispatchToOracle(project, text, chatId, thread), undefined, thread);
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
  const cmds = MENU.map(([command, description]) => ({ command, description }));
  await tg("setMyCommands", { commands: cmds });
  await tg("setMyCommands", { commands: cmds, scope: { type: "all_private_chats" } });
  await tg("deleteWebhook", { drop_pending_updates: false });
  await resolvePublicIP();
  console.log(`omega-tg-bot v3 up. botId=${BOT_ID} commands=${MENU.length} allow=${ALLOW.join(",") || "ALL"}`);
  setInterval(() => pollReports().catch(() => {}), 15000); // Monitor: relay oracle done.json reports
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
        const msg = u.message; if (!msg?.text) continue;
        const chat = msg.chat, chatId = chat.id, from = msg.from?.id ?? 0, text = msg.text.trim();
        const thread = msg.message_thread_id;
        if (!allowed(from)) { console.log(`drop from ${from}`); continue; }
        // Stateful flows awaiting a typed reply (login code paste, new-project brief).
        // A slash command cancels the pending flow and is processed normally.
        const p = getPending(from);
        if (p && !text.startsWith("/")) {
          clearPending(from);
          if (p.kind === "login-code") {
            await tg("sendChatAction", { chat_id: chatId, action: "typing", message_thread_id: thread });
            // Paste the code into the waiting `aisb-reauth` session; the engine
            // waits for Claude to write fresh creds, then syncs them to the SHARED
            // store and re-establishes the symlink (atomic — no 0-byte truncation).
            const j = extractJson(await omega(["claude-login-code", text]));
            const ok = !!j?.ok;
            const body = ok
              ? ` ✅ <b>Connecté</b>\n 📧 ${esc(j.email || "?")}\n ⏱ token frais — ${j.expires_min || "?"} min\n 🔗 credential partagé mis à jour (toutes les sessions).`
              : ` ❌ <b>Échec de validation</b>\n ${esc(j?.error || "le code n'a pas été accepté")}\n\n <i>Le code expire vite — relance « Login » pour un lien frais.</i>`;
            await send(chatId, card("LOGIN", body),
              kb([[{ text: "💳 Account", callback_data: "nav:account" }, ...(ok ? [] : [{ text: "🔁 Relancer Login", callback_data: "acct:login" }])]]), thread);
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
            await brainReply(chatId, msg.message_id, thread, `Nouveau projet "${safe}" (dossier ~/Station/${category}/${safe}). Description opérateur : ${desc}. Initialise le projet en conséquence (scaffolding de base adapté à la description), puis propose un plan de démarrage concret et les prochaines étapes.`, (t) => projectOracle(safe, t), safe);
            continue;
          }
          if (p.kind === "add-project") {
            await tg("sendChatAction", { chat_id: chatId, action: "typing", message_thread_id: thread });
            await send(chatId, await addProject(text), undefined, thread);
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
            if (!/^\d+:/.test(token) || !botInfo.ok) { await send(chatId, "❌ Token invalide (vérifie le format <code>123456:ABC…</code> et que le bot existe).", kb([[{ text: "🔁 Réessayer", callback_data: `agent:tglink:${agentId}`.slice(0, 64) }]]), thread); continue; }
            // Project = the agent id if it's a known project, else the agent id itself.
            const project = (repoPath(agentId) || gitRepos().find(r => r.name.toLowerCase() === agentId.toLowerCase())) ? agentId : agentId;
            const bots = loadAgentBots();
            bots[agentId] = { token, allow: ALLOW.slice(), project };
            saveAgentBots(bots);
            try { Bun.spawnSync(["chmod", "600", AGENT_BOTS_FILE]); } catch {}
            const spawn = spawnAgentBot(agentId);
            const me = `@${botInfo.result?.username || "?"}`;
            await send(chatId, spawn === "ok"
              ? `<b>✅ Bot associé à « ${esc(agentId)} »</b>\nBot ${esc(me)} démarré, whitelisté à toi seul.\nParle-lui : tu t'adresses à l'<b>oracle ${esc(project)}</b> (scopé à ce projet — team, workers, workflows).`
              : `<b>⚠️ Bot enregistré mais service KO</b>\n${esc(spawn)}\nVérifie : <code>systemctl --user status omega-tg-agent-${esc(agentId)}</code>`,
              kb([[{ text: "🤖 Agents", callback_data: "nav:agents" }]]), thread);
            continue;
          }
        }
        if (text.startsWith("/")) {
          clearPending(from);
          const [c, ...a] = text.slice(1).split(/\s+/); const cmd = c.split("@")[0].toLowerCase();
          if (cmd === "setupgroup") await cmdSetupGroup(chat, chatId, thread);
          else if (cmd === "sync") await cmdSync(chatId, thread);
          else if (cmd === "dispatch" && a.length >= 2) { const [p, ...m] = a; await send(chatId, pre(`dispatch → ${p}`, await omega(["dispatch", p, m.join(" ")])), undefined, thread); }
          else if (KNOWN.has(cmd)) { const v = await view(cmd); await send(chatId, v.text, v.markup, thread); }
          else {
            // Unknown command → the AISB Master brain (commands gain intelligence:
            // any /verb the operator types is understood + dispatched, not dropped to the menu).
            await brainReply(chatId, msg.message_id, thread, text);
          }
        } else {
          // Free text in a project TOPIC = a MISSION → dispatch a REAL oracle session
          // (omega dispatch <project>): its own mission, visible on the VPS, it delegates
          // to dynamic workflows / workers / audit-review. Each message = a new mission.
          // Elsewhere (no topic) → ATLAS (converse / brainstorm / dispatch).
          const g = loadGroups();
          const proj = thread ? g.topics?.[String(thread)] : undefined;
          if (proj) { react(chatId, msg.message_id, "🚀"); await send(chatId, await dispatchToOracle(proj, text, chatId, thread), undefined, thread); }
          else await brainReply(chatId, msg.message_id, thread, text);
        }
      } catch (e: any) { console.error("update error:", e?.message || e); }
    }
  }
}
main();
