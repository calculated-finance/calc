#![allow(unused_variables, unused_imports)]

use crate::types::{pair::Pair, position_type::PositionType};
use astrovault::assets::asset::AssetInfo;
use cosmwasm_std::{from_json, Deps, QuerierWrapper, StdError, StdResult};



pub fn query_route_exist(
    deps: Deps,
    pair: &Pair
) -> StdResult<bool> {
    let pools = pair.route_pools();
    

    Ok(true)
}



pub fn valid_route(
    querier: &QuerierWrapper,
    pair: &Pair,
) -> StdResult<()> {
    let _route = pair.route();

    Ok(())
}

pub fn get_token_out_denom(
    querier: &QuerierWrapper,
    token_in_denom: String,
    pool_id: u64,
    next_pool_id: u64,
) -> StdResult<String> {
    todo!()
}


pub fn get_pool_assets(querier: &QuerierWrapper, pool_id: u64) -> Result<Vec<String>, StdError> {
    todo!()
}

pub fn calculate_route(
    querier: &QuerierWrapper,
    pair: &Pair,
    swap_denom: String,
) -> StdResult<Vec<()>> {
    todo!()
}


/* 
#[cfg(test)]
mod get_token_out_denom_tests {

    #[test]
    fn when_swap_denom_not_in_pair_denoms_fails() {
        todo!()
    }
}

#[cfg(test)]
mod calculate_route_tests {
    use super::calculate_route;

    #[test]
    fn when_swap_denom_not_in_pair_denoms_fails() {
        
        todo!()

    }

    #[test]
    fn when_initial_pool_does_not_contain_swap_denom_fails() {
        
        todo!()

    }

    #[test]
    fn when_intermediary_pool_does_not_contain_target_denom_fails() {
        
        todo!()
    }

    #[test]
    fn when_final_pool_does_not_contain_target_denom_fails() {
        
        todo!()

    }

    #[test]
    fn calculates_1_pool_route() {
        
        todo!()

    }

    #[test]
    fn calculates_2_pool_route() {
        
        todo!()

    }

    #[test]
    fn calculates_3_pool_route() {
        
        todo!()

    }

    #[test]
    fn with_cl_pools_when_swap_denom_not_in_pair_denoms_fails() {
        
        todo!()

    }

    #[test]
    fn with_cl_pools_when_initial_pool_does_not_contain_swap_denom_fails() {
        
        todo!()

    }

    #[test]
    fn with_cl_pools_when_intermediary_pool_does_not_contain_target_denom_fails() {
        
        todo!()

    }

    #[test]
    fn with_cl_pools_when_final_pool_does_not_contain_target_denom_fails() {
        
        todo!()

    }

    #[test]
    fn with_cl_pools_calculates_1_pool_route() {
        
        todo!()

    }

    #[test]
    fn with_cl_pools_calculates_2_pool_route() {
        todo!()
    }

    #[test]
    fn with_cl_pools_calculates_3_pool_route() {
        
        todo!()
    }

    #[test]
    fn with_ss_pools_when_swap_denom_not_in_pair_denoms_fails() {
        
        todo!()

    }

    #[test]
    fn with_ss_pools_when_initial_pool_does_not_contain_swap_denom_fails() {
        
        todo!()
    }

    #[test]
    fn with_ss_pools_when_intermediary_pool_does_not_contain_target_denom_fails() {
        
        todo!()
    }

    #[test]
    fn with_ss_pools_when_final_pool_does_not_contain_target_denom_fails() {
        
        todo!()
    }

    #[test]
    fn with_ss_pools_calculates_1_pool_route() {
        
        todo!()

    }

    #[test]
    fn with_ss_pools_calculates_2_pool_route() {
        
        todo!()

    }

    #[test]
    fn with_ss_pools_calculates_3_pool_route() {
        
        todo!()
    }
}
 */