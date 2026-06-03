#!/usr/bin/env bun
/**
 * OmegaOS Telegram Command Bot — button-driven phone control center.
 * ───────────────────────────────────────────────────────────────────────────
 * Dependency-free (Bun + raw Telegram Bot API). Every command opens an inline
 * keyboard of sub-actions; each button runs an `omega` CLI action on the host.
 * Single poller per bot token. token/allow-list ← ~/.omega/telegram.toml.
 */
import { $ } from "bun";
import { readFileSync } from "node:fs";
import { homedir } from "node:os";

const OMEGA_DIR = process.env.OMEGA_DIR || `${homedir()}/.omega`;
const TG_TOML = `${OMEGA_DIR}/telegram.toml`;
const OMEGA = process.env.OMEGA_BIN || `${homedir()}/.local/bin/omega`;

function readToml(path: string): Record<string, string> {
  const out: Record<string, string> = {};
  try { for (const l of readFileSync(path, "utf8").split("\n")) { const m = l.match(/^\s*([a-z_]+)\s*=\s*(.+?)\s*$/i); if (m) out[m[1]] = m[2].replace(/^"|"$/g, ""); } } catch {}
  return out;
}
const cfg = readToml(TG_TOML);
const TOKEN = process.env.OMEGA_MC_TELEGRAM_TOKEN || cfg.bot_token;
if (!TOKEN) { console.error("omega-tg-bot: no bot_token in " + TG_TOML); process.exit(1); }
const API = `https://api.telegram.org/bot${TOKEN}`;
const ALLOW = (cfg.allow_user_ids?.match(/\d+/g) || []).map(Number);
const allowed = (id: number) => ALLOW.length === 0 || ALLOW.includes(id);

// ── Telegram API ─────────────────────────────────────────────────────────────
async function tg(method: string, body: any): Promise<any> {
  try { const r = await fetch(`${API}/${method}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(body) }); return await r.json(); }
  catch (e) { console.error(`tg ${method}:`, e); return { ok: false }; }
}
const MAXLEN = 3500;
type Btn = { text: string; callback_data: string };
const kb = (rows: Btn[][]) => ({ inline_keyboard: rows });
async function send(chat: number, text: string, markup?: any) {
  await tg("sendMessage", { chat_id: chat, text: text.slice(0, 4096), parse_mode: "HTML", disable_web_page_preview: true, reply_markup: markup });
}
async function edit(chat: number, msgId: number, text: string, markup?: any) {
  const r = await tg("editMessageText", { chat_id: chat, message_id: msgId, text: text.slice(0, 4096), parse_mode: "HTML", disable_web_page_preview: true, reply_markup: markup });
  if (!r.ok) await send(chat, text, markup); // e.g. "message is not modified"
}

// ── omega CLI ────────────────────────────────────────────────────────────────
async function omega(args: string[]): Promise<string> {
  try { const r = await $`${OMEGA} ${args}`.quiet().nothrow(); const o = (r.stdout.toString() + r.stderr.toString()).trim(); return o || `(no output, exit ${r.exitCode})`; }
  catch (e: any) { return `error: ${e?.message || e}`; }
}
const esc = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
const pre = (title: string, body: string) => `<b>${esc(title)}</b>\n<pre>${esc(body).slice(0, MAXLEN)}</pre>`;
const back = (to = "menu"): Btn => ({ text: "« Back", callback_data: `nav:${to}` });

// parse helpers for dynamic button lists
async function sessionNames(): Promise<string[]> {
  const out = await omega(["list"]);
  return out.split("\n").map(l => l.replace(/^[^A-Za-z0-9_-]*/, "").trim().split(/\s+/)[0]).filter(s => /^[A-Za-z0-9][\w.-]*$/.test(s));
}
async function auditIds(): Promise<string[]> {
  const out = await omega(["audit", "list"]);
  const ids: string[] = [];
  for (const l of out.split("\n")) { const m = l.match(/^\s{2,}([a-z][a-z0-9-]*audit)\b/); if (m) ids.push(m[1]); }
  return [...new Set(ids)];
}

// ── command menu (also the setMyCommands list) ───────────────────────────────
const MENU: [string, string][] = [
  ["menu", "Action hub — all commands as buttons"],
  ["status", "Live system status"],
  ["sessions", "Active sessions — Status / Kill"],
  ["projects", "Projects — list / new / add"],
  ["audits", "Quality Arsenal — tap an audit"],
  ["account", "Account / billing / accounts"],
  ["model", "AI provider + model"],
  ["skills", "Installed skills"],
  ["dispatch", "Dispatch a mission to an oracle"],
  ["killall", "Kill all sessions (keeps infra)"],
  ["clean", "Cleanup stray sessions + state"],
  ["help", "Show the action hub"],
];
function menuKb() {
  return kb([
    [{ text: "📊 Status", callback_data: "nav:status" }, { text: "🖥 Sessions", callback_data: "nav:sessions" }],
    [{ text: "📁 Projects", callback_data: "nav:projects" }, { text: "🔍 Audits", callback_data: "nav:audits" }],
    [{ text: "💳 Account", callback_data: "nav:account" }, { text: "🧠 Model", callback_data: "nav:model" }],
    [{ text: "🧩 Skills", callback_data: "nav:skills" }, { text: "🚀 Dispatch", callback_data: "nav:dispatch" }],
    [{ text: "🧹 Clean", callback_data: "nav:clean" }, { text: "🛑 Kill all", callback_data: "nav:killall" }],
  ]);
}
const menuText = "<b>Ω OmegaOS — action hub</b>\nTap an action. Each runs on your server via the <code>omega</code> CLI.";

// ── render a "view" for a command (text + buttons). Used by both /cmd and nav. ─
async function view(name: string): Promise<{ text: string; markup: any }> {
  switch (name) {
    case "menu": case "help":
      return { text: menuText, markup: menuKb() };
    case "status":
      return { text: pre("System status", await omega(["doctor"])), markup: kb([[{ text: "🔄 Refresh", callback_data: "nav:status" }, back()]]) };
    case "sessions": {
      const names = await sessionNames();
      const rows = names.slice(0, 12).map(s => [{ text: `📊 ${s}`.slice(0, 30), callback_data: `sess:status:${s}`.slice(0, 64) }, { text: "🛑 Kill", callback_data: `sess:kill:${s}`.slice(0, 64) }]);
      return { text: pre("Active sessions", names.length ? names.join("\n") : "(none)"), markup: kb([...rows, [{ text: "🔄 Refresh", callback_data: "nav:sessions" }, back()]]) };
    }
    case "projects":
      return { text: pre("Projects", await omega(["projects"])), markup: kb([[{ text: "📋 List", callback_data: "proj:list" }], [{ text: "➕ New project", callback_data: "proj:new" }, { text: "📁 Add existing", callback_data: "proj:add" }], [back()]]) };
    case "audits": {
      const ids = await auditIds();
      const rows: Btn[][] = [];
      for (let i = 0; i < Math.min(ids.length, 23); i += 2) rows.push(ids.slice(i, i + 2).map(a => ({ text: a.slice(0, 28), callback_data: `aud:run:${a}`.slice(0, 64) })));
      return { text: `<b>Quality Arsenal</b>\nTap an audit to see how to run it (${ids.length} available).`, markup: kb([...rows, [back()]]) };
    }
    case "account":
      return { text: pre("Account / billing", await omega(["monitor"])), markup: kb([[{ text: "💳 Billing", callback_data: "acct:billing" }, { text: "👤 Accounts", callback_data: "acct:accounts" }], [{ text: "🔄 Refresh", callback_data: "nav:account" }, back()]]) };
    case "model":
      return { text: pre("Model / providers", await omega(["config", "show"])), markup: kb([[{ text: "🔄 Refresh", callback_data: "nav:model" }, back()]]) };
    case "skills":
      return { text: pre("Skills", Bun.spawnSync(["ls", "-1", `${OMEGA_DIR}/skills`]).stdout.toString().trim() || "(none)"), markup: kb([[back()]]) };
    case "dispatch":
      return { text: "<b>🚀 Dispatch</b>\nSend: <code>/dispatch &lt;project&gt; &lt;mission&gt;</code>\nThe mission goes to that project's oracle.", markup: kb([[{ text: "📁 See projects", callback_data: "nav:projects" }], [back()]]) };
    case "killall":
      return { text: "<b>🛑 Kill all sessions?</b>\nKeeps your infra (Home/System, bridge, master).", markup: kb([[{ text: "✅ Yes, kill all", callback_data: "do:killall" }], [{ text: "✖ Cancel", callback_data: "nav:menu" }]]) };
    case "clean":
      return { text: "<b>🧹 Cleanup?</b>\nKill stray sessions + prune stale state. Never touches current/infra.", markup: kb([[{ text: "✅ Yes, cleanup", callback_data: "do:clean" }], [{ text: "✖ Cancel", callback_data: "nav:menu" }]]) };
    default:
      return { text: menuText, markup: menuKb() };
  }
}

// ── callback (button tap) handler ────────────────────────────────────────────
async function onCallback(data: string, chat: number, msgId: number) {
  const [ns, action, ...rest] = data.split(":");
  const arg = rest.join(":");
  if (ns === "nav") { const v = await view(action); return edit(chat, msgId, v.text, v.markup); }
  if (ns === "sess" && action === "status") return edit(chat, msgId, pre(`Session ${arg}`, await omega(["capture", arg])), kb([[{ text: "🔄 Refresh", callback_data: `sess:status:${arg}`.slice(0, 64) }, back("sessions")]]));
  if (ns === "sess" && action === "kill")   { const o = await omega(["kill", arg]); return edit(chat, msgId, pre(`Kill ${arg}`, o), kb([[back("sessions")]])); }
  if (ns === "proj" && action === "list")   return edit(chat, msgId, pre("Projects", await omega(["projects"])), kb([[back("projects")]]));
  if (ns === "proj" && action === "new")    return edit(chat, msgId, "<b>➕ New project</b>\nRun on the server: <code>omega new-project</code> (guided). From the phone, send: <code>/dispatch &lt;name&gt; &lt;goal&gt;</code> once it exists.", kb([[back("projects")]]));
  if (ns === "proj" && action === "add")    return edit(chat, msgId, "<b>📁 Add existing</b>\nClone/move the repo under your projects root; <code>omega projects</code> auto-discovers it.", kb([[back("projects")]]));
  if (ns === "aud" && action === "run")     return edit(chat, msgId, pre(`Audit: ${arg}`, await omega(["audit", "run", arg])), kb([[back("audits")]]));
  if (ns === "acct" && action === "billing") return edit(chat, msgId, pre("Billing", await omega(["monitor"])), kb([[back("account")]]));
  if (ns === "acct" && action === "accounts") return edit(chat, msgId, pre("Accounts", await omega(["provision", "groups"])), kb([[back("account")]]));
  if (ns === "do" && action === "killall")  return edit(chat, msgId, pre("kill-all", await omega(["kill-all"])), kb([[back("menu")]]));
  if (ns === "do" && action === "clean")    return edit(chat, msgId, pre("cleanup", await omega(["cleanup"])), kb([[back("menu")]]));
  return edit(chat, msgId, menuText, menuKb());
}

// ── poll loop ────────────────────────────────────────────────────────────────
async function main() {
  await tg("setMyCommands", { commands: MENU.map(([command, description]) => ({ command, description })) });
  await tg("deleteWebhook", { drop_pending_updates: false });
  console.log(`omega-tg-bot v2 (buttons) up. commands=${MENU.length} allow=${ALLOW.join(",") || "ALL"}`);
  let offset = 0;
  while (true) {
    const r = await tg("getUpdates", { offset, timeout: 50, allowed_updates: ["message", "callback_query"] });
    if (!r.ok) { await Bun.sleep(2000); continue; }
    for (const u of r.result) {
      offset = u.update_id + 1;
      try {
        if (u.callback_query) {
          const q = u.callback_query, from = q.from?.id ?? 0;
          await tg("answerCallbackQuery", { callback_query_id: q.id });
          if (!allowed(from)) continue;
          await onCallback(q.data || "", q.message.chat.id, q.message.message_id);
          continue;
        }
        const msg = u.message; if (!msg?.text) continue;
        const chat = msg.chat.id, from = msg.from?.id ?? 0, text = msg.text.trim();
        if (!allowed(from)) { console.log(`drop from ${from}`); continue; }
        if (text.startsWith("/")) {
          const [c, ...a] = text.slice(1).split(/\s+/);
          const cmd = c.split("@")[0].toLowerCase();
          if (cmd === "dispatch" && a.length >= 2) { const [p, ...m] = a; await send(chat, pre(`dispatch → ${p}`, await omega(["dispatch", p, m.join(" ")]))); }
          else { const v = await view(cmd); await send(chat, v.text, v.markup); }
        } else {
          await omega(["send", "aisb-master", text]);
          await send(chat, "🧠 Sent to the AISB Master. Use /menu for actions.");
        }
      } catch (e: any) { console.error("update error:", e?.message || e); }
    }
  }
}
main();
