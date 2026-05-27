# TG-SEC — Telegram security: sender_id allow-list

**Category:** Safety
**Added:** 2026-05-27

## Rule

Omega's Telegram bridge accepts messages only from configured allow_user_ids. Everything else is silently dropped + logged.

## Origin

Anyone with the bot token could potentially DM it. Two-level filter ensures only the owner controls the VPS.
