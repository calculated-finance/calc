use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct Pool {
    pub address: Addr,
    pub denoms: Vec<String>,
}
