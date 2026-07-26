#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# OmegaOS — omega stream · the snapshot loop
# ───────────────────────────────────────────────────────────────────────────
#   omega-stream.sh <target> <session> [interval] [lines]
#     target   : an ssh host alias from ~/.ssh/config, OR the literal "local"
#     session  : the source rmux session name on that box
#     interval : poll seconds  (default 3)
#     lines    : scrollback lines to capture (default 120)
#
# Installed at ~/.omega/bin/omega-stream.sh. `omega stream` creates the viewer with
#   rmux new-session -d -s <viewer> "$HOME/.omega/bin/omega-stream.sh <target> <session> <interval> <lines>"
# so THIS LOOP *IS* THE VIEWER SESSION'S COMMAND. Four things follow, each learned
# the hard way — treat them as invariants, not preferences:
#
#  1. SNAPSHOT THE RENDERED SCREEN, never replay raw bytes. `pipe-pane -O | tail -f`
#     renders as garbage: a full-screen TUI emits cursor moves and partial redraws
#     that only mean anything against a live screen buffer. capture-pane returns the
#     RENDERED text. Capture, clear, print, sleep. That is the entire mechanism.
#  2. PULL, never push. The viewer box reaches out; the source ships nothing. A
#     push-based shipper died once and the mirror FROZE while the source kept
#     growing — and a frozen mirror is indistinguishable from a quiet one. Pulling
#     puts the liveness of the stream on the box that can notice it stopped.
#  3. THE LOOP MUST NEVER EXIT. If it exits, the rmux session dies and the operator
#     sees NOTHING — strictly worse than an error on screen. No `set -e`. Errors are
#     RENDERED (banner + failure state), never fatal. It retries forever.
#  4. rmux IS NOT tmux and is NOT on the non-interactive PATH: always the absolute
#     $HOME/.local/bin/rmux. In the remote command that path MUST be written \$HOME
#     so it expands on the REMOTE box — a locally-expanded $HOME once pointed a Linux
#     box at /Users/hacker/... , silently.
# ═══════════════════════════════════════════════════════════════════════════

# Deliberately NO `set -e` / `set -u` / `pipefail`: invariant 3. A stray non-zero
# exit or unset var must never take the viewer session down with it.

TARGET="$1"
SESSION="$2"
INTERVAL="${3:-3}"
SCROLLBACK="${4:-120}"          # NOT named LINES: bash owns LINES (terminal height).

RMUX_LOCAL="$HOME/.local/bin/rmux"
# shellcheck disable=SC2016  # NOT expanding here is the whole point: see invariant 4.
RMUX_REMOTE='$HOME/.local/bin/rmux'   # single-quoted: expands on the REMOTE box.

usage() {
    cat <<'EOF'
omega-stream.sh — mirror a live rmux session by snapshotting its rendered screen.

  usage: omega-stream.sh <target> <session> [interval] [lines]

    target    ssh host alias from ~/.ssh/config, or the literal "local"
    session   rmux session name on that box
    interval  poll seconds (default 3)
    lines     scrollback lines to capture (default 120)

  examples:
    omega-stream.sh local BluePrint-OS
    omega-stream.sh matrix MAC-STREAM 3 120
EOF
}

# ── arg validation ────────────────────────────────────────────────────────────
# Bad args are a STARTUP fault, not a runtime one, so this is the ONE place the
# script may stop: invariant 3 governs failures hit while STREAMING, not an argv we
# cannot use at all. It still must not vanish instantly — if we are the command of a
# viewer session, exiting takes the session with us and the operator never gets to
# read why — so the usage is held on screen for a BOUNDED grace first.
# Bounded, never forever: rmux exports RMUX / RMUX_PANE (NOT $TMUX, which reports
# "not in a multiplexer" from inside rmux), but neither can tell "I am the session's
# command" from "a human typed me at a prompt inside a session" — both have them set.
# So an unbounded hold would freeze the terminal of anyone who simply typo'd.
USAGE_GRACE=10
die_usage() {
    printf '%s\n\n' "omega-stream: $1" >&2
    usage >&2
    if [ -t 1 ]; then
        printf '\n[warn]  omega stream · BAD ARGUMENTS · exiting in %ss (ctrl-c to quit now)\n' "$USAGE_GRACE" >&2
        sleep "$USAGE_GRACE"
    fi
    exit 2
}

[ -z "$TARGET" ]  && die_usage "missing <target>"
[ -z "$SESSION" ] && die_usage "missing <session>"
case "$SESSION" in *\'*) die_usage "session name may not contain a single quote";; esac
case "$TARGET"  in *\'*|-*) die_usage "invalid target '$TARGET'";; esac
case "$INTERVAL"   in ''|*[!0-9]*) die_usage "interval must be a positive integer (got '$INTERVAL')";; esac
case "$SCROLLBACK" in ''|*[!0-9]*) die_usage "lines must be a positive integer (got '$SCROLLBACK')";; esac
[ "$INTERVAL" -lt 1 ]   && INTERVAL=1
[ "$SCROLLBACK" -lt 1 ] && SCROLLBACK=1

LABEL="$TARGET:$SESSION"
ERRFILE="$(mktemp -t omega-stream.XXXXXX 2>/dev/null || echo "/tmp/omega-stream.$$.err")"
trap 'rm -f "$ERRFILE"' EXIT

# ── state ─────────────────────────────────────────────────────────────────────
LAST_FRAME=""       # last frame we successfully captured
LAST_OK=0           # epoch seconds of that capture; 0 == never connected

# ── geometry ──────────────────────────────────────────────────────────────────
# Re-read every iteration: the viewer resizes the moment the operator attaches.
term_rows() {
    local r
    r="$(tput lines 2>/dev/null)"
    case "$r" in ''|*[!0-9]*) r=40;; esac
    [ "$r" -lt 8 ] && r=8
    printf '%s' "$r"
}
term_cols() {
    local c
    c="$(tput cols 2>/dev/null)"
    case "$c" in ''|*[!0-9]*) c=200;; esac
    [ "$c" -lt 20 ] && c=20
    printf '%s' "$c"
}

# Truncate to the viewer width so nothing wraps. A wrapped line costs a second row
# and the screen scrolls, which loses the TOP of the frame — worse than losing the
# right edge. Only when stdout is a tty: piped output (tests, logs) stays complete.
fit() {
    local s="$1"
    if [ "$IS_TTY" = 1 ] && [ "${#s}" -gt "$COLS" ]; then
        printf '%s' "${s:0:$COLS}"
    else
        printf '%s' "$s"
    fi
}

# ── the capture ───────────────────────────────────────────────────────────────
# stdout -> the frame, stderr -> $ERRFILE (never mixed into the frame), rc -> state.
capture() {
    if [ "$TARGET" = "local" ]; then
        "$RMUX_LOCAL" capture-pane -p -t "$SESSION" -S -"$SCROLLBACK" 2>"$ERRFILE"
    else
        # \$HOME is written so it expands on the REMOTE box (invariant 4).
        # -n : stdin from /dev/null, so ssh never swallows the viewer's keystrokes.
        ssh -n -o ConnectTimeout=10 -o BatchMode=yes "$TARGET" \
            "$RMUX_REMOTE capture-pane -p -t '$SESSION' -S -$SCROLLBACK" 2>"$ERRFILE"
    fi
}

# ── the render ────────────────────────────────────────────────────────────────
# line 1        : status banner
# lines 2..N-1  : the last (rows-2) lines of the frame  (row N stays free: no scroll)
render() {
    local banner="$1" extra="$2" body="$3"
    local avail=$(( ROWS - 2 ))
    [ -n "$extra" ] && avail=$(( avail - 1 ))
    [ "$avail" -lt 1 ] && avail=1

    # Split $body into lines without mapfile: OmegaOS also installs on macOS, whose
    # /bin/bash is 3.2 and has no mapfile.
    local -a lines=()
    local ln
    if [ -n "$body" ]; then
        while IFS= read -r ln || [ -n "$ln" ]; do lines+=("$ln"); done <<< "$body"
    fi

    local n=${#lines[@]}
    local start=$(( n - avail ))
    [ "$start" -lt 0 ] && start=0

    [ "$IS_TTY" = 1 ] && printf '\033[H\033[2J\033[3J'   # home + clear + drop scrollback

    fit "$banner"; printf '\n'
    [ -n "$extra" ] && { fit "$extra"; printf '\n'; }
    local l
    for l in "${lines[@]:$start:$avail}"; do fit "$l"; printf '\n'; done
}

# ── the loop ──────────────────────────────────────────────────────────────────
while true; do
    ROWS="$(term_rows)"
    COLS="$(term_cols)"
    IS_TTY=0; [ -t 1 ] && IS_TTY=1

    OUT="$(capture)"; RC=$?
    ERR="$(tr '\n' ' ' < "$ERRFILE" 2>/dev/null | sed 's/  */ /g; s/ *$//')"
    NOW="$(date +%s)"

    if [ "$RC" -eq 0 ]; then
        # A capture that SUCCEEDS but is unchanged is a QUIET session, not a dead
        # one. Liveness is "the last capture succeeded", never "the content moved".
        LAST_FRAME="$OUT"
        LAST_OK="$NOW"
        render "[live]  omega stream · $LABEL · $(date +%H:%M:%S) · every ${INTERVAL}s" "" "$OUT"
    else
        # Two causes. ssh's own 255 means we never reached the box; anything else
        # non-zero means the command ran and rmux could not find the session.
        if [ "$TARGET" != "local" ] && [ "$RC" -eq 255 ]; then
            STATE="HOST UNREACHABLE"
        else
            STATE="SESSION NOT FOUND"
        fi

        if [ "$LAST_OK" -gt 0 ]; then
            # SOURCE GONE: we HAD frames and stopped getting them. Keep the last good
            # frame on screen, but label it stale with its age — a frozen mirror must
            # never be able to pass for a quiet one.
            AGE=$(( NOW - LAST_OK ))
            render \
                "[warn]  omega stream · $LABEL · $STATE · last good frame ${AGE}s ago" \
                "------- SOURCE GONE · frame below is STALE (${AGE}s old) · ${ERR:-no error text} -------" \
                "$LAST_FRAME"
        else
            render \
                "[warn]  omega stream · $LABEL · $STATE · never connected" \
                "" \
                "$(printf 'omega stream could not read %s\n\n  state : %s\n  exit  : %s\n  error : %s\n\nretrying every %ss — this loop never gives up.' \
                     "$LABEL" "$STATE" "$RC" "${ERR:-(no output on stderr)}" "$INTERVAL")"
        fi
    fi

    sleep "$INTERVAL"
done
