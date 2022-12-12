use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct DelegationPacket {
    pub validator_address: String
}