use chrono::{TimeZone, Utc};
use cosmwasm_std::{StdResult, Timestamp};
use cron::Schedule;
use std::{convert::TryInto, str::FromStr};

pub fn get_next_target_time(
    schedule_expression: String,
    block_time: Timestamp,
) -> StdResult<Timestamp> {
    let schedule = Schedule::from_str(&schedule_expression).expect("valid cron expression");
    let next_execution_time = schedule
        .after(
            &Utc.timestamp(
                block_time.seconds().try_into().expect("valid timestamp"),
                block_time
                    .subsec_nanos()
                    .try_into()
                    .expect("valid timestamp"),
            ),
        )
        .next()
        .unwrap();

    Ok(Timestamp::from_seconds(
        next_execution_time.timestamp() as u64
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_next_execution_time_monthly_should_get_next_month() {
        let mock_current_time = Utc.ymd(2022, 1, 15).and_hms(1, 0, 1);
        let mock_current_timestamp =
            Timestamp::from_seconds(mock_current_time.timestamp().try_into().unwrap());

        let expected_next_execution_time = Utc.ymd(2022, 2, 1).and_hms(1, 0, 0);
        let expected_next_execution_timestamp =
            Timestamp::from_seconds(expected_next_execution_time.timestamp().try_into().unwrap());

        let actual_next_execution_time =
            get_next_target_time("0 0 1 1 * *".to_string(), mock_current_timestamp).unwrap();

        assert_eq!(
            expected_next_execution_timestamp,
            actual_next_execution_time
        )
    }

    #[test]
    fn get_next_execution_time_monthly_from_late_last_execution_should_get_next_month() {
        // current time is 1 month and 5 days since last execution - so need to find next possible time
        let mock_current_time = Utc.ymd(2022, 2, 15).and_hms(1, 0, 0);
        let mock_current_timestamp =
            Timestamp::from_seconds(mock_current_time.timestamp().try_into().unwrap());

        let expected_next_execution_time = Utc.ymd(2022, 3, 1).and_hms(1, 0, 0);
        let expected_next_execution_timestamp =
            Timestamp::from_seconds(expected_next_execution_time.timestamp().try_into().unwrap());

        let actual_next_execution_time =
            get_next_target_time("0 0 1 1 * *".to_string(), mock_current_timestamp).unwrap();

        assert_eq!(
            expected_next_execution_timestamp,
            actual_next_execution_time
        )
    }

    #[test]
    fn get_next_execution_time_monthly_from_late_last_execution_should_get_today() {
        // current time is 2 months and 23 hours since last execution - so need to find next possible time
        let mock_current_time = Utc.ymd(2022, 3, 1).and_hms(1, 0, 0);
        let mock_current_timestamp =
            Timestamp::from_seconds(mock_current_time.timestamp().try_into().unwrap());

        let expected_next_execution_time = Utc.ymd(2022, 3, 1).and_hms(2, 0, 0);
        let expected_next_execution_timestamp =
            Timestamp::from_seconds(expected_next_execution_time.timestamp().try_into().unwrap());

        let actual_next_execution_time =
            get_next_target_time("0 0 2 1 * *".to_string(), mock_current_timestamp).unwrap();

        assert_eq!(
            expected_next_execution_timestamp,
            actual_next_execution_time
        )
    }

    #[test]
    fn get_next_execution_time_weekly_should_get_next_week() {
        // current time is 4 days since last execution - so in expected time frame
        let mock_current_time = Utc.ymd(2022, 1, 5).and_hms(1, 0, 1);
        let mock_current_timestamp =
            Timestamp::from_seconds(mock_current_time.timestamp().try_into().unwrap());

        let expected_next_execution_time = Utc.ymd(2022, 1, 8).and_hms(1, 0, 0);
        let expected_next_execution_timestamp =
            Timestamp::from_seconds(expected_next_execution_time.timestamp().try_into().unwrap());

        let actual_next_execution_time =
            get_next_target_time("0 0 1 ? * SAT".to_string(), mock_current_timestamp).unwrap();

        assert_eq!(
            expected_next_execution_timestamp,
            actual_next_execution_time
        )
    }

    #[test]
    fn get_next_execution_time_weekly_from_late_last_execution_should_get_next_week() {
        // current time 8 days since last execution - so need to find next possible time
        let mock_current_time = Utc.ymd(2022, 1, 9).and_hms(1, 0, 0);
        let mock_current_timestamp =
            Timestamp::from_seconds(mock_current_time.timestamp().try_into().unwrap());

        let expected_next_execution_time = Utc.ymd(2022, 1, 15).and_hms(1, 0, 0);
        let expected_next_execution_timestamp =
            Timestamp::from_seconds(expected_next_execution_time.timestamp().try_into().unwrap());

        let actual_next_execution_time =
            get_next_target_time("0 0 1 ? * SAT".to_string(), mock_current_timestamp).unwrap();

        assert_eq!(
            expected_next_execution_timestamp,
            actual_next_execution_time
        )
    }

    #[test]
    fn get_next_execution_time_weekly_from_late_last_execution_should_get_today() {
        // current time is 13 days and 23 hours since last execution - so need to find next possible time
        let mock_current_time = Utc.ymd(2022, 1, 15).and_hms(1, 0, 0);
        let mock_current_timestamp =
            Timestamp::from_seconds(mock_current_time.timestamp().try_into().unwrap());

        let expected_next_execution_time = Utc.ymd(2022, 1, 15).and_hms(2, 0, 0);
        let expected_next_execution_timestamp =
            Timestamp::from_seconds(expected_next_execution_time.timestamp().try_into().unwrap());

        let actual_next_execution_time =
            get_next_target_time("0 0 2 ? * SAT".to_string(), mock_current_timestamp).unwrap();

        assert_eq!(
            expected_next_execution_timestamp,
            actual_next_execution_time
        )
    }

    #[test]
    fn get_next_execution_time_daily_should_get_next_day() {
        // current time is 12 hours since last execution = so in expected time frame
        let mock_current_time = Utc.ymd(2022, 1, 1).and_hms(12, 0, 1);
        let mock_current_timestamp =
            Timestamp::from_seconds(mock_current_time.timestamp().try_into().unwrap());

        let expected_next_execution_time = Utc.ymd(2022, 1, 2).and_hms(1, 0, 0);
        let expected_next_execution_timestamp =
            Timestamp::from_seconds(expected_next_execution_time.timestamp().try_into().unwrap());

        let actual_next_execution_time =
            get_next_target_time("0 0 1 * * ?".to_string(), mock_current_timestamp).unwrap();

        assert_eq!(
            expected_next_execution_timestamp,
            actual_next_execution_time
        )
    }

    #[test]
    fn get_next_execution_time_daily_from_late_last_execution_should_get_next_day() {
        // current time 8 days since last execution - so need to find next possible time
        let mock_current_time = Utc.ymd(2022, 1, 9).and_hms(1, 0, 0);
        let mock_current_timestamp =
            Timestamp::from_seconds(mock_current_time.timestamp().try_into().unwrap());

        let expected_next_execution_time = Utc.ymd(2022, 1, 10).and_hms(1, 0, 0);
        let expected_next_execution_timestamp =
            Timestamp::from_seconds(expected_next_execution_time.timestamp().try_into().unwrap());

        let actual_next_execution_time =
            get_next_target_time("0 0 1 * * ?".to_string(), mock_current_timestamp).unwrap();

        assert_eq!(
            expected_next_execution_timestamp,
            actual_next_execution_time
        )
    }

    #[test]
    fn get_next_execution_time_daily_from_late_last_execution_should_get_today() {
        // current time is 13 days and 23 hours since last execution - so need to find next possible time
        let mock_current_time = Utc.ymd(2022, 1, 15).and_hms(1, 0, 0);
        let mock_current_timestamp =
            Timestamp::from_seconds(mock_current_time.timestamp().try_into().unwrap());

        let expected_next_execution_time = Utc.ymd(2022, 1, 15).and_hms(2, 0, 0);
        let expected_next_execution_timestamp =
            Timestamp::from_seconds(expected_next_execution_time.timestamp().try_into().unwrap());

        let actual_next_execution_time =
            get_next_target_time("0 0 2 * * ?".to_string(), mock_current_timestamp).unwrap();

        assert_eq!(
            expected_next_execution_timestamp,
            actual_next_execution_time
        )
    }

    #[test]
    fn get_next_execution_time_hourly_should_get_next_hour() {
        // current time is 30 minutes since last execution - so in expected time frame
        let mock_current_time = Utc.ymd(2022, 1, 1).and_hms(1, 30, 1);
        let mock_current_timestamp =
            Timestamp::from_seconds(mock_current_time.timestamp().try_into().unwrap());

        let expected_next_execution_time = Utc.ymd(2022, 1, 1).and_hms(2, 0, 0);
        let expected_next_execution_timestamp =
            Timestamp::from_seconds(expected_next_execution_time.timestamp().try_into().unwrap());

        let actual_next_execution_time =
            get_next_target_time("0 0 * ? * *".to_string(), mock_current_timestamp).unwrap();

        assert_eq!(
            expected_next_execution_timestamp,
            actual_next_execution_time
        )
    }

    #[test]
    fn get_next_execution_time_hourly_from_late_last_execution_should_get_next_hour() {
        // current time 8 days since last execution - so need to find next possible time
        let mock_current_time = Utc.ymd(2022, 1, 9).and_hms(1, 0, 0);
        let mock_current_timestamp =
            Timestamp::from_seconds(mock_current_time.timestamp().try_into().unwrap());

        let expected_next_execution_time = Utc.ymd(2022, 1, 9).and_hms(2, 0, 0);
        let expected_next_execution_timestamp =
            Timestamp::from_seconds(expected_next_execution_time.timestamp().try_into().unwrap());

        let actual_next_execution_time =
            get_next_target_time("0 0 * ? * *".to_string(), mock_current_timestamp).unwrap();

        assert_eq!(
            expected_next_execution_timestamp,
            actual_next_execution_time
        )
    }

    #[test]
    fn get_next_execution_time_hourly_from_late_last_execution_should_get_today() {
        // current time is 13 days and 23 hours since last execution - so need to find next possible time
        let mock_current_time = Utc.ymd(2022, 1, 15).and_hms(1, 58, 0);
        let mock_current_timestamp =
            Timestamp::from_seconds(mock_current_time.timestamp().try_into().unwrap());

        let expected_next_execution_time = Utc.ymd(2022, 1, 15).and_hms(2, 0, 0);
        let expected_next_execution_timestamp =
            Timestamp::from_seconds(expected_next_execution_time.timestamp().try_into().unwrap());

        let actual_next_execution_time =
            get_next_target_time("0 0 * ? * *".to_string(), mock_current_timestamp).unwrap();

        assert_eq!(
            expected_next_execution_timestamp,
            actual_next_execution_time
        )
    }
}
