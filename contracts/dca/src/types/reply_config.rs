use cosmwasm_schema::cw_serde;
use cosmwasm_std::ReplyOn;

#[cw_serde]
pub struct ReplyConfig {
    pub id: u64,
    pub on: ReplyOn,
}
