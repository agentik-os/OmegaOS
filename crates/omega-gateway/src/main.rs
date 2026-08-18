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
            let cfg = GatewayConfig::load(&dir);
            let host = reachable_host_url(&cfg.bind);
            let port_hint = cfg.bind.rsplit(':').next().unwrap_or("4477");
            let payload = format!("omega://pair?host={host}&code={}", pc.code);
            qr2term::print_qr(&payload).ok();
            println!();
            println!("Pairing code: {}  (valid 5 minutes)", pc.code);
            println!("Machine:      {host}");
            println!();
            println!("Paste this whole line into the app, or scan the QR:");
            println!("{payload}");
            if is_loopback_only(&cfg.bind) && !host.starts_with("https://") {
                println!();
                println!("NOTE: this gateway listens on {} — loopback only.", cfg.bind);
                println!("      Only an app on THIS machine can pair with it. To pair from");
                println!("      another device, set bind = \"0.0.0.0:{port_hint}\" in");
                println!("      {}/gateway.toml and restart it.", dir.display());
            }
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

/// The address another device can actually reach this gateway on, as a full
/// URL.
///
/// This used to return /etc/hostname: a bare name with no scheme and no port,
/// which the apps could not use and no other machine could resolve. The pairing
/// payload (and its QR) is meant to be the WHOLE answer, so it has to carry an
/// address that works from the device doing the pairing.
///
/// Preference order, most-reachable first: the tailnet address (works from
/// anywhere on the tailnet, which is how these machines are actually reached),
/// then the LAN address of the default route, then loopback for a same-machine
/// pairing.
fn reachable_host_url(bind: &str) -> String {
    let port = bind.rsplit(':').next().and_then(|p| p.parse::<u16>().ok()).unwrap_or(4477);

    // The address has to be one THIS gateway serves on, which is decided by the
    // bind and nothing else. Reading the machine's interfaces instead was worse
    // than the bare hostname it replaced: with the default loopback bind it
    // advertised a tailnet address that another process answers (and 400s),
    // so pairing failed with a plausible-looking address instead of an
    // obviously wrong one.
    // A `tailscale serve` mapping in front of this port is the best answer
    // available: it reaches other devices over TLS on the tailnet while the
    // gateway itself stays on loopback, so nothing is exposed to the public
    // interface this machine may also have.
    if let Some(url) = tailscale_serve_url(port) {
        return url;
    }

    if !binds_all_interfaces(bind) {
        // A specific bind: advertise exactly it. Loopback means same machine
        // only, which the caller states plainly rather than papering over.
        let host = bind.rsplit_once(':').map(|(h, _)| h).unwrap_or("127.0.0.1");
        let host = host.trim_matches(|c| c == '[' || c == ']');
        return format!("http://{host}:{port}");
    }

    // Bound to every interface, so a remote address genuinely reaches us.
    if let Some(ip) = tailscale_ipv4() {
        return format!("http://{ip}:{port}");
    }
    if let Some(ip) = lan_ipv4() {
        return format!("http://{ip}:{port}");
    }
    format!("http://127.0.0.1:{port}")
}

/// Whether the bind accepts connections on every interface (`0.0.0.0`, `::`,
/// or a bare `:port`), which is the only case where an address other than the
/// bind's own can be honestly advertised.
fn binds_all_interfaces(bind: &str) -> bool {
    let host = bind.rsplit_once(':').map(|(h, _)| h).unwrap_or("");
    let host = host.trim_matches(|c| c == '[' || c == ']');
    host.is_empty() || host == "0.0.0.0" || host == "::" || host == "*"
}

/// True when the gateway only answers on loopback, so no other device can pair
/// with it whatever address the payload carries.
fn is_loopback_only(bind: &str) -> bool {
    let host = bind.rsplit_once(':').map(|(h, _)| h).unwrap_or("");
    let host = host.trim_matches(|c| c == '[' || c == ']');
    host.starts_with("127.") || host == "localhost" || host == "::1"
}

/// The `https://<node>.<tailnet>:<port>` URL when `tailscale serve` already
/// fronts this port. Parsed from `tailscale serve status`, whose listing pairs
/// each served URL with the local target it proxies to.
fn tailscale_serve_url(port: u16) -> Option<String> {
    let out = std::process::Command::new("tailscale").args(["serve", "status"]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut current: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("https://") {
            // "https://host:port (tailnet only)" — keep the URL itself.
            current = trimmed.split_whitespace().next().map(|s| s.trim_end_matches('/').to_string());
            continue;
        }
        // A mapping line points at the local target, e.g. "|-- / proxy http://127.0.0.1:4477".
        if trimmed.starts_with("|--") && trimmed.contains(&format!("127.0.0.1:{port}")) {
            if let Some(url) = &current {
                // Only a root mapping serves the whole API surface.
                if trimmed.split_whitespace().nth(1) == Some("/") {
                    return Some(url.clone());
                }
            }
        }
    }
    None
}

fn tailscale_ipv4() -> Option<String> {
    let out = std::process::Command::new("tailscale").args(["ip", "-4"]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let ip = String::from_utf8_lossy(&out.stdout).lines().next()?.trim().to_string();
    if ip.is_empty() { None } else { Some(ip) }
}

/// The local address the OS would use to reach the outside world. Opening a UDP
/// socket and reading its local address sends no traffic; it just asks the
/// routing table which interface would be chosen.
fn lan_ipv4() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let addr = socket.local_addr().ok()?;
    let ip = addr.ip().to_string();
    if ip.starts_with("127.") { None } else { Some(ip) }
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
