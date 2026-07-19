#!/usr/bin/env bash
# stax-scaffold.sh — vendor the Stax engine + a generic shell + tokens into a target
# project, so any app can be ported to the panel grammar (Phase 2 of the conversion
# playbook). Idempotent: copies/refreshes the framework files, and writes the shell /
# registry / entry / css stubs ONLY if they do not already exist (never clobbers your
# domain wiring). Zero deps beyond React. Works for Vite or Next (client) apps.
set -uo pipefail

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
REPO="$OMEGA_DIR/repos/stax"
CORE="$REPO/frameword/packages/panels-core/src/index.ts"
REACT="$REPO/frameword/packages/panels-react/src/index.tsx"
TOKENS="$REPO/frameword/apps/crm-specimen/src/tokens.css"

c_info(){ printf '\033[36m[stax]\033[0m %s\n' "$*"; }
c_ok(){   printf '\033[32m[stax]\033[0m %s\n' "$*"; }
c_warn(){ printf '\033[33m[stax]\033[0m %s\n' "$*"; }
c_die(){  printf '\033[31m[stax]\033[0m %s\n' "$*"; exit 1; }

TARGET="${1:-}"
[[ -n "$TARGET" ]] || c_die "usage: stax-scaffold.sh <target-project-dir>"
[[ -d "$TARGET" ]] || c_die "target dir not found: $TARGET"

# Ensure the framework checkout exists (clone if the installer never ran).
if [[ ! -f "$CORE" || ! -f "$REACT" ]]; then
  c_warn "framework not found at $REPO — cloning agentik-os/stax main"
  mkdir -p "$OMEGA_DIR/repos"
  git clone https://github.com/agentik-os/stax "$REPO" >/dev/null 2>&1 \
    || c_die "clone failed — check network / gh auth"
fi
[[ -f "$CORE" && -f "$REACT" ]] || c_die "engine source still missing under $REPO"

# Choose src/stax or stax at the project root.
if [[ -d "$TARGET/src" ]]; then DST="$TARGET/src/stax"; else DST="$TARGET/stax"; fi
mkdir -p "$DST"
c_info "scaffolding into $DST"

# 1) Vendor the engine (import rewritten to the local file — no npm package needed).
cp -f "$CORE" "$DST/panels-core.ts"
sed 's#@frameword/panels-core#./panels-core#g' "$REACT" > "$DST/panels-react.tsx"
[[ -f "$TOKENS" ]] && cp -f "$TOKENS" "$DST/tokens.css"
c_ok "engine vendored: panels-core.ts, panels-react.tsx, tokens.css (refreshed from main)"

# 2) Generic shell (only if absent).
write_if_absent(){ # $1 path ; stdin = content
  if [[ -e "$1" ]]; then c_warn "kept existing $(basename "$1")"; cat >/dev/null; else cat > "$1"; c_ok "wrote $(basename "$1")"; fi
}

write_if_absent "$DST/shell.tsx" <<'EOF'
"use client";
import { useEffect, type ReactNode } from "react";
import { useWorkspace, useIsCompact, panelWidth, type WorkspaceApi } from "./panels-react";
import type { PanelInstance } from "./panels-core";

export type Renderers = Record<string, (panel: PanelInstance, ws: WorkspaceApi) => ReactNode>;
export interface SpaceDef {
  spaceId: string;
  label: string;
  target: { panelType: string; resourceKey: string };
}

function label(ws: WorkspaceApi, p: PanelInstance) {
  const def = ws.registry[p.target.panelType];
  return def?.label ? def.label(p.target.resourceKey) : p.target.resourceKey;
}

export function Shell({ spaces, renderers }: { spaces: SpaceDef[]; renderers: Renderers }) {
  const ws = useWorkspace();
  const compact = useIsCompact();
  useEffect(() => {
    const h = (e: KeyboardEvent) => {
      if (e.key === "Escape" && ws.state.focusedPanelId) ws.closePanel(ws.state.focusedPanelId);
    };
    window.addEventListener("keydown", h);
    return () => window.removeEventListener("keydown", h);
  }, [ws]);
  return (
    <div className="stax-root">
      <aside className="stax-sidebar">
        {spaces.map((s) => (
          <button key={s.spaceId} className="stax-space-btn"
            data-active={ws.state.spaceId === s.spaceId}
            onClick={() => ws.openSpace(s.spaceId, s.target)}>{s.label}</button>
        ))}
      </aside>
      <div className="stax-main">
        <header className="stax-topbar"><Breadcrumb /></header>
        {compact ? <PushHost renderers={renderers} /> : <ColumnHost renderers={renderers} />}
      </div>
    </div>
  );
}

function Breadcrumb() {
  const ws = useWorkspace();
  return (
    <nav className="stax-breadcrumb" aria-label="Breadcrumb">
      {ws.path.map((id, i) => {
        const p = ws.state.panelsById[id];
        return (
          <span key={id}>
            {i > 0 && <span className="stax-sep">/</span>}
            <button className="stax-crumb" onClick={() => ws.navigateTo(id)}>{label(ws, p)}</button>
          </span>
        );
      })}
    </nav>
  );
}

function ColumnHost({ renderers }: { renderers: Renderers }) {
  const ws = useWorkspace();
  if (!ws.state.rootInstanceId) return <Empty />;
  const context = ws.path.map((id) => ws.state.panelsById[id]);
  const refs = ws.state.referenceRailOrder.map((id) => ws.state.panelsById[id]);
  return (
    <div className="stax-stage">
      {context.map((p) => <Panel key={p.instanceId} panel={p} renderers={renderers} />)}
      {refs.map((p) => <Panel key={p.instanceId} panel={p} renderers={renderers} isReference />)}
    </div>
  );
}

function PushHost({ renderers }: { renderers: Renderers }) {
  const ws = useWorkspace();
  const leafId = ws.state.contextLeafId;
  if (!ws.state.rootInstanceId || !leafId) return <Empty />;
  return (
    <div className="stax-stage stax-push">
      <Panel panel={ws.state.panelsById[leafId]} renderers={renderers} />
    </div>
  );
}

function Panel({ panel, renderers, isReference }: { panel: PanelInstance; renderers: Renderers; isReference?: boolean }) {
  const ws = useWorkspace();
  const width = panelWidth(ws.registry, panel);
  const isRoot = panel.role === "root";
  const body = renderers[panel.target.panelType];
  const pinned = panel.retention === "retained";
  return (
    <section className="stax-panel" data-role={panel.role} data-reference={isReference || undefined}
      data-focused={ws.state.focusedPanelId === panel.instanceId}
      style={{ width }} onMouseDown={() => ws.focusPanel(panel.instanceId)}>
      <header className="stax-panel-head">
        <span className="stax-panel-title">{label(ws, panel)}</span>
        <span className="stax-panel-ctrls">
          {!isRoot && !isReference && (
            <button className="stax-ico" title={pinned ? "Unpin" : "Pin"}
              onClick={() => (pinned ? ws.unpinPanel(panel.instanceId) : ws.pinPanel(panel.instanceId))}>
              {pinned ? "★" : "☆"}
            </button>
          )}
          <button className="stax-ico" title="Close"
            onClick={() => (isReference ? ws.unpinPanel(panel.instanceId) : ws.closePanel(panel.instanceId))}>✕</button>
        </span>
      </header>
      <div className="stax-panel-body">
        {body ? body(panel, ws) : <div className="stax-missing">No renderer for &quot;{panel.target.panelType}&quot;. Add one in registry.tsx.</div>}
      </div>
      {/* The footer is the ONE action zone — a renderer that needs actions returns its
          own <footer className="stax-panel-foot">…</footer> as the last body element. */}
    </section>
  );
}

function Empty() {
  return <div className="stax-empty">Pick a space to begin.</div>;
}
EOF

write_if_absent "$DST/registry.tsx" <<'EOF'
import type { PanelRegistry } from "./panels-react";
import type { SpaceDef, Renderers } from "./shell";

// width belongs to the KIND, not the user (S 380 / M 480 / L 640 / XL 800).
export const REGISTRY: PanelRegistry = {
  home: { size: "M", label: () => "Home" },
  // contact: { size: "M", label: (k) => `Contact ${k}` },
  // deal:    { size: "L", label: (k) => `Deal ${k}` },
};

// The sidebar sections — each opens a Space (replaces the active thread).
export const SPACES: SpaceDef[] = [
  { spaceId: "home", label: "Home", target: { panelType: "home", resourceKey: "home" } },
];

// One renderer per panelType. Fetch data from panel.target.resourceKey / params.
// Drill to the right with ws.openDetail(panel.instanceId, { panelType, resourceKey }).
export const renderers: Renderers = {
  home: (_panel, ws) => (
    <div style={{ padding: 16 }}>
      <p className="body">Empty Stax workspace. Register panel types and renderers here,</p>
      <p className="body-sm">then drill with <code>ws.openDetail(panel.instanceId, target)</code>.</p>
    </div>
  ),
};
EOF

write_if_absent "$DST/Stax.tsx" <<'EOF'
"use client";
import { WorkspaceProvider } from "./panels-react";
import { Shell } from "./shell";
import { REGISTRY, SPACES, renderers } from "./registry";
import "./stax.css";

// Mount <Stax /> as your single app surface (one route in Next; the app root in Vite).
export function Stax() {
  return (
    <WorkspaceProvider registry={REGISTRY} urlSync storageKey="stax-workspace">
      <Shell spaces={SPACES} renderers={renderers} />
    </WorkspaceProvider>
  );
}
EOF

write_if_absent "$DST/stax.css" <<'EOF'
@import "./tokens.css";

.stax-root { display: flex; height: 100vh; background: var(--background); color: var(--foreground); }
.stax-sidebar { width: 220px; flex: none; border-right: 1px solid var(--border);
  background: var(--sidebar); padding: 12px 8px; display: flex; flex-direction: column; gap: 2px; }
.stax-space-btn { text-align: left; padding: 8px 12px; border: 0; border-radius: var(--radius);
  background: transparent; color: var(--foreground); font: inherit; cursor: pointer; }
.stax-space-btn:hover { background: var(--muted); }
.stax-space-btn[data-active="true"] { background: var(--secondary); font-weight: 600; }

.stax-main { flex: 1; min-width: 0; display: flex; flex-direction: column; }
.stax-topbar { height: 48px; flex: none; border-bottom: 1px solid var(--border);
  display: flex; align-items: center; padding: 0 16px; }
.stax-breadcrumb { display: flex; align-items: center; gap: 6px; font-size: 13px; }
.stax-crumb { border: 0; background: transparent; color: var(--muted-foreground);
  cursor: pointer; padding: 2px 4px; border-radius: var(--radius-sm); font: inherit; }
.stax-crumb:hover { color: var(--foreground); background: var(--muted); }
.stax-sep { color: var(--border); }

.stax-stage { flex: 1; display: flex; align-items: stretch; gap: 0;
  overflow-x: auto; overflow-y: hidden; scroll-behavior: smooth; }
.stax-stage.stax-push { overflow-x: hidden; }
.stax-stage.stax-push .stax-panel { width: 100% !important; }

.stax-panel { flex: none; display: flex; flex-direction: column; min-width: 0;
  border-right: 1px solid var(--border); background: var(--card); }
.stax-panel[data-role="root"] { position: sticky; left: 0; z-index: 2; }
.stax-panel[data-reference="true"] { background: var(--secondary); }
.stax-panel[data-focused="true"] { box-shadow: inset 0 2px 0 0 var(--accent); }
.stax-panel-head { height: 44px; flex: none; display: flex; align-items: center;
  justify-content: space-between; padding: 0 12px; border-bottom: 1px solid var(--border); }
.stax-panel-title { font-weight: 600; font-size: 13px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.stax-panel-ctrls { display: flex; gap: 2px; flex: none; }
.stax-ico { border: 0; background: transparent; cursor: pointer; color: var(--muted-foreground);
  width: 26px; height: 26px; border-radius: var(--radius-sm); font-size: 13px; }
.stax-ico:hover { background: var(--muted); color: var(--foreground); }
.stax-panel-body { flex: 1; overflow-y: auto; overflow-x: hidden; container-type: inline-size; }
.stax-panel-foot { flex: none; border-top: 1px solid var(--border); padding: 12px;
  display: flex; gap: 8px; justify-content: flex-end; background: var(--card); }

.stax-empty, .stax-missing { padding: 32px; color: var(--muted-foreground); }
EOF

cat <<NEXT

$(c_ok "scaffold complete → $DST")
Next steps (Phase 3 — PORT):
  1. Mount <Stax /> as your single app surface:
       Vite  → import { Stax } from "./stax/Stax"; render <Stax /> in main.tsx
       Next  → a client component at app/page.tsx that returns <Stax />
  2. Fill $DST/registry.tsx — one panelType + renderer per screen, and the SPACES sidebar.
  3. Replace navigations: router.push / <Link> → ws.openDetail(parentId, target) (drill)
     or ws.openSpace(spaceId, target) (section). Delete view/edit modals; keep dialogs.
  4. Verify: ws.violations must stay [] ; a shared URL restores the ContextPath.
See ~/.omega/skills/stax/references/conversion-playbook.md for the full pipeline.
NEXT
