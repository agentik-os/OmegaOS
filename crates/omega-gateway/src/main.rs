use clap::{Parser, Subcommand};
use omega_gateway::account_login::{self, AuthStatus};
use omega_gateway::accounts::AccountStore;
use omega_gateway::auth::{DeviceStore, PairingCode};
use omega_gateway::chat_store::ChatStore;
use omega_gateway::config::{gateway_dir, GatewayConfig};
use omega_gateway::protocol::AccountKind;
use omega_gateway::server::{build_router, AppState};

#[derive(Parser)]
#[command(name = "omega-gatewayd", about = "OmegaOS gateway daemon")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run the gateway server (default)
    Serve,
    /// Print a one-time pairing code + QR (valid 5 minutes)
    Pair,
    /// Print the wire-protocol JSON Schema (for TS type generation)
    Schema,
    /// List paired devices (id, name, created_at, revoked)
    Devices,
    /// List chats (id, title, agent, updated_at)
    Chats,
    /// Revoke a device by id (its token stops verifying immediately)
    Revoke {
        device_id: String,
    },
    /// List account slots (slug, label, kind, default, live auth status)
    Accounts,
    /// Create a new isolated credential slot (kind: claude|codex)
    AccountAdd {
        slug: String,
        label: String,
        kind: String,
    },
}

/// Parses the CLI's free-text `kind` argument into an [`AccountKind`].
/// Pure and case-insensitive so `Claude`/`CODEX` are accepted alongside the
/// documented lowercase form; anything else is `None` (the caller reports
/// the exact invalid value and exits non-zero).
fn parse_account_kind(s: &str) -> Option<AccountKind> {
    match s.to_lowercase().as_str() {
        "claude" => Some(AccountKind::Claude),
        "codex" => Some(AccountKind::Codex),
        _ => None,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    let dir = gateway_dir();
    match cli.command.unwrap_or(Command::Serve) {
        Command::Pair => {
            let pc = PairingCode::create(&dir, 300)?;
            let host = hostname_or_default();
            let payload = format!("omega://pair?host={host}&code={}", pc.code);
            qr2term::print_qr(&payload).ok();
            println!("Pairing code: {}  (valid 5 minutes)", pc.code);
            println!("Payload: {payload}");
        }
        Command::Schema => println!("{}", omega_gateway::protocol::schema_json()),
        Command::Devices => {
            let store = DeviceStore::open(&dir);
            let devices = store.list();
            if devices.is_empty() {
                println!("no paired devices");
            } else {
                println!("{:<18} {:<20} {:<28} REVOKED", "ID", "NAME", "CREATED_AT");
                for d in devices {
                    println!("{:<18} {:<20} {:<28} {}", d.id, d.name, d.created_at, d.revoked);
                }
            }
        }
        Command::Chats => {
            let store = ChatStore::open(&dir);
            let chats = store.list();
            if chats.is_empty() {
                println!("no chats");
            } else {
                println!("{:<18} {:<24} {:<8} UPDATED_AT", "ID", "TITLE", "AGENT");
                for c in chats {
                    let agent = match c.agent {
                        omega_gateway::protocol::ChatAgent::Claude => "claude",
                        omega_gateway::protocol::ChatAgent::Codex => "codex",
                    };
                    let title = c.title.as_deref().unwrap_or("-");
                    println!("{:<18} {:<24} {:<8} {}", c.id, title, agent, c.updated_at);
                }
            }
        }
        Command::Revoke { device_id } => {
            let mut store = DeviceStore::open(&dir);
            if store.revoke(&device_id) {
                println!("revoked device {device_id}");
            } else {
                eprintln!("no such device: {device_id}");
                std::process::exit(1);
            }
        }
        Command::Accounts => {
            let store = AccountStore::open(&dir);
            let accounts = store.list();
            if accounts.is_empty() {
                println!("no accounts");
            } else {
                println!("{:<16} {:<20} {:<8} {:<8} STATUS", "SLUG", "LABEL", "KIND", "DEFAULT");
                for a in accounts {
                    let slot = store.slot_dir(&a.slug);
                    let status = match account_login::status(&a, &slot) {
                        AuthStatus::LoggedIn => "logged in",
                        AuthStatus::LoggedOut => "logged out",
                        AuthStatus::Unknown => "unknown",
                    };
                    let kind = match a.kind {
                        AccountKind::Claude => "claude",
                        AccountKind::Codex => "codex",
                    };
                    println!(
                        "{:<16} {:<20} {:<8} {:<8} {}",
                        a.slug, a.label, kind, a.is_default, status
                    );
                }
            }
        }
        Command::AccountAdd { slug, label, kind } => {
            let Some(kind) = parse_account_kind(&kind) else {
                eprintln!("unknown account kind: {kind:?} (expected claude|codex)");
                std::process::exit(1);
            };
            let store = AccountStore::open(&dir);
            match store.create_slot(&slug, &label, kind) {
                Ok(account) => {
                    let slot_dir = store.slot_dir(&account.slug);
                    println!("created account slot: {} ({})", account.slug, account.label);
                    match kind {
                        AccountKind::Claude => {
                            println!("next step, log this slot in — either:");
                            println!("  run: CLAUDE_CONFIG_DIR={} claude auth login", slot_dir.display());
                            println!("  or:  complete login via the app's account pairing flow");
                        }
                        AccountKind::Codex => {
                            println!("next step, log this slot in — either:");
                            println!("  run: CODEX_HOME={} codex login", slot_dir.display());
                            println!(
                                "  or:  POST an API key to /v1/accounts/{}/apikey via the app",
                                account.slug
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("failed to create account: {e}");
                    std::process::exit(1);
                }
            }
        }
        Command::Serve => {
            let cfg = GatewayConfig::load(&dir);
            let bind = cfg.bind.clone();
            let state = AppState::new(dir, cfg.clone());
            omega_gateway::events::spawn_background_emitters(state.events.clone(), &cfg);
            let app = build_router(state);
            let listener = tokio::net::TcpListener::bind(&bind).await?;
            tracing::info!("omega-gateway listening on {bind}");
            axum::serve(listener, app).await?;
        }
    }
    Ok(())
}

fn hostname_or_default() -> String {
    std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_account_kind_accepts_claude_and_codex_case_insensitively() {
        assert_eq!(parse_account_kind("claude"), Some(AccountKind::Claude));
        assert_eq!(parse_account_kind("Claude"), Some(AccountKind::Claude));
        assert_eq!(parse_account_kind("codex"), Some(AccountKind::Codex));
        assert_eq!(parse_account_kind("CODEX"), Some(AccountKind::Codex));
    }

    #[test]
    fn parse_account_kind_rejects_unknown() {
        assert_eq!(parse_account_kind("bogus"), None);
        assert_eq!(parse_account_kind(""), None);
    }

    #[test]
    fn cli_parses_accounts_subcommand() {
        let cli = Cli::try_parse_from(["omega-gatewayd", "accounts"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Accounts)));
    }

    #[test]
    fn cli_parses_account_add_subcommand_with_positional_args() {
        let cli =
            Cli::try_parse_from(["omega-gatewayd", "account-add", "work-a", "Work A", "claude"]).unwrap();
        match cli.command {
            Some(Command::AccountAdd { slug, label, kind }) => {
                assert_eq!(slug, "work-a");
                assert_eq!(label, "Work A");
                assert_eq!(kind, "claude");
            }
            other => panic!("expected AccountAdd, got {other:?}"),
        }
    }

    #[test]
    fn cli_rejects_account_add_missing_args() {
        assert!(Cli::try_parse_from(["omega-gatewayd", "account-add", "only-slug"]).is_err());
    }

    #[test]
    fn account_add_creates_slot_via_store() {
        // Exercises the same store call the CLI handler makes end to end
        // (no real login, no process spawn) — the CLI branch itself is a
        // thin println wrapper around AccountStore::create_slot, already
        // covered exhaustively in accounts.rs.
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        let kind = parse_account_kind("claude").unwrap();
        let account = store.create_slot("work-a", "Work A", kind).unwrap();
        assert_eq!(account.slug, "work-a");
        assert_eq!(account.kind, AccountKind::Claude);
        assert!(account.is_default);
    }
}
