use cosmwasm_std::{Attribute, Event};

use crate::error::ContractError;

pub fn find_first_event_by_type(
    events: &Vec<Event>,
    target_type: String,
) -> Result<&Event, ContractError> {
    return events
        .iter()
        .find(|event| event.ty == target_type)
        .ok_or_else(|| ContractError::CustomError {
            val: format!("could not find event with type: {}", target_type),
        });
}

pub fn find_first_attribute_by_key(
    attributes: &Vec<Attribute>,
    target_key: String,
) -> Result<&Attribute, ContractError> {
    return attributes
        .iter()
        .find(|attribute| attribute.key == target_key)
        .ok_or_else(|| ContractError::CustomError {
            val: format!("could not find attribute with key: {}", target_key),
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_first_event_by_type_finds_event_successfully() {
        let mock_event = Event::new("mock-event");
        let mock_wasm_trade_event = Event::new("wasm-trade");
        let events = vec![mock_event, mock_wasm_trade_event];
        let result = find_first_event_by_type(&events, String::from("wasm-trade")).unwrap();

        assert_eq!(result.ty, "wasm-trade");
    }

    #[test]
    fn find_first_event_by_type_given_two_matching_events_finds_first_event_successfully() {
        let mock_wasm_trade_event_one = Event::new("wasm-trade").add_attribute("index", "1");

        let mock_wasm_trade_event_two = Event::new("wasm-trade").add_attribute("index", "2");

        let events = vec![mock_wasm_trade_event_one, mock_wasm_trade_event_two];
        let result = find_first_event_by_type(&events, String::from("wasm-trade")).unwrap();

        assert_eq!(result.ty, "wasm-trade");

        assert_eq!(result.attributes[0].value, "1")
    }

    #[test]
    fn find_first_event_by_type_given_no_matching_events_should_fail() {
        let mock_wasm_trade_event = vec![Event::new("not-wasm-trade")];
        let result = find_first_event_by_type(&mock_wasm_trade_event, String::from("wasm-trade"))
            .unwrap_err();

        assert_eq!(
            result.to_string(),
            "Custom Error val: \"could not find event with type: wasm-trade\""
        );
    }

    #[test]
    fn find_first_event_by_type_given_no_events_should_fail() {
        let empty: Vec<Event> = Vec::new();
        let result = find_first_event_by_type(&empty, String::from("wasm-trade")).unwrap_err();

        assert_eq!(
            result.to_string(),
            "Custom Error val: \"could not find event with type: wasm-trade\""
        );
    }

    #[test]
    fn find_first_attribute_by_key_finds_attribute_successfully() {
        let mock_attribute_one = Attribute::new("test-one", "value");
        let mock_attribute_two = Attribute::new("test-two", "value");
        let attributes = vec![mock_attribute_one, mock_attribute_two];
        let result = find_first_attribute_by_key(&attributes, String::from("test-one")).unwrap();

        assert_eq!(result.key, "test-one");
    }

    #[test]
    fn find_first_attribute_by_key_given_two_matching_attributes_finds_first_attribute_successfully(
    ) {
        let mock_attribute_one = Attribute::new("test", "1");
        let mock_attribute_two = Attribute::new("test", "2");
        let attributes = vec![mock_attribute_one, mock_attribute_two];
        let result = find_first_attribute_by_key(&attributes, String::from("test")).unwrap();

        assert_eq!(result.key, "test");

        assert_eq!(result.value, "1");
    }

    #[test]
    fn find_first_attribute_by_key_given_no_matching_attributes_should_fail() {
        let mock_attribute_one = Attribute::new("mock", "value");
        let attributes = vec![mock_attribute_one];
        let result =
            find_first_attribute_by_key(&attributes, String::from("test-one")).unwrap_err();

        assert_eq!(
            result.to_string(),
            "Custom Error val: \"could not find attribute with key: test-one\""
        );
    }

    #[test]
    fn find_first_attribute_by_key_given_no_attributes_should_fail() {
        let attributes: Vec<Attribute> = Vec::new();
        let result =
            find_first_attribute_by_key(&attributes, String::from("test-one")).unwrap_err();

        assert_eq!(
            result.to_string(),
            "Custom Error val: \"could not find attribute with key: test-one\""
        );
    }
}
