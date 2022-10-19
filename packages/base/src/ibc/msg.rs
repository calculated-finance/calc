use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum CalcIBC {
    Test { value: String },
}

#[cw_serde]
pub struct TestResponse {
    pub value: String,
}
