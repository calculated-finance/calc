use crate::{
    contract::{AFTER_SWAPPING_FOR_BOW_DEPOSIT, AFTER_UNSTAKING_FROM_BOW},
    msg::ExecuteMsg,
    state::{cache::BOW_CACHE, config::get_config, pairs::find_pair, pools::get_pool},
    types::{reply_config::ReplyConfig, vault::Vault},
};
use bow_helpers::{msg::BowStakingExecuteMsg, queries::query_pool_balances};
use cosmos_sdk_proto::{
    cosmos::authz::v1beta1::MsgExec, cosmwasm::wasm::v1::MsgExecuteContract, traits::Message, Any,
};
use cosmwasm_std::{
    to_binary, Addr, Binary, Coin, CosmosMsg, Decimal256, DepsMut, Env, StdError, StdResult,
    SubMsg, Uint128, WasmMsg,
};

pub fn execute_trigger_message(env: Env, trigger_id: Uint128) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::ExecuteTrigger { trigger_id }).unwrap(),
        funds: vec![],
    })
}

pub fn swap_for_bow_deposit_messages(
    deps: &mut DepsMut,
    env: &Env,
    pool_address: &Addr,
    amount: Coin,
    slippage_tolerance: Option<Decimal256>,
) -> StdResult<Vec<CosmosMsg>> {
    let mut cache = BOW_CACHE.load(deps.storage)?;
    let pool_response = get_pool(deps.storage, pool_address)?;

    if pool_response.is_none() {
        return Err(StdError::GenericErr {
            msg: format!("BOW pool {} does not exist", pool_address),
        });
    }

    let denoms = pool_response.unwrap().denoms;
    let pool_balances = query_pool_balances(deps.querier, pool_address)?.balances;

    let pool_balances_with_denoms: Vec<(&String, Uint128)> =
        denoms.iter().zip(pool_balances).collect();

    let pool_total = pool_balances.iter().sum::<Uint128>();

    let swap_messages = pool_balances_with_denoms
        .into_iter()
        .map(|(denom, pool_balance)| {
            let amount = Coin::new(
                amount
                    .amount
                    .checked_multiply_ratio(pool_balance, pool_total)
                    .expect("amount to swap for bow deposit")
                    .into(),
                amount.denom.clone(),
            );

            let denoms = [amount.denom.clone(), denom.to_string()];

            if denoms[0] == denoms[1] {
                cache.deposit.push(amount);

                return None;
            }

            let pair = find_pair(deps.storage, denoms.clone())
                .expect(&format!("Pair for denoms {:?}", denoms));

            Some(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::Swap {
                    pair_address: pair.address,
                    slippage_tolerance,
                    reply_config: Some(ReplyConfig {
                        id: AFTER_SWAPPING_FOR_BOW_DEPOSIT,
                        on: cosmwasm_std::ReplyOn::Success,
                    }),
                })
                .expect("swap on fin message"),
                funds: vec![amount],
            }))
        })
        .flat_map(|msg| msg)
        .collect::<Vec<CosmosMsg>>();

    BOW_CACHE.save(deps.storage, &cache)?;

    Ok(swap_messages)
}

pub fn unstake_from_bow_message(deps: &mut DepsMut, env: &Env, vault: &Vault) -> StdResult<SubMsg> {
    let config = get_config(deps.storage)?;

    let mut buffer = vec![];

    MsgExecuteContract {
        contract: config.bow_staking_address.to_string(),
        sender: vault.owner.to_string(),
        msg: to_binary(&BowStakingExecuteMsg::Withdraw {
            amount: vault.balance.clone(),
        })
        .unwrap()
        .to_vec(),
        funds: vec![],
    }
    .encode(&mut buffer)
    .unwrap();

    Ok(SubMsg::reply_on_success(
        CosmosMsg::Stargate {
            type_url: "/cosmos.authz.v1beta1.MsgExec".to_string(),
            value: Binary(
                MsgExec {
                    grantee: env.contract.address.to_string(),
                    msgs: vec![Any {
                        type_url: "/cosmwasm.wasm.v1.MsgExecuteContract".to_string(),
                        value: buffer,
                    }],
                }
                .encode_to_vec(),
            ),
        },
        AFTER_UNSTAKING_FROM_BOW,
    ))
}
