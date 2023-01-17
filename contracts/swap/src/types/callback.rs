use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;

#[cw_serde]
pub struct Callback {
    pub contract_addr: String,
    pub msg: Binary,
}
