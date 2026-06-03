#!/usr/bin/env node
'use strict';
/*
 * omega-os — one-command installer for OmegaOS.
 *   npx omega-os            install into ./OmegaOS (or $HOME/Station/OmegaOS if ~/Station exists)
 *   npx omega-os --dir DIR  install into DIR
 *   npx omega-os --help
 *
 * Zero runtime deps (Node builtins only) so npx is fast + robust. Clones the
 * PUBLIC repo agentik-os/OmegaOS and runs its install.sh, showing the OMEGA
 * banner + a single progress bar driven by install.sh's `==> Phase N` markers.
 */
const { spawn, spawnSync } = require('child_process');
const os = require('os');
const fs = require('fs');
const path = require('path');

const REPO = 'https://github.com/agentik-os/OmegaOS.git';
const tty = process.stdout.isTTY;
const C = (n, s) => (tty ? `\x1b[${n}m${s}\x1b[0m` : s);
const cyan = (s) => C('36', s), mag = (s) => C('35;1', s), grn = (s) => C('32', s),
      red = (s) => C('31', s), gray = (s) => C('90', s), bold = (s) => C('1', s), yel = (s) => C('33', s);

const OMEGA = [
  '    ██████╗ ███╗   ███╗███████╗ ██████╗  █████╗ ',
  '   ██╔═══██╗████╗ ████║██╔════╝██╔════╝ ██╔══██╗',
  '   ██║   ██║██╔████╔██║█████╗  ██║  ███╗███████║',
  '   ██║   ██║██║╚██╔╝██║██╔══╝  ██║   ██║██╔══██║',
  '   ╚██████╔╝██║ ╚═╝ ██║███████╗╚██████╔╝██║  ██║',
  '    ╚═════╝ ╚═╝     ╚═╝╚══════╝ ╚═════╝ ╚═╝  ╚═╝',
];

// install.sh phase markers → ordered progress steps. The clone is step 0.
const STEPS = [
  { key: 'clone',  label: 'Fetching OmegaOS' },
  { re: /Phase 1: Environment/i,   label: 'Environment detection' },
  { re: /Phase 2:/i,               label: 'Dependencies (rust, deps, mosh, bun)' },
  { re: /Phase 3: Building rmux/i, label: 'Building rmux (multiplexer)' },
  { re: /Phase 4: Building OmegaOS/i, label: 'Building OmegaOS' },
  { re: /Phase 5: Configuring/i,   label: 'Configuring (~/.omega, skills, agents, rules)' },
  { re: /Phase 6: Shell/i,         label: 'Shell integration' },
  { re: /Phase 6\.5/i,             label: 'Self-containment (hooks, identity, service)' },
  { re: /Phase 6\.9/i,             label: 'Companion tools' },
  { key: 'done',   label: 'Finalizing' },
];

function banner() {
  process.stdout.write('\n');
  for (const l of OMEGA) process.stdout.write('  ' + mag(l) + '\n');
  process.stdout.write('\n  ' + bold(cyan('OmegaOS')) + gray('  ·  the agentic terminal OS  ·  rmux + AI orchestration') + '\n\n');
}

function bar(idx, total, label) {
  const pct = Math.min(100, Math.round((idx / (total - 1)) * 100));
  const w = 32;
  const filled = Math.round((pct / 100) * w);
  const b = '█'.repeat(filled) + gray('░'.repeat(w - filled));
  const line = `  ${mag('▸ OMEGA')}  ${b} ${bold(String(pct).padStart(3) + '%')}  ${cyan(label)}`;
  if (tty) process.stdout.write('\r\x1b[2K' + line);
  else process.stdout.write(line + '\n');
}

function die(msg) { process.stdout.write('\n  ' + red('✗ ' + msg) + '\n\n'); process.exit(1); }

function have(cmd) { return spawnSync(cmd, ['--version'], { stdio: 'ignore' }).status === 0; }

function main() {
  const args = process.argv.slice(2);
  if (args.includes('--help') || args.includes('-h')) {
    banner();
    process.stdout.write('  Usage: npx omega-os [--dir <path>]\n\n');
    process.stdout.write('    --dir <path>   install location (default: ~/Station/OmegaOS if ~/Station exists, else ./OmegaOS)\n');
    process.stdout.write('    --help         this help\n\n');
    process.stdout.write('  Installs OmegaOS from ' + REPO + '\n  then runs its install.sh (builds rmux + omega, ~8 min on a fresh box).\n\n');
    return;
  }

  banner();

  if (process.platform === 'win32') die('OmegaOS installs on Linux/macOS (rmux + bash). Use WSL or a Linux VPS.');
  if (!have('git')) die('git is required. Install git and re-run: npx omega-os');

  // target dir
  let dir = null;
  const di = args.indexOf('--dir');
  if (di !== -1 && args[di + 1]) dir = args[di + 1];
  if (!dir) {
    const station = path.join(os.homedir(), 'Station');
    dir = fs.existsSync(station) ? path.join(station, 'OmegaOS') : path.join(process.cwd(), 'OmegaOS');
  }
  dir = path.resolve(dir);

  const total = STEPS.length;
  let step = 0;
  bar(step, total, STEPS[0].label);

  // 1) clone (or pull if present)
  const exists = fs.existsSync(path.join(dir, '.git'));
  const cloneArgs = exists
    ? ['-C', dir, 'pull', '--ff-only']
    : ['clone', '--depth', '1', REPO, dir];
  const clone = spawnSync('git', cloneArgs, { encoding: 'utf8' });
  if (clone.status !== 0) die('git ' + (exists ? 'pull' : 'clone') + ' failed:\n  ' + gray((clone.stderr || '').trim()));

  // 2) run install.sh, parse phase markers to drive the bar
  const sh = path.join(dir, 'install.sh');
  if (!fs.existsSync(sh)) die('install.sh not found in ' + dir);
  step = 1; bar(step, total, STEPS[1].label);

  const child = spawn('bash', [sh], { cwd: dir, env: process.env });
  let buf = '';
  const onData = (d) => {
    buf += d.toString();
    let nl;
    while ((nl = buf.indexOf('\n')) !== -1) {
      const line = buf.slice(0, nl); buf = buf.slice(nl + 1);
      for (let i = step; i < STEPS.length; i++) {
        if (STEPS[i].re && STEPS[i].re.test(line)) { step = i; bar(step, total, STEPS[i].label); break; }
      }
    }
  };
  child.stdout.on('data', onData);
  child.stderr.on('data', onData);

  child.on('close', (code) => {
    if (code === 0) {
      step = total - 1; bar(step, total, 'Done');
      process.stdout.write('\n\n  ' + grn('✓ OmegaOS installed') + '  →  ' + bold(dir) + '\n\n');
      process.stdout.write('  Next:\n');
      process.stdout.write('    ' + cyan('source ~/.zshrc') + gray('   # or reopen your shell') + '\n');
      process.stdout.write('    ' + cyan('omega doctor') + gray('     # verify') + '\n');
      process.stdout.write('    ' + cyan('omega') + gray('            # launch the TUI') + '\n\n');
      process.stdout.write('  ' + gray('Set up Telegram / providers from the TUI — Enter on any panel opens a guided wizard.') + '\n\n');
    } else {
      process.stdout.write('\n\n  ' + red('✗ install.sh exited ' + code) + '\n');
      process.stdout.write('  ' + gray('Re-run inside the clone to see full output:  cd ' + dir + ' && ./install.sh') + '\n\n');
      process.exit(code || 1);
    }
  });
}

main();
