# /logicaudit — Before/After Matrix (hermux-perf)

Only ONE file was modified by this audit: `crates/omega-core/src/session.rs`
(removed dead `fn shell_escape`). Everything else is read-only analysis.

| Item | Before | After | Verdict |
|---|---|---|---|
| `cargo build` (workspace) | Finished, 0 errors, 14 dead-code warnings | Finished, 0 errors, 13 dead-code warnings | ✅ no regression |
| `shell_escape` (session.rs:575) | present, 0 callers, `dead_code` warning | removed | ✅ warning gone, nothing broke |
| `team.rs::shell_escape` (separate, used) | used at team.rs:68 | untouched | ✅ unaffected |
| FIFO keystroke order (forwarder) | guaranteed (single consumer) | unchanged (read-only) | ✅ verified safe |
| Bounded drain (main.rs:1181) | 8ms budget, ZERO-poll exit | unchanged | ✅ verified safe |
| Scrollback fast/slow split (app.rs) | snapshot tail / subprocess history | unchanged | ✅ verified safe |
| Paste chunking (session.rs:497) | bracketed + 4096B char-boundary chunks | unchanged | ✅ verified safe |

## Breakage scan
`grep -rn "shell_escape(" crates/omega-core/src/` → only `team.rs:68` (its own
copy). No reference to the removed session.rs function anywhere. Post-fix build
clean. **0 breakages.**
