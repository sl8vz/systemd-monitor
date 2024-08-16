use super::proxy::manager::ManagerProxy;
use super::proxy::unit::UnitProxy;

use anyhow::Result;
use derive_more::{Display, FromStr};
use futures::StreamExt;
use tokio::task::JoinSet;
use zbus::Connection;

#[derive(Clone)]
pub struct Unit {
    name: String,
    path: String,
}

#[derive(Debug, Display, FromStr)]
enum ActiveState {
    Active,
    Reloading,
    Inactive,
    Failed,
    Activating,
    Deactivating,
}

#[derive(Debug, Display, FromStr)]
enum LoadState {
    Loaded,
    Error,
    Masked,
}

pub async fn get_all_units(conn: &Connection) -> Result<Vec<Unit>> {
    let proxy = ManagerProxy::new(conn).await?;
    let units: Vec<Unit> = proxy
        .list_units().await?
        .into_iter()
        .map(|unit| Unit {
            name: unit.0,
            path: unit.6.to_string(),
        })
        .collect();
    Ok(units)
}

pub async fn get_units_filtered(conn: &Connection, unit_list: &[&str]) -> Result<Vec<Unit>> {
    let proxy = ManagerProxy::new(conn).await?;
    let units = proxy.list_units_by_names(unit_list).await?;
    let units: Vec<Unit> = units
        .into_iter()
        .map(|unit| Unit {
            name: unit.0,
            path: unit.6.to_string(),
        })
        .collect();
    Ok(units)
}

pub async fn watch_unit(conn: Connection, unit: Unit) -> Result<()> {
    let proxy = UnitProxy::builder(&conn)
        .path(unit.path.as_str())?
        .build()
        .await?;

    tokio::join!(
        async {
            let mut stream = proxy.receive_active_state_changed().await;
            while let Some(state) = stream.next().await {
                if let Ok(new_state) = state.get().await {
                    if let Ok(state) = new_state.parse::<ActiveState>() {
                        println!("Unit: {} Active state: {}", unit.name, state);
                    }
                }
            }
        },
        async {
            let mut stream = proxy.receive_load_state_changed().await;
            while let Some(state) = stream.next().await {
                if let Ok(new_state) = state.get().await {
                    if let Ok(state) = new_state.parse::<LoadState>() {
                        println!("Unit: {} Load state: {}", unit.name, state);
                    }
                }
            }
        }
    );
    Ok(())
}

pub async fn watch_units(conn: Connection, units: Vec<Unit>) -> Result<()> {
    let mut watchers = JoinSet::new();

    for unit in units {
        let conn = conn.clone();
        watchers.spawn(async move {
            watch_unit(conn, unit.clone()).await.unwrap();
        });
    }
    while (watchers.join_next().await).is_some() {}

    Ok(())
}
