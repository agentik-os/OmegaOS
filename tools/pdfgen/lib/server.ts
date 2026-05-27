import { spawn, ChildProcess } from "node:child_process";
import http from "node:http";

const PORT = Number(process.env.AGENTIK_PDF_PORT || 4317);
const BASE = `http://127.0.0.1:${PORT}`;

function waitForReady(url: string, timeoutMs = 60_000): Promise<void> {
  return new Promise((resolve, reject) => {
    const start = Date.now();
    const tick = () => {
      const req = http.get(url, (res) => {
        res.resume();
        if (res.statusCode && res.statusCode < 500) return resolve();
        retry();
      });
      req.on("error", retry);
      req.setTimeout(2000, () => req.destroy());
    };
    const retry = () => {
      if (Date.now() - start > timeoutMs) {
        return reject(new Error(`server not ready after ${timeoutMs}ms`));
      }
      setTimeout(tick, 400);
    };
    tick();
  });
}

let proc: ChildProcess | null = null;

export async function startServer(cwd: string): Promise<string> {
  // Reuse existing server if reachable
  try {
    await waitForReady(BASE, 1500);
    return BASE;
  } catch {
    // Start one
  }
  proc = spawn("npx", ["next", "start", "--port", String(PORT)], {
    cwd,
    env: { ...process.env, NODE_ENV: "production" },
    stdio: ["ignore", "pipe", "pipe"],
    detached: true
  });
  proc.stdout?.on("data", (d) => process.stderr.write(`[next] ${d}`));
  proc.stderr?.on("data", (d) => process.stderr.write(`[next] ${d}`));
  await waitForReady(BASE, 60_000);
  return BASE;
}

export async function stopServer(): Promise<void> {
  if (proc && !proc.killed) {
    try {
      // Kill the entire process group to catch Next's child processes too
      process.kill(-proc.pid!, "SIGTERM");
    } catch {
      proc.kill("SIGTERM");
    }
    await new Promise((r) => setTimeout(r, 400));
    if (!proc.killed) {
      try {
        process.kill(-proc.pid!, "SIGKILL");
      } catch {
        proc.kill("SIGKILL");
      }
    }
  }
  proc = null;
}
