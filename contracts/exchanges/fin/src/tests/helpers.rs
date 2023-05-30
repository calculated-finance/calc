use cosmwasm_std::Addr;

use crate::types::pair::Pair;

impl Default for Pair {
    fn default() -> Self {
        Pair {
            base_denom: "base_denom".to_string(),
            quote_denom: "quote_denom".to_string(),
            address: Addr::unchecked("pair-address"),
        }
    }
}
