use crate::{
    state::paths::get_paths,
    types::exchange::{UnweightedExchange, WeightedExchange},
};
use cosmwasm_std::{Decimal256, Deps, QuerierWrapper, StdResult};
use fin_helpers::queries::{query_base_price, query_quote_price};

pub fn get_paths_handler(
    deps: Deps,
    from: &str,
    to: &str,
) -> StdResult<Vec<Vec<WeightedExchange>>> {
    Ok(get_paths(deps.storage, from, to)?
        .iter()
        .map(|path| {
            path.iter()
                .map(|exchange| -> StdResult<WeightedExchange> {
                    Ok(WeightedExchange {
                        exchange: exchange.clone(),
                        price: get_price(deps.querier, exchange, from)?,
                    })
                })
                .flatten()
                .collect::<Vec<WeightedExchange>>()
        })
        .collect())
}

pub fn get_price(
    querier: QuerierWrapper,
    exchange: &UnweightedExchange,
    from: &str,
) -> StdResult<Decimal256> {
    Ok(match exchange {
        UnweightedExchange::Fin {
            address,
            base_denom,
            ..
        } => match base_denom == from {
            true => query_quote_price(querier, address.clone()),
            false => query_base_price(querier, address.clone()),
        },
    })
}
