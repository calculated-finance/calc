use crate::{msg::TriggerIdResponse, state::triggers::get_trigger_by_order_idx};
#[cfg(not(feature = "library"))]
use cosmwasm_std::Uint128;
use cosmwasm_std::{Deps, StdError, StdResult};

pub fn get_trigger_id_by_fin_limit_order_idx(
    deps: Deps,
    order_idx: Uint128,
) -> StdResult<TriggerIdResponse> {
    let trigger = get_trigger_by_order_idx(deps.storage, order_idx)?;

    if let Some(trigger) = trigger {
        Ok(TriggerIdResponse {
            trigger_id: trigger.vault_id,
        })
    } else {
        Err(StdError::not_found("Trigger"))
    }
}

#[cfg(test)]
mod get_trigger_id_by_fin_limit_order_idx_tests {
    use super::get_trigger_id_by_fin_limit_order_idx;
    use crate::{
        state::triggers::save_trigger,
        types::trigger::{Trigger, TriggerConfiguration},
    };
    use cosmwasm_std::{testing::mock_dependencies, Decimal, Uint128};

    #[test]
    fn should_return_not_found_when_no_trigger_exists() {
        let err =
            get_trigger_id_by_fin_limit_order_idx(mock_dependencies().as_ref(), Uint128::one())
                .unwrap_err();

        assert_eq!(err.to_string(), "Trigger not found");
    }

    #[test]
    fn should_return_not_found_when_trigger_has_no_order_idx() {
        let mut deps = mock_dependencies();

        let order_idx = Uint128::new(12);
        let vault_id = Uint128::one();

        save_trigger(
            deps.as_mut().storage,
            Trigger {
                vault_id,
                configuration: TriggerConfiguration::FinLimitOrder {
                    target_price: Decimal::percent(120),
                    order_idx: None,
                },
            },
        )
        .unwrap();

        let err = get_trigger_id_by_fin_limit_order_idx(mock_dependencies().as_ref(), order_idx)
            .unwrap_err();

        assert_eq!(err.to_string(), "Trigger not found");
    }

    #[test]
    fn should_return_trigger_id() {
        let mut deps = mock_dependencies();

        let order_idx = Uint128::new(12);
        let vault_id = Uint128::one();

        save_trigger(
            deps.as_mut().storage,
            Trigger {
                vault_id,
                configuration: TriggerConfiguration::FinLimitOrder {
                    target_price: Decimal::percent(120),
                    order_idx: Some(order_idx),
                },
            },
        )
        .unwrap();

        let response = get_trigger_id_by_fin_limit_order_idx(deps.as_ref(), order_idx).unwrap();

        assert_eq!(response.trigger_id, vault_id);
    }
}
