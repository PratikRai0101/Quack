use clap::Parser;
use dotenvy::dotenv;
use std::env;

mod groq;
mod tui;
mod context;
mod shell;

#[derive(Parser)]
struct Args {
    /// Command to replay
    #[arg(long)]
    cmd: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let args = Args::parse();
    let _api_key = env::var("GROQ_API_KEY").ok();

    // Minimal behaviour: print the command and exit.
    if let Some(cmd) = args.cmd {
        println!("Would replay command: {}", cmd);
    } else {
        println!("Quack CLI — provide --cmd to replay a failing command");
    }

    Ok(())
}
