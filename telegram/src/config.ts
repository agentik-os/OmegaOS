/**
 * Configuration for Claude Telegram Bot.
 *
 * All environment variables, paths, constants, and safety settings.
 */

import { homedir } from "os";
import { resolve, dirname } from "path";
import { existsSync, readFileSync } from "fs";
import type { McpServerConfig } from "./types";

// ============== Environment Setup ==============

const HOME = homedir();

// Ensure necessary paths are available for Claude's bash commands
// LaunchAgents don't inherit the full shell environment
const EXTRA_PATHS = [
  `${HOME}/.local/bin`,
  `${HOME}/.bun/bin`,
  "/opt/homebrew/bin",
  "/opt/homebrew/sbin",
  "/usr/local/bin",
];

const currentPath = process.env.PATH || "";
const pathParts = currentPath.split(":");
for (const extraPath of EXTRA_PATHS) {
  if (!pathParts.includes(extraPath)) {
    pathParts.unshift(extraPath);
  }
}
process.env.PATH = pathParts.join(":");

// ============== Core Configuration ==============

export const TELEGRAM_TOKEN = process.env.TELEGRAM_BOT_TOKEN || "";
export const ALLOWED_USERS: number[] = (
  process.env.TELEGRAM_ALLOWED_USERS || ""
)
  .split(",")
  .filter((x) => x.trim())
  .map((x) => parseInt(x.trim(), 10))
  .filter((x) => !isNaN(x));

export const WORKING_DIR = process.env.CLAUDE_WORKING_DIR || HOME;
export const OPENAI_API_KEY = process.env.OPENAI_API_KEY || "";

// ============== Claude CLI Path ==============

// Auto-detect via the canonical chain shared with omega-tg-bot.ts resolveClaude
// (mirrors omega-core agents.rs claude_available): env override first (legacy
// CLAUDE_CLI_PATH kept for backward compat, then CLAUDE_BIN; bare names resolve
// on PATH via Bun.which), then ~/.local/bin/claude, PATH, ~/.claude/local/claude,
// ~/.npm-global/bin/claude.
function findClaudeCli(): string {
  const fromEnv = (v: string | undefined) =>
    v ? (v.includes("/") ? v : Bun.which(v)) : null;
  const candidates = [
    fromEnv(process.env.CLAUDE_CLI_PATH),
    fromEnv(process.env.CLAUDE_BIN),
    `${HOME}/.local/bin/claude`,
    Bun.which("claude"),
    `${HOME}/.claude/local/claude`,
    `${HOME}/.npm-global/bin/claude`,
  ];
  for (const c of candidates) if (c && existsSync(c)) return c;

  // Final fallback (legacy default — the spawn error surfaces if it's absent)
  return "/usr/local/bin/claude";
}

export const CLAUDE_CLI_PATH = findClaudeCli();

// ============== MCP Configuration ==============

// MCP servers loaded from mcp-config.ts
let MCP_SERVERS: Record<string, McpServerConfig> = {};

try {
  // Dynamic import of MCP config
  const mcpConfigPath = resolve(dirname(import.meta.dir), "mcp-config.ts");
  const mcpModule = await import(mcpConfigPath).catch(() => null);
  if (mcpModule?.MCP_SERVERS) {
    MCP_SERVERS = mcpModule.MCP_SERVERS;
    console.log(
      `Loaded ${Object.keys(MCP_SERVERS).length} MCP servers from mcp-config.ts`
    );
  }
} catch {
  console.log("No mcp-config.ts found - running without MCPs");
}

export { MCP_SERVERS };

// ============== Security Configuration ==============

// Allowed directories for file operations. The AISB Master orchestrates across
// ALL projects (it dispatches to per-project oracles and reads anywhere under
// home), so the default scope is the whole home dir. The destructive-action
// confirmation rule in the safety prompt still applies. Narrow via ALLOWED_PATHS.
const defaultAllowedPaths = [HOME];

const allowedPathsStr = process.env.ALLOWED_PATHS || "";
export const ALLOWED_PATHS: string[] = allowedPathsStr
  ? allowedPathsStr
      .split(",")
      .map((p) => p.trim())
      .filter(Boolean)
  : defaultAllowedPaths;

// Build safety prompt dynamically from ALLOWED_PATHS
function buildSafetyPrompt(allowedPaths: string[]): string {
  const pathsList = allowedPaths
    .map((p) => `   - ${p} (and subdirectories)`)
    .join("\n");

  return `
CRITICAL SAFETY RULES FOR TELEGRAM BOT:

1. NEVER delete, remove, or overwrite files without EXPLICIT confirmation from the user.
   - If user asks to delete something, respond: "Are you sure you want to delete [file]? Reply 'yes delete it' to confirm."
   - Only proceed with deletion if user replies with explicit confirmation like "yes delete it", "confirm delete"
   - This applies to: rm, trash, unlink, shred, or any file deletion

2. You can ONLY access files in these directories:
${pathsList}
   - REFUSE any file operations outside these paths

3. NEVER run dangerous commands like:
   - rm -rf (recursive force delete)
   - Any command that affects files outside allowed directories
   - Commands that could damage the system

4. For any destructive or irreversible action, ALWAYS ask for confirmation first.

You are running via Telegram, so the user cannot easily undo mistakes. Be extra careful!
`;
}

export const SAFETY_PROMPT = buildSafetyPrompt(ALLOWED_PATHS);

// ============== AISB Master system prompt (OmegaOS) ==============
// This Telegram session IS the AISB Master — the orchestrator brain of OmegaOS.
// It does NOT do project work itself; it classifies the request and DISPATCHES
// to the right per-project oracle via the `omega` CLI (on PATH at ~/.local/bin).
// The 13 agents live here, in the system prompt (no external dashboard/config).
const OMEGA_MASTER_PROMPT = `You are AISB MASTER — the orchestrator brain of OmegaOS, reached over Telegram.
You run from $HOME with full system scope. The \`omega\` CLI is on your PATH.

YOUR ROLE (router, not worker):
- Classify each request: SIMPLE (answer / one quick command yourself) → MEDIUM/COMPLEX/EPIC (DISPATCH to a project oracle).
- You do NOT edit a project's code yourself. You decide WHERE the work goes and dispatch it, then report back.

THE 13 AISB AGENTS (Matrix-named) you command:
- ORACLE — one per project; strategic: classifies, plans, dispatches workers, runs the quality gate.
- MORPHEUS — executes implementation work (the builder).
- SERAPH — guards the quality gate (adversarial verification, audits).
- KEYMAKER — provisions keys/credentials for every provider.
- SMITH — learns from outcomes, proposes improvements.
- NEO — monitors sessions/resources/health.
- NIOBE — steers the fleet (multi-session coordination).
- LINK — keeps the operator console / comms online.
- ZION — dashboard + persisted state.
- ARCHITECT — system/ecosystem design.
- CONSTRUCT — builds resources/scaffolds on request.
- MEROVINGIAN — indexes/dedupes knowledge.
- PYTHIA — watches platform docs for changes.

HOW TO DISPATCH (run via your Bash tool):
- Dispatch a mission to a project's oracle:   omega dispatch <project> "<mission>"
- Full end-to-end orchestrated mission:        omega orchestrate "<mission>"
- List live sessions / agents:                 omega list
- Classify a mission's complexity:             omega route "<mission>"
- Check an oracle's history:                   omega timeline <oracle>
- Spawn a team in split panes:                 omega team ...
Always prefer the omega CLI for orchestration; report what you dispatched and where.

When the user just wants an answer or a quick fact, answer directly — don't over-orchestrate.
Be concise (this is a phone chat). Lead with the answer.`;

// If OmegaOS shipped a composed Master prompt (aisb-master.md = static prompt +
// the live Laws/Rules from the typed registry), PREPEND it as the single source
// of truth; otherwise use the built-in prompt above. Then append linuz90's
// path-safety rules. session.ts uses SYSTEM_PROMPT as the SDK systemPrompt.
function loadOmegaMasterPrompt(): string {
  const candidates = [
    process.env.OMEGA_SYSTEM_PROMPT_FILE || "",
    `${HOME}/.omega/agents/_master-runtime.md`,
    `${HOME}/.omega/agents/aisb-master.md`,
  ].filter(Boolean);
  for (const f of candidates) {
    try {
      if (existsSync(f)) {
        const txt = readFileSync(f, "utf8").trim();
        if (txt) return txt + "\n\n" + OMEGA_MASTER_PROMPT;
      }
    } catch {
      /* fall through to built-in */
    }
  }
  return OMEGA_MASTER_PROMPT;
}

export const SYSTEM_PROMPT = loadOmegaMasterPrompt() + "\n\n" + SAFETY_PROMPT;

// Dangerous command patterns to block
export const BLOCKED_PATTERNS = [
  "rm -rf /",
  "rm -rf ~",
  "rm -rf $HOME",
  "sudo rm",
  ":(){ :|:& };:", // Fork bomb
  "> /dev/sd",
  "mkfs.",
  "dd if=",
];

// Query timeout (3 minutes)
export const QUERY_TIMEOUT_MS = 180_000;

// ============== Voice Transcription ==============

const BASE_TRANSCRIPTION_PROMPT = `Transcribe this voice message accurately.
The speaker may use multiple languages (English, and possibly others).
Focus on accuracy for proper nouns, technical terms, and commands.`;

let TRANSCRIPTION_CONTEXT = "";
if (process.env.TRANSCRIPTION_CONTEXT_FILE) {
  try {
    const file = Bun.file(process.env.TRANSCRIPTION_CONTEXT_FILE);
    if (await file.exists()) {
      TRANSCRIPTION_CONTEXT = (await file.text()).trim();
    }
  } catch {
    // File not found or unreadable — proceed without context
  }
}

export const TRANSCRIPTION_PROMPT = TRANSCRIPTION_CONTEXT
  ? `${BASE_TRANSCRIPTION_PROMPT}\n\nAdditional context:\n${TRANSCRIPTION_CONTEXT}`
  : BASE_TRANSCRIPTION_PROMPT;

export const TRANSCRIPTION_AVAILABLE = !!OPENAI_API_KEY;

// ============== Thinking Keywords ==============

const thinkingKeywordsStr =
  process.env.THINKING_KEYWORDS || "think,pensa,ragiona";
const thinkingDeepKeywordsStr =
  process.env.THINKING_DEEP_KEYWORDS || "ultrathink,think hard,pensa bene";

export const THINKING_KEYWORDS = thinkingKeywordsStr
  .split(",")
  .map((k) => k.trim().toLowerCase());
export const THINKING_DEEP_KEYWORDS = thinkingDeepKeywordsStr
  .split(",")
  .map((k) => k.trim().toLowerCase());

// ============== Media Group Settings ==============

export const MEDIA_GROUP_TIMEOUT = 1000; // ms to wait for more photos in a group

// ============== Telegram Message Limits ==============

export const TELEGRAM_MESSAGE_LIMIT = 4096; // Max characters per message
export const TELEGRAM_SAFE_LIMIT = 4000; // Safe limit with buffer for formatting
export const STREAMING_THROTTLE_MS = 500; // Throttle streaming updates
export const BUTTON_LABEL_MAX_LENGTH = 30; // Max chars for inline button labels

// ============== Audit Logging ==============

export const AUDIT_LOG_PATH =
  process.env.AUDIT_LOG_PATH || "/tmp/claude-telegram-audit.log";
export const AUDIT_LOG_JSON =
  (process.env.AUDIT_LOG_JSON || "false").toLowerCase() === "true";

// ============== Rate Limiting ==============

export const RATE_LIMIT_ENABLED =
  (process.env.RATE_LIMIT_ENABLED || "true").toLowerCase() === "true";
export const RATE_LIMIT_REQUESTS = parseInt(
  process.env.RATE_LIMIT_REQUESTS || "20",
  10
);
export const RATE_LIMIT_WINDOW = parseInt(
  process.env.RATE_LIMIT_WINDOW || "60",
  10
);

// ============== File Paths ==============

export const SESSION_FILE = "/tmp/claude-telegram-session.json";
export const RESTART_FILE = "/tmp/claude-telegram-restart.json";
export const TEMP_DIR = "/tmp/telegram-bot";

// Temp paths that are always allowed for bot operations
export const TEMP_PATHS = ["/tmp/", "/private/tmp/", "/var/folders/"];

// Ensure temp directory exists
await Bun.write(`${TEMP_DIR}/.keep`, "");

// ============== Validation ==============

if (!TELEGRAM_TOKEN) {
  console.error("ERROR: TELEGRAM_BOT_TOKEN environment variable is required");
  process.exit(1);
}

if (ALLOWED_USERS.length === 0) {
  console.error(
    "ERROR: TELEGRAM_ALLOWED_USERS environment variable is required"
  );
  process.exit(1);
}

console.log(
  `Config loaded: ${ALLOWED_USERS.length} allowed users, working dir: ${WORKING_DIR}`
);
