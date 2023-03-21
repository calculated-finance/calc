use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {}

// https://github.com/mars-protocol/red-bank/blob/master/packages/types/src/red_bank/msg.rs

#[cw_serde]
pub enum RedBankExecuteMsg {
    Deposit {
        /// Address that will receive the coins
        on_behalf_of: Option<String>,
    },

    /// Withdraw native coins
    Withdraw {
        /// Asset to withdraw
        denom: String,
        /// Amount to be withdrawn. If None is specified, the full amount will be withdrawn.
        amount: Option<Uint128>,
        /// The address where the withdrawn amount is sent
        recipient: Option<String>,
    },
}

// https://github.com/mars-protocol/red-bank/blob/master/packages/types/src/incentives.rs

#[cw_serde]
pub enum IncentivesExecuteMsg {
    /// Claim rewards. MARS rewards accrued by the user will be staked into xMARS before
    /// being sent.
    ClaimRewards {},
}


#[cw_serde]
#[derive(QueryResponses)]
pub enum RedBankQueryMsg {
    #[returns(UserCollateralResponse)]
    UserCollateral {
        user: String,
        denom: String,
    },
}

#[cw_serde]
pub struct UserCollateralResponse {
    /// Asset denom
    pub denom: String,
    /// Scaled collateral amount stored in contract state
    pub amount_scaled: Uint128,
    /// Underlying asset amount that is actually deposited at the current block
    pub amount: Uint128,
    /// Wether the user is using asset as collateral or not
    pub enabled: bool,
}