use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub enum IbcMessage {
    Delegate {
        delegator_address: String,
        validator_address: String,
        amount: Uint128
    }
}

#[cw_serde]
pub enum IbcAcks {
    Delegate {}
}
