use cosmwasm_std::{Decimal, QuerierWrapper, StdResult};

use osmosis_std::types::osmosis::gamm::v2::QuerySpotPriceRequest;

use crate::{pool::Pool, position_type::PositionType};

pub fn query_quote_price(
    querier: QuerierWrapper,
    pool: &Pool,
    swap_denom: &str,
) -> StdResult<Decimal> {
    let position_type = match swap_denom == pool.quote_denom {
        true => PositionType::Enter,
        false => PositionType::Exit,
    };

    // check this logic
    let base_asset_denom;
    let quote_asset_denom;

    match position_type {
        PositionType::Enter => {
            base_asset_denom = pool.base_denom.clone();
            quote_asset_denom = pool.quote_denom.clone();
        }
        PositionType::Exit => {
            base_asset_denom = pool.quote_denom.clone();
            quote_asset_denom = pool.base_denom.clone();
        }
    }

    QuerySpotPriceRequest {
        pool_id: pool.pool_id,
        base_asset_denom,
        quote_asset_denom,
    }
    .query(&querier)
    .unwrap()
    .spot_price
    .parse::<Decimal>()
}
