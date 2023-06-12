use cosmwasm_std::{DepsMut, MessageInfo, Response};

use crate::{types::pair::Pair, ContractError};

pub fn create_pairs_handler(
    _deps: DepsMut,
    _info: MessageInfo,
    _pairs: Vec<Pair>,
) -> Result<Response, ContractError> {
    unimplemented!();
}

#[cfg(test)]
mod create_pairs_tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr,
    };

    use crate::{
        contract::instantiate,
        handlers::create_pairs::create_pairs_handler,
        msg::InstantiateMsg,
        state::pairs::{find_pair, save_pair},
        tests::constants::ADMIN,
        types::pair::Pair,
        ContractError,
    };

    #[test]
    fn with_non_admin_sender_fails() {
        let mut deps = mock_dependencies();

        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info(ADMIN, &[]),
            InstantiateMsg {
                admin: Addr::unchecked(ADMIN),
            },
        )
        .unwrap();

        assert_eq!(
            create_pairs_handler(deps.as_mut(), mock_info("not-admin", &[]), vec![]).unwrap_err(),
            ContractError::Unauthorized {}
        )
    }

    #[test]
    fn overwrites_existing_pair() {
        let mut deps = mock_dependencies();

        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info(ADMIN, &[]),
            InstantiateMsg {
                admin: Addr::unchecked(ADMIN),
            },
        )
        .unwrap();

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let new_route = vec![167, 2];

        create_pairs_handler(
            deps.as_mut(),
            mock_info(ADMIN, &[]),
            vec![Pair {
                route: new_route.clone(),
                ..pair.clone()
            }],
        )
        .unwrap();

        let updated_pair = find_pair(deps.as_ref().storage, pair.denoms()).unwrap();

        assert_ne!(pair.route, updated_pair.route);
        assert_eq!(updated_pair.route, new_route);
    }
}