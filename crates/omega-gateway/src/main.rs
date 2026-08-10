use clap::{Parser, Subcommand};
use omega_gateway::auth::PairingCode;
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
        Command::Serve => {
            let cfg = GatewayConfig::load(&dir);
            let bind = cfg.bind.clone();
            let app = build_router(AppState { dir, cfg });
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
