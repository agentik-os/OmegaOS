#!/usr/bin/env python3
"""
OmegaOS Telegram Bot — Remote dispatch interface.

Receives messages in a Telegram group, dispatches missions to oracles,
monitors done.json signals, and posts results back.

Setup:
  1. Create a bot via @BotFather
  2. Set OMEGA_BOT_TOKEN and OMEGA_CHAT_ID in .env
  3. Run: python bot/main.py

Requires: python-telegram-bot>=21.0
"""

import asyncio
import json
import logging
import os
import subprocess
from pathlib import Path

logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s")
logger = logging.getLogger("omega-bot")

STATE_DIR = Path(os.environ.get("OMEGA_STATE", Path.home() / ".omega" / "state"))
BOT_TOKEN = os.environ.get("OMEGA_BOT_TOKEN", "")
CHAT_ID = int(os.environ.get("OMEGA_CHAT_ID", "0"))
OMEGA_BIN = os.environ.get("OMEGA_BIN", "omega")


def run_omega(*args: str) -> tuple[int, str]:
    """Run an omega CLI command and return (exit_code, output)."""
    try:
        result = subprocess.run(
            [OMEGA_BIN, *args],
            capture_output=True,
            text=True,
            timeout=30,
        )
        return result.returncode, result.stdout + result.stderr
    except Exception as e:
        return 1, str(e)


def scan_done_signals() -> list[dict]:
    """Scan state dir for new done.json files."""
    signals = []
    if not STATE_DIR.exists():
        return signals
    for path in STATE_DIR.glob("worker-*.done.json"):
        consumed = path.with_suffix(".json.consumed")
        if consumed.exists():
            continue
        try:
            data = json.loads(path.read_text())
            signals.append(data)
        except (json.JSONDecodeError, OSError):
            pass
    return signals


def consume_done_signal(session: str) -> None:
    """Mark a done.json as consumed."""
    path = STATE_DIR / f"worker-{session}.done.json"
    consumed = path.with_suffix(".json.consumed")
    if path.exists():
        path.rename(consumed)


async def handle_message(text: str, send_reply) -> None:
    """Route an incoming message to the right omega command."""
    text = text.strip()

    if text.startswith("/start"):
        await send_reply(
            "OmegaOS Bot ready.\n\n"
            "Commands:\n"
            "/list — Show active sessions\n"
            "/dispatch <project> <mission> — Dispatch oracle\n"
            "/status <session> — Show session output\n"
            "/kill <session> — Kill a session\n"
            "/patrol — Run health check"
        )
        return

    if text.startswith("/list"):
        code, output = run_omega("list")
        await send_reply(f"```\n{output}\n```")
        return

    if text.startswith("/dispatch"):
        parts = text.split(maxsplit=2)
        if len(parts) < 3:
            await send_reply("Usage: /dispatch <project> <mission>")
            return
        project, mission = parts[1], parts[2]
        code, output = run_omega("dispatch", project, mission)
        icon = "✓" if code == 0 else "✗"
        await send_reply(f"{icon} {output}")
        return

    if text.startswith("/status"):
        parts = text.split(maxsplit=1)
        if len(parts) < 2:
            await send_reply("Usage: /status <session>")
            return
        code, output = run_omega("status", parts[1])
        await send_reply(f"```\n{output[-2000:]}\n```")
        return

    if text.startswith("/kill"):
        parts = text.split(maxsplit=1)
        if len(parts) < 2:
            await send_reply("Usage: /kill <session>")
            return
        code, output = run_omega("kill", parts[1])
        await send_reply(output)
        return

    if text.startswith("/patrol"):
        code, output = run_omega("patrol", "--once")
        await send_reply(f"```\n{output}\n```")
        return

    # Default: treat as a dispatch to default project
    await send_reply(
        "Unknown command. Use /list, /dispatch, /status, /kill, or /patrol."
    )


async def done_watcher(send_message, interval: int = 30) -> None:
    """Background loop: watch for done.json signals and post them."""
    while True:
        try:
            signals = scan_done_signals()
            for signal in signals:
                session = signal.get("session", "unknown")
                status = signal.get("status", "unknown")
                summary = signal.get("summary", "")
                icon = {"done_clean": "✅", "pending": "⏳", "failed": "❌"}.get(status, "❓")

                msg = f"{icon} **{session}** → {status}\n{summary}"
                await send_message(msg)
                consume_done_signal(session)
        except Exception as e:
            logger.error(f"Done watcher error: {e}")

        await asyncio.sleep(interval)


def main():
    """Entry point — requires python-telegram-bot."""
    if not BOT_TOKEN:
        logger.error("Set OMEGA_BOT_TOKEN environment variable")
        logger.info("Falling back to CLI-only mode (no Telegram)")
        logger.info("Run omega commands directly via the omega CLI")
        return

    try:
        from telegram import Update
        from telegram.ext import Application, MessageHandler, CommandHandler, filters
    except ImportError:
        logger.error("Install python-telegram-bot: pip install python-telegram-bot>=21.0")
        return

    app = Application.builder().token(BOT_TOKEN).build()

    async def on_message(update: Update, context):
        if update.message and update.message.text:
            async def reply(text):
                await update.message.reply_text(text, parse_mode="Markdown")
            await handle_message(update.message.text, reply)

    app.add_handler(MessageHandler(filters.TEXT & ~filters.COMMAND, on_message))
    app.add_handler(CommandHandler("start", lambda u, c: on_message(u, c)))
    app.add_handler(CommandHandler("list", lambda u, c: on_message(u, c)))
    app.add_handler(CommandHandler("dispatch", lambda u, c: on_message(u, c)))
    app.add_handler(CommandHandler("status", lambda u, c: on_message(u, c)))
    app.add_handler(CommandHandler("kill", lambda u, c: on_message(u, c)))
    app.add_handler(CommandHandler("patrol", lambda u, c: on_message(u, c)))

    logger.info("OmegaOS Telegram bot starting...")
    app.run_polling()


if __name__ == "__main__":
    main()
