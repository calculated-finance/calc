use cosmwasm_std::{testing::mock_dependencies, Decimal};
use std::str::FromStr;

use crate::{
    handlers::save_buy_adjustments_handler::save_buy_adjustments_handler,
    state::buy_adjustments::get_buy_adjustment,
};

#[test]
fn saves_buy_adjustments() {
    let adjustments = vec![
        (30, Decimal::from_str("0.921").unwrap()),
        (35, Decimal::from_str("0.926").unwrap()),
        (40, Decimal::from_str("0.931").unwrap()),
        (45, Decimal::from_str("0.936").unwrap()),
        (50, Decimal::from_str("0.941").unwrap()),
        (55, Decimal::from_str("0.946").unwrap()),
        (60, Decimal::from_str("0.951").unwrap()),
        (70, Decimal::from_str("0.961").unwrap()),
        (80, Decimal::from_str("0.971").unwrap()),
        (90, Decimal::from_str("0.981").unwrap()),
    ];

    let mut deps = mock_dependencies();
    save_buy_adjustments_handler(deps.as_mut(), adjustments.clone()).unwrap();

    adjustments.iter().for_each(|(model, adjustment)| {
        let stored_adjustment = get_buy_adjustment(deps.as_ref().storage, *model).unwrap();
        assert_eq!(stored_adjustment, *adjustment);
    })
}

#[test]
fn overwrites_buy_adjustments() {
    let old_adjustments = vec![
        (30, Decimal::from_str("0.921").unwrap()),
        (35, Decimal::from_str("0.926").unwrap()),
        (40, Decimal::from_str("0.931").unwrap()),
        (45, Decimal::from_str("0.936").unwrap()),
        (50, Decimal::from_str("0.941").unwrap()),
        (55, Decimal::from_str("0.946").unwrap()),
        (60, Decimal::from_str("0.951").unwrap()),
        (70, Decimal::from_str("0.961").unwrap()),
        (80, Decimal::from_str("0.971").unwrap()),
        (90, Decimal::from_str("0.981").unwrap()),
    ];

    let mut deps = mock_dependencies();
    save_buy_adjustments_handler(deps.as_mut(), old_adjustments.clone()).unwrap();

    let new_adjustments = vec![
        (30, Decimal::from_str("1.921").unwrap()),
        (35, Decimal::from_str("1.926").unwrap()),
        (40, Decimal::from_str("1.931").unwrap()),
        (45, Decimal::from_str("1.936").unwrap()),
        (50, Decimal::from_str("1.941").unwrap()),
        (55, Decimal::from_str("1.946").unwrap()),
        (60, Decimal::from_str("1.951").unwrap()),
        (70, Decimal::from_str("1.961").unwrap()),
        (80, Decimal::from_str("1.971").unwrap()),
        (90, Decimal::from_str("1.981").unwrap()),
    ];

    save_buy_adjustments_handler(deps.as_mut(), new_adjustments.clone()).unwrap();

    new_adjustments.iter().zip(old_adjustments.iter()).for_each(
        |((model, new_adjustment), (_, old_adjustment))| {
            let stored_adjustment = get_buy_adjustment(deps.as_ref().storage, *model).unwrap();
            assert_eq!(stored_adjustment, *new_adjustment);
            assert_ne!(stored_adjustment, *old_adjustment);
        },
    )
}
