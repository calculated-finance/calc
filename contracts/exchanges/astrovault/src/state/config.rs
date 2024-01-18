use crate::types::config::{Config, RouterConfig};
use cosmwasm_std::{QuerierWrapper, StdResult, Storage};
use cw_storage_plus::Item;

#[cfg(target_arch = "wasm32")]
use astrovault::router::query_msg as RouterQuery;

const CONFIG: Item<Config> = Item::new("config_v2");
const ROUTER_CONFIG: Item<RouterConfig> = Item::new("rc_v2");


pub fn get_config(store: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(store)
}

pub fn update_config(store: &mut dyn Storage, config: Config) -> StdResult<Config> {
    CONFIG.save(store, &config)?;
    Ok(config)
}


pub fn get_router_config(
    storage: &dyn Storage,
) -> StdResult<RouterConfig> {
    ROUTER_CONFIG.load(storage)
}


#[cfg(target_arch = "wasm32")]
pub fn update_router_config(
    querier: &QuerierWrapper,
    storage: &mut dyn Storage,
    router: &str,
) -> StdResult<()> {

    #[cfg(target_arch = "wasm32")]
    let res = querier.query_wasm_smart::<RouterQuery::ConfigResponse>(
        router, 
        &RouterQuery::QueryMsg::Config {}
    )?;

    let res = from_json(String::from(""))?;


    ROUTER_CONFIG.save(storage, &res)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn update_router_config(
    _: &QuerierWrapper,
    _: &mut dyn Storage,
    _: &str,
) -> StdResult<()> {
    Ok(())
}