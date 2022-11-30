use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum CalcIBC {
    Test { value: String }
}

#[cw_serde]
pub enum KCalc {
    TestResponse { value: String }
}