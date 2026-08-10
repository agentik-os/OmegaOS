use clap::{Parser, Subcommand};
use omega_gateway::auth::{DeviceStore, PairingCode};
use omega_gateway::chat_store::ChatStore;
use omega_gateway::config::{gateway_dir, GatewayConfig};
use omega_gateway::server::{build_router, AppState};

#[derive(Parser)]
#[command(name = "omega-gatewayd", about = "OmegaOS gateway daemon")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
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
