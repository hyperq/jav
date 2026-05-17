#![allow(dead_code)]

mod cloud115;
mod scraper;
mod store;
mod tui;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "jav", about = "JAV magnet TUI")]
struct Args {
    #[arg(short, long, default_value = "https://www.javbus.com")]
    base: String,

    #[arg(short, long)]
    proxy: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let client = scraper::JavClient::new(&args.base, args.proxy.as_deref())?;

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let db_path = format!("{home}/.jav/data.db");
    let store = store::Store::new(&db_path)?;

    tui::run(client, store).await
}
