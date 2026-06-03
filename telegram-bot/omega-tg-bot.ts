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
const cfg = readKV(TG_TOML, /^\s*([a-z_]+)\s*=\s*(.+?)\s*$/i);
const TOKEN = process.env.OMEGA_MC_TELEGRAM_TOKEN || cfg.bot_token;
if (!TOKEN) { console.error("omega-tg-bot: no bot_token in " + TG_TOML); process.exit(1); }
const API = `https://api.telegram.org/bot${TOKEN}`;
const BOT_ID = Number(TOKEN.split(":")[0]);
const ALLOW = (cfg.allow_user_ids?.match(/\d+/g) || []).map(Number);
const allowed = (id: number) => ALLOW.length === 0 || ALLOW.includes(id);

// group/topic registry (persisted)
type Groups = { hub?: number; isForum?: boolean; topics?: Record<string, string> };
function loadGroups(): Groups { try { return JSON.parse(readFileSync(GROUPS_FILE, "utf8")); } catch { return {}; } }
function saveGroups(g: Groups) { try { writeFileSync(GROUPS_FILE, JSON.stringify(g, null, 2)); } catch {} }

// ── Telegram API ─────────────────────────────────────────────────────────────
async function tg(method: string, body: any): Promise<any> {
  try { const r = await fetch(`${API}/${method}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(body) }); return await r.json(); }
  catch (e) { console.error(`tg ${method}:`, e); return { ok: false }; }
}
const MAXLEN = 3500;
type Btn = { text: string; callback_data?: string; url?: string };
const kb = (rows: Btn[][]) => ({ inline_keyboard: rows });
async function send(chat: number, text: string, markup?: any, thread?: number) {
  await tg("sendMessage", { chat_id: chat, text: text.slice(0, 4096), parse_mode: "HTML", disable_web_page_preview: true, reply_markup: markup, message_thread_id: thread });
}
async function edit(chat: number, msgId: number, text: string, markup?: any) {
  const r = await tg("editMessageText", { chat_id: chat, message_id: msgId, text: text.slice(0, 4096), parse_mode: "HTML", disable_web_page_preview: true, reply_markup: markup });
  if (!r.ok) await send(chat, text, markup);
}

// ── omega CLI ────────────────────────────────────────────────────────────────
async function omega(args: string[]): Promise<string> {
  try { const r = await $`${OMEGA} ${args}`.quiet().nothrow(); const o = (r.stdout.toString() + r.stderr.toString()).trim(); return o || `(no output, exit ${r.exitCode})`; }
  catch (e: any) { return `error: ${e?.message || e}`; }
}
const esc = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
const pre = (title: string, body: string) => `<b>${esc(title)}</b>\n<pre>${esc(body).slice(0, MAXLEN)}</pre>`;
const back = (to = "menu"): Btn => ({ text: "« Back", callback_data: `nav:${to}` });

function dashboardURL(): { url: string; pw: string } {
  const mc = readKV(MC_ENV, /^([A-Z_]+)=(.*)$/);
  const host = mc.HOSTNAME?.trim();
  const url = host ? `https://${host}` : `http://${process.env.OMEGA_PUBLIC_IP || "<server-ip>"}:8080`;
  return { url, pw: mc.OMEGA_MC_WEB_PASSWORD || "" };
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
const menuText = "<b>Ω OmegaOS — action hub</b>\nTap an action. Each runs on your server via the <code>omega</code> CLI.";

// ── views ────────────────────────────────────────────────────────────────────
async function view(name: string): Promise<{ text: string; markup: any }> {
  switch (name) {
    case "menu": case "help": case "commands": return { text: menuText, markup: menuKb() };
    case "agents": {
      const ags = await mcAgents();
      if (!ags.length) return { text: "<b>🤖 AISB agents</b>\nDashboard not reachable. Bring it up: <code>omega-mc-up</code>.", markup: kb([[back()]]) };
      const rows: Btn[][] = [];
      for (let i = 0; i < ags.length; i += 2) rows.push(ags.slice(i, i + 2).map(a => ({ text: a.id.slice(0, 28), callback_data: `agent:info:${a.id}`.slice(0, 64) })));
      return { text: `<b>🤖 AISB agents (${ags.length})</b>\nTap one for its role. To chat with an agent, use the agents bot (OmegaMC) — see /dashboard.`, markup: kb([...rows, [back()]]) };
    }
    case "dashboard": {
      const { url, pw } = dashboardURL();
      const t = `<b>🖥 Mission Control</b>\nOpen: ${url}\n${pw ? `Password: <code>${esc(pw)}</code>\n` : ""}${url.includes("<server-ip>") || url.startsWith("http://") ? "\n⚠️ For secure external access, enable Tailscale (no exposed port)." : ""}`;
      return { text: t, markup: kb([[{ text: "🌐 Open dashboard", url: url.replace("<server-ip>", process.env.OMEGA_PUBLIC_IP || "") }], [back()]]) };
    }
    case "status": return { text: pre("System status", await omega(["doctor"])), markup: kb([[{ text: "🔄 Refresh", callback_data: "nav:status" }, back()]]) };
    case "sessions": {
      const names = await sessionNames();
      const rows = names.slice(0, 12).map(s => [{ text: `📊 ${s}`.slice(0, 30), callback_data: `sess:status:${s}`.slice(0, 64) }, { text: "🛑 Kill", callback_data: `sess:kill:${s}`.slice(0, 64) }]);
      return { text: pre("Active sessions", names.join("\n") || "(none)"), markup: kb([...rows, [{ text: "🔄 Refresh", callback_data: "nav:sessions" }, back()]]) };
    }
    case "projects": return { text: pre("Projects", await omega(["projects"])), markup: kb([[{ text: "📋 List", callback_data: "proj:list" }], [{ text: "➕ New", callback_data: "proj:new" }, { text: "📁 Add existing", callback_data: "proj:add" }], [{ text: "🔁 Sync topics", callback_data: "nav:sync" }], [back()]]) };
    case "audits": {
      const ids = await auditIds(); const rows: Btn[][] = [];
      for (let i = 0; i < ids.length; i += 2) rows.push(ids.slice(i, i + 2).map(a => ({ text: a.slice(0, 28), callback_data: `aud:run:${a}`.slice(0, 64) })));
      return { text: `<b>Quality Arsenal</b>\nTap an audit (${ids.length} available).`, markup: kb([...rows, [back()]]) };
    }
    case "account": return { text: pre("Account / billing", await omega(["monitor"])), markup: kb([[{ text: "💳 Billing", callback_data: "acct:billing" }, { text: "👤 Accounts", callback_data: "acct:accounts" }], [{ text: "🔄 Refresh", callback_data: "nav:account" }, back()]]) };
    case "model": return { text: pre("Model / providers", await omega(["config", "show"])), markup: kb([[{ text: "🔄 Refresh", callback_data: "nav:model" }, back()]]) };
    case "skills": return { text: pre("Skills", Bun.spawnSync(["ls", "-1", `${OMEGA_DIR}/skills`]).stdout.toString().trim() || "(none)"), markup: kb([[back()]]) };
    case "dispatch": return { text: "<b>🚀 Dispatch</b>\nSend: <code>/dispatch &lt;project&gt; &lt;mission&gt;</code>", markup: kb([[{ text: "📁 Projects", callback_data: "nav:projects" }], [back()]]) };
    case "setupgroup": return { text: "<b>👥 Group hub</b>\nRun <code>/setupgroup</code> <b>inside a supergroup</b> where this bot is an <b>admin</b> (and topics are enabled). It registers the group as your project hub, then <code>/sync</code> maps each project to a topic.", markup: kb([[back()]]) };
    case "sync": { const g = loadGroups(); return { text: g.hub ? `<b>🔁 Sync</b>\nHub group registered. Run <code>/sync</code> inside it to map projects → topics.` : "<b>🔁 Sync</b>\nNo hub group yet — run <code>/setupgroup</code> in your supergroup first.", markup: kb([[back()]]) }; }
    case "killall": return { text: "<b>🛑 Kill all sessions?</b>\nKeeps infra (Home/System, bridge, master).", markup: kb([[{ text: "✅ Yes", callback_data: "do:killall" }], [{ text: "✖ Cancel", callback_data: "nav:menu" }]]) };
    case "clean": return { text: "<b>🧹 Cleanup?</b>\nKill stray sessions + prune state. Never touches infra.", markup: kb([[{ text: "✅ Yes", callback_data: "do:clean" }], [{ text: "✖ Cancel", callback_data: "nav:menu" }]]) };
    default: return { text: menuText, markup: menuKb() };
  }
}

async function onCallback(data: string, chat: number, msgId: number) {
  const [ns, action, ...rest] = data.split(":"); const arg = rest.join(":");
  if (ns === "nav") { const v = await view(action); return edit(chat, msgId, v.text, v.markup); }
  if (ns === "sess" && action === "status") return edit(chat, msgId, pre(`Session ${arg}`, await omega(["capture", arg])), kb([[{ text: "🔄 Refresh", callback_data: `sess:status:${arg}`.slice(0, 64) }, back("sessions")]]));
  if (ns === "sess" && action === "kill") return edit(chat, msgId, pre(`Kill ${arg}`, await omega(["kill", arg])), kb([[back("sessions")]]));
  if (ns === "proj" && action === "list") return edit(chat, msgId, pre("Projects", await omega(["projects"])), kb([[back("projects")]]));
  if (ns === "proj" && action === "new") return edit(chat, msgId, "<b>➕ New project</b>\nOn the server: <code>omega new-project</code> (guided).", kb([[back("projects")]]));
  if (ns === "proj" && action === "add") return edit(chat, msgId, "<b>📁 Add existing</b>\nClone/move the repo under your projects root; <code>omega projects</code> auto-discovers it.", kb([[back("projects")]]));
  if (ns === "aud" && action === "run") return edit(chat, msgId, pre(`Audit: ${arg}`, await omega(["audit", "run", arg])), kb([[back("audits")]]));
  if (ns === "acct" && action === "billing") return edit(chat, msgId, pre("Billing", await omega(["monitor"])), kb([[back("account")]]));
  if (ns === "acct" && action === "accounts") return edit(chat, msgId, pre("Accounts", await omega(["provision", "groups"])), kb([[back("account")]]));
  if (ns === "do" && action === "killall") return edit(chat, msgId, pre("kill-all", await omega(["kill-all"])), kb([[back("menu")]]));
  if (ns === "do" && action === "clean") return edit(chat, msgId, pre("cleanup", await omega(["cleanup"])), kb([[back("menu")]]));
  if (ns === "agent" && action === "info") { const a = (await mcAgents()).find(x => x.id === arg); return edit(chat, msgId, `<b>🤖 ${esc(arg)}</b>\n${esc(a?.description || "(no description)")}\n\n<i>To chat with this agent, use the OmegaMC agents bot.</i>`, kb([[back("agents")]])); }
  return edit(chat, msgId, menuText, menuKb());
}

// ── group setup: verify the bot is admin, register the supergroup as hub ─────
async function cmdSetupGroup(chat: any, chatId: number, thread?: number) {
  if (chat.type !== "group" && chat.type !== "supergroup") return send(chatId, "⚠️ Run /setupgroup <b>inside a Telegram group</b> (a supergroup with Topics enabled).", undefined, thread);
  const admins = await tg("getChatAdministrators", { chat_id: chatId });
  const isAdmin = admins.ok && admins.result.some((a: any) => a.user?.id === BOT_ID);
  if (!isAdmin) return send(chatId, "⚠️ I'm <b>not an admin</b> here. Add me as admin (with <i>Manage Topics</i>), then run /setupgroup again.", undefined, thread);
  const g = loadGroups(); g.hub = chatId; g.isForum = !!chat.is_forum; g.topics ||= {}; saveGroups(g);
  return send(chatId, `✅ Registered this group as the <b>project hub</b>.${chat.is_forum ? "\nTopics are enabled — run <code>/sync</code> to map each project to its own topic." : "\n⚠️ Topics are NOT enabled. Turn on <i>Topics</i> in group settings, then /sync."}`, undefined, thread);
}

// ── sync: one forum topic per project; route topic messages to its oracle ────
async function cmdSync(chatId: number, thread?: number) {
  const g = loadGroups();
  if (!g.hub) return send(chatId, "No hub group yet — run /setupgroup in your supergroup first.", undefined, thread);
  if (!g.isForum) return send(chatId, "This group has no Topics enabled — turn on Topics, re-run /setupgroup, then /sync.", undefined, thread);
  const projects = await projectNames();
  if (!projects.length) return send(g.hub, "No projects discovered yet. Create one (<code>omega new-project</code>) or add a repo under your projects root, then /sync.", undefined, thread);
  g.topics ||= {}; const existing = new Set(Object.values(g.topics)); let made = 0;
  for (const p of projects) {
    if (existing.has(p)) continue;
    const r = await tg("createForumTopic", { chat_id: g.hub, name: p.slice(0, 128) });
    if (r.ok) { g.topics[String(r.result.message_thread_id)] = p; made++; }
  }
  saveGroups(g);
  return send(g.hub, `🔁 Sync done. ${made} new topic(s) created; ${Object.keys(g.topics).length} project topic(s) total. Messages in a project's topic now dispatch to that project's oracle.`, undefined, thread);
}

// ── poll loop ────────────────────────────────────────────────────────────────
async function main() {
  // Register the menu on BOTH default and all_private_chats scopes — some Telegram
  // clients read the private-chat scope preferentially, so setting only default
  // can leave a stale/empty menu in DMs.
  const cmds = MENU.map(([command, description]) => ({ command, description }));
  await tg("setMyCommands", { commands: cmds });
  await tg("setMyCommands", { commands: cmds, scope: { type: "all_private_chats" } });
  await tg("deleteWebhook", { drop_pending_updates: false });
  try { process.env.OMEGA_PUBLIC_IP ||= (await (await fetch("https://ifconfig.me")).text()).trim(); } catch {}
  console.log(`omega-tg-bot v3 up. botId=${BOT_ID} commands=${MENU.length} allow=${ALLOW.join(",") || "ALL"}`);
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
          await onCallback(q.data || "", q.message.chat.id, q.message.message_id); continue;
        }
        const msg = u.message; if (!msg?.text) continue;
        const chat = msg.chat, chatId = chat.id, from = msg.from?.id ?? 0, text = msg.text.trim();
        const thread = msg.message_thread_id;
        if (!allowed(from)) { console.log(`drop from ${from}`); continue; }
        if (text.startsWith("/")) {
          const [c, ...a] = text.slice(1).split(/\s+/); const cmd = c.split("@")[0].toLowerCase();
          if (cmd === "setupgroup") await cmdSetupGroup(chat, chatId, thread);
          else if (cmd === "sync") await cmdSync(chatId, thread);
          else if (cmd === "dispatch" && a.length >= 2) { const [p, ...m] = a; await send(chatId, pre(`dispatch → ${p}`, await omega(["dispatch", p, m.join(" ")])), undefined, thread); }
          else { const v = await view(cmd); await send(chatId, v.text, v.markup, thread); }
        } else {
          // In a project topic → dispatch to that project's oracle; else → AISB Master.
          const g = loadGroups();
          if (thread && g.topics?.[String(thread)]) {
            const proj = g.topics[String(thread)];
            await send(chatId, pre(`dispatch → ${proj}`, await omega(["dispatch", proj, text])), undefined, thread);
          } else {
            await omega(["send", "aisb-master", text]);
            await send(chatId, "🧠 Sent to the AISB Master. Use /menu for actions.", undefined, thread);
          }
        }
      } catch (e: any) { console.error("update error:", e?.message || e); }
    }
  }
}
main();
