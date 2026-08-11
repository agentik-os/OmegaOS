#!/usr/bin/env bun
import { createHash } from "node:crypto";
import { readdir, readFile, stat, writeFile } from "node:fs/promises";
import { join, relative, resolve, sep } from "node:path";

const PRODUCTS = [
  "mindset-os",
  "health-energy-os",
  "habit-tracker-os",
  "alignment-os",
  "strategy-portfolio-os",
  "brainstorm-os",
  "market-research-os",
  "blueprint-os",
  "design-os",
  "stepper-os",
  "builder-os",
  "quality-evaluation-release-os",
  "storyteller-os",
  "revenue-os",
  "delivery-customer-success-os",
  "relationship-network-os",
  "wealth-capital-os",
  "execution-os",
  "operations-automation-os",
  "review-governance-os",
  "context-memory-os",
  "ai-logic-os",
  "content-os",
  "books-os",
] as const;

const REQUIRED = [
  "README.md",
  "README_FR.md",
  "MANIFEST.json",
  "OMEGA_INTEGRATION.md",
  "SKILL.md",
] as const;

type FileRecord = { path: string; sha256: string; bytes: number };
type Manifest = Record<string, unknown> & {
  counts?: Record<string, number>;
  files?: Array<string | FileRecord>;
};

const repoRoot = resolve(import.meta.dir, "..");
const osRoot = join(repoRoot, "OS");
const writeMode = process.argv.includes("--write");
const EXCLUDED_DIRS = new Set([
  ".git",
  ".pytest_cache",
  ".venv",
  "__pycache__",
  "ledger",
  "node_modules",
]);

async function walk(root: string, dir = root): Promise<string[]> {
  const found: string[] = [];
  const entries = await readdir(dir, { withFileTypes: true });
  entries.sort((a, b) => a.name.localeCompare(b.name));
  for (const entry of entries) {
    if (EXCLUDED_DIRS.has(entry.name) || entry.name.endsWith(".egg-info")) continue;
    const fullPath = join(dir, entry.name);
    if (entry.isDirectory()) {
      found.push(...(await walk(root, fullPath)));
    } else if (
      entry.isFile() &&
      entry.name !== "MANIFEST.json" &&
      entry.name !== "checksums.sha256" &&
      !entry.name.endsWith(".pyc")
    ) {
      found.push(relative(root, fullPath).split(sep).join("/"));
    }
  }
  return found;
}

async function record(root: string, path: string): Promise<FileRecord> {
  const content = await readFile(join(root, path));
  return {
    path,
    sha256: createHash("sha256").update(content).digest("hex"),
    bytes: content.byteLength,
  };
}

async function verifyMarkdownLinks(root: string, paths: string[], slug: string): Promise<void> {
  for (const path of paths.filter((candidate) => candidate.endsWith(".md"))) {
    const content = await readFile(join(root, path), "utf8");
    const links = content.matchAll(/\[[^\]]*\]\(([^)]+)\)/g);
    for (const match of links) {
      const rawTarget = match[1].trim().replace(/^<|>$/g, "");
      if (
        !rawTarget ||
        rawTarget.startsWith("#") ||
        rawTarget.startsWith("/") ||
        rawTarget.startsWith("~") ||
        /^[a-z][a-z0-9+.-]*:/i.test(rawTarget)
      ) {
        continue;
      }
      const target = decodeURIComponent(rawTarget.split("#", 1)[0]);
      const resolved = resolve(root, join(path, ".."), target);
      if (!resolved.startsWith(`${root}${sep}`) && resolved !== root) {
        failures.push(`${slug}: ${path} link escapes the pack (${rawTarget})`);
        continue;
      }
      try {
        await stat(resolved);
      } catch {
        failures.push(`${slug}: ${path} has broken link ${rawTarget}`);
      }
    }
  }
}

function countPrefix(files: FileRecord[], prefix: string): number {
  return files.filter((file) => file.path.startsWith(`${prefix}/`)).length;
}

function expectedCounts(files: FileRecord[]): Record<string, number> {
  const nestedSkills = countPrefix(files, "skills");
  return {
    agents: countPrefix(files, "agents"),
    skills: nestedSkills || (files.some((file) => file.path === "SKILL.md") ? 1 : 0),
    protocols: countPrefix(files, "protocols"),
    references: countPrefix(files, "references"),
    scripts: countPrefix(files, "scripts"),
    assets: countPrefix(files, "assets"),
    files: files.length,
  };
}

function stable(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function checksumFile(files: FileRecord[]): string {
  return files.map((file) => `${file.sha256}  ${file.path}`).join("\n") + "\n";
}

const failures: string[] = [];
let verifiedFiles = 0;

for (const slug of PRODUCTS) {
  const root = join(osRoot, slug);
  for (const required of REQUIRED) {
    try {
      const metadata = await stat(join(root, required));
      if (!metadata.isFile()) failures.push(`${slug}: ${required} is not a file`);
    } catch {
      failures.push(`${slug}: missing ${required}`);
    }
  }

  let manifest: Manifest;
  try {
    manifest = JSON.parse(await readFile(join(root, "MANIFEST.json"), "utf8"));
  } catch (error) {
    failures.push(`${slug}: MANIFEST.json is invalid (${String(error)})`);
    continue;
  }

  const paths = await walk(root);
  const files = await Promise.all(paths.map((path) => record(root, path)));
  files.sort((a, b) => a.path.localeCompare(b.path));
  await verifyMarkdownLinks(root, paths, slug);
  const counts = expectedCounts(files);
  const expectedChecksum = checksumFile(files);
  const nextManifest: Manifest = {
    ...manifest,
    schema_version: 1,
    counts,
    files,
  };

  if (writeMode) {
    await writeFile(join(root, "MANIFEST.json"), stable(nextManifest));
    await writeFile(join(root, "checksums.sha256"), expectedChecksum);
  } else {
    if (stable(manifest) !== stable(nextManifest)) {
      failures.push(`${slug}: MANIFEST.json inventory or counts are stale`);
    }
    let actualChecksum = "";
    try {
      actualChecksum = await readFile(join(root, "checksums.sha256"), "utf8");
    } catch {
      failures.push(`${slug}: missing checksums.sha256`);
    }
    if (actualChecksum !== expectedChecksum) {
      failures.push(`${slug}: checksums.sha256 is stale`);
    }
  }
  verifiedFiles += files.length;
}

if (failures.length) {
  for (const failure of failures) console.error(`FAIL ${failure}`);
  process.exit(1);
}

console.log(
  `${writeMode ? "Wrote" : "Verified"} ${PRODUCTS.length} OS manifests covering ${verifiedFiles} content files.`,
);
