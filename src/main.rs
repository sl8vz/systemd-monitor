mod proxy;
mod systemd;

use anyhow::Result;
use clap::Parser;
use zbus::Connection;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    /// Unit list to monitor
    #[arg(short = 'u', long, value_parser, value_name = "Unit list to monitor")]
    pub units: Option<Vec<String>>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let conn = Connection::system().await?;
    let units = if let Some(unit_list) = Cli::parse().units {
        systemd::get_units_filtered(
            &conn,
            unit_list
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .as_slice(),
        )
        .await?
    } else {
        systemd::get_all_units(&conn).await?
    };
    systemd::watch_units(conn, units).await?;
    Ok(())
}
