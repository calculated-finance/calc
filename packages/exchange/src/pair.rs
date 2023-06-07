use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Pair {
    pub denoms: [String; 2],
    pub decimal_delta: i8,
    pub price_precision: u8,
}

impl Pair {
    pub fn other_denom(self, denom: String) -> String {
        if self.denoms[0] == denom {
            self.denoms[1].clone()
        } else {
            self.denoms[0].clone()
        }
    }
}

impl Default for Pair {
    fn default() -> Self {
        Pair {
            denoms: ["uusd".to_string(), "uatom".to_string()],
            decimal_delta: 0,
            price_precision: 3,
        }
    }
}
