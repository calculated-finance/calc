use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub enum UnweightedExchange {
    Fin {
        address: Addr,
        quote_denom: String,
        base_denom: String,
    },
}