use crate::{
    state::paths::get_paths,
    types::{exchange::Pair, path::Path},
};
use cosmwasm_std::{Decimal256, Deps, QuerierWrapper, StdResult};
use fin_helpers::queries::{query_base_price, query_quote_price};

pub fn get_paths_handler(deps: Deps, from: &str, to: &str) -> StdResult<Vec<Path>> {
    Ok(get_paths(deps.storage, from, to)?
        .iter()
        .map(|exchanges| Path {
            cost: exchanges
                .iter()
                .map(|exchange| get_price(deps.querier, exchange, from))
                .flatten()
                .sum(),
            exchanges: exchanges.clone(),
        })
        .collect::<Vec<Path>>())
}

pub fn get_price(querier: QuerierWrapper, exchange: &Pair, from: &str) -> StdResult<Decimal256> {
    Ok(match exchange {
        Pair::Fin {
            address,
            base_denom,
            ..
        } => match base_denom == from {
            true => query_quote_price(querier, address.clone()),
            false => query_base_price(querier, address.clone()),
        },
    })
}
