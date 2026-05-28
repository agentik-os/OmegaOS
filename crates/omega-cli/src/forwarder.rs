//! Ordered keystroke forwarder.
//!
//! The TUI chat-focus path forwards one keystroke per event to a target rmux
//! session. A previous optimization fired each forward on its own
//! `tokio::spawn`, which — on the multi-threaded `#[tokio::main]` runtime —
//! let N independent tasks race to the SDK transport's single FIFO actor on
//! different worker threads. Fast typing / drain batches could therefore be
//! delivered OUT OF ORDER (e.g. "abc" landing as "acb").
//!
//! This module replaces those racing spawns with ONE consumer task draining a
//! single FIFO mpsc channel. Because exactly one task reaches the transport,
//! ordering is guaranteed. Adjacent `Text` messages for the same session are
//! coalesced into a single send for efficiency; a `Key` or a session switch
//! flushes pending text first, preserving exact interleaving order.

use omega_core::session::SessionManager;
use tokio::sync::mpsc::{self, error::TryRecvError, UnboundedSender};

/// Error sink shared with the TUI loop; the consumer writes here on failure
/// and the loop drains it into `app.status_message` each tick.
pub type StatusSink = std::sync::Arc<std::sync::Mutex<Option<String>>>;

/// A single forward request. `Text` is coalesce-able; `Key` is a discrete
/// named token (Enter, Up, Space, BSpace…) that must not be merged or reordered.
pub enum ForwardMsg {
    Text { session: String, text: String },
    Key { session: String, key: String },
    /// A user paste — forwarded as ONE bracketed-paste block (no auto-Enter)
    /// so the target app buffers it instead of submitting on every embedded
    /// newline. Flushes pending coalesced text first to preserve order.
    Paste { session: String, text: String },
}

fn set_err(sink: &StatusSink, msg: String) {
    if let Ok(mut g) = sink.lock() {
        *g = Some(msg);
    }
}

/// Flush any pending coalesced text for a session, in order, before the next
/// non-text or different-session message is dispatched.
async fn flush_text(pend: &mut Option<(String, String)>, mgr: &SessionManager, sink: &StatusSink) {
    if let Some((session, text)) = pend.take() {
        if let Err(e) = mgr.send_text_raw(&session, &text).await {
            set_err(sink, format!("Forward failed: {}", e));
        }
    }
}

/// Spawn the single ordered consumer task and return the channel sender.
/// The event loop only calls `tx.send(..)` (synchronous, non-blocking on an
/// unbounded channel), so the hot path stays instant while delivery order is
/// guaranteed by the lone consumer.
pub fn spawn_forwarder(mgr: SessionManager, sink: StatusSink) -> UnboundedSender<ForwardMsg> {
    let (tx, mut rx) = mpsc::unbounded_channel::<ForwardMsg>();
    tokio::spawn(async move {
        // Pending coalesced text: (session, accumulated_text).
        let mut pend: Option<(String, String)> = None;
        loop {
            // With pending text, drain greedily (try_recv) to coalesce; when
            // the queue is momentarily empty, flush and block on the next recv.
            let next = if pend.is_some() {
                match rx.try_recv() {
                    Ok(m) => m,
                    Err(TryRecvError::Empty) => {
                        flush_text(&mut pend, &mgr, &sink).await;
                        continue;
                    }
                    Err(TryRecvError::Disconnected) => {
                        flush_text(&mut pend, &mgr, &sink).await;
                        break;
                    }
                }
            } else {
                match rx.recv().await {
                    Some(m) => m,
                    None => break,
                }
            };

            match next {
                ForwardMsg::Text { session, text } => match pend.as_mut() {
                    Some((s, buf)) if *s == session => buf.push_str(&text),
                    _ => {
                        flush_text(&mut pend, &mgr, &sink).await;
                        pend = Some((session, text));
                    }
                },
                ForwardMsg::Key { session, key } => {
                    // Flush queued text BEFORE the key so interleaving order
                    // (e.g. "ab" then BackSpace) is preserved exactly.
                    flush_text(&mut pend, &mgr, &sink).await;
                    if let Err(e) = mgr.send_key(&session, &key).await {
                        set_err(&sink, format!("Forward {} failed: {}", key, e));
                    }
                }
                ForwardMsg::Paste { session, text } => {
                    // Flush queued text first so the paste lands in order, then
                    // send the whole block as one bracketed paste (no Enter).
                    flush_text(&mut pend, &mgr, &sink).await;
                    if let Err(e) = mgr.send_paste_raw(&session, &text).await {
                        set_err(&sink, format!("Paste failed: {}", e));
                    }
                }
            }
        }
    });
    tx
}
