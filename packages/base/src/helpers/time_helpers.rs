use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use cosmwasm_std::{StdError, StdResult, Timestamp};
use cron::Schedule;
use std::{convert::TryInto, str::FromStr};

pub fn try_hydrate_cron_expression(
    cron_expression: String,
    timestamp: Timestamp,
) -> StdResult<String> {
    let date = date_time_from_timestamp(timestamp);

    let hydrated_expression = cron_to_quartz(
        cron_expression
            .split_whitespace()
            .zip(vec![
                date.minute().to_string(),
                date.hour().to_string(),
                date.day().to_string(),
                date.month().to_string(),
                date.weekday().to_string(),
            ])
            .map(|(old, new)| str::replace(old, "$", &new))
            .collect::<Vec<String>>()
            .join(" "),
    );

    match Schedule::from_str(&hydrated_expression.clone()) {
        Ok(_) => Ok(quartz_to_cron(hydrated_expression)),
        Err(_) => Err(StdError::generic_err(format!(
            "Invalid cron expression: {}",
            hydrated_expression
        ))),
    }
}

fn cron_to_quartz(cron_expression: String) -> String {
    format!("0 {}", cron_expression)
}

fn quartz_to_cron(quartz_expression: String) -> String {
    quartz_expression
        .strip_prefix("0 ")
        .expect("should have a seconds value at the start")
        .to_string()
}

pub fn get_next_target_time(
    schedule_expression: String,
    block_time: Timestamp,
) -> StdResult<Timestamp> {
    let schedule = Schedule::from_str(&cron_to_quartz(schedule_expression))
        .expect("should be valid cron expression");

    println!(
        "{:?}",
        schedule
            .after(&date_time_from_timestamp(block_time))
            .next()
            .unwrap()
            .to_rfc2822()
    );

    let next_execution_time = schedule
        .after(&date_time_from_timestamp(block_time))
        .next()
        .expect("should have a next execution time");

    Ok(Timestamp::from_seconds(
        next_execution_time.timestamp() as u64
    ))
}

pub fn date_time_from_timestamp(timestamp: Timestamp) -> DateTime<Utc> {
    Utc.timestamp(
        timestamp.seconds() as i64,
        timestamp
            .subsec_nanos()
            .try_into()
            .expect("should be a valid timestamp"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hydrate_with_seconds_works() {
        let cron_expression = "* * * * *".to_string();
        let date = Utc.ymd(2021, 1, 1).and_hms(0, 0, 0);

        let hydrated = try_hydrate_cron_expression(
            cron_expression,
            Timestamp::from_seconds(date.timestamp() as u64),
        )
        .unwrap();

        assert_eq!(hydrated, "* * * * *");
    }

    #[test]
    fn hydrate_with_minutes_works() {
        let cron_expression = "$ * * * *".to_string();
        let date = Utc.ymd(2021, 1, 1).and_hms(0, 0, 0);

        let hydrated = try_hydrate_cron_expression(
            cron_expression,
            Timestamp::from_seconds(date.timestamp() as u64),
        )
        .unwrap();

        assert_eq!(hydrated, "0 * * * *");
    }

    #[test]
    fn hydrate_with_hours_works() {
        let cron_expression = "* 0 * * *".to_string();
        let date = Utc.ymd(2021, 1, 1).and_hms(0, 0, 0);

        let hydrated = try_hydrate_cron_expression(
            cron_expression,
            Timestamp::from_seconds(date.timestamp() as u64),
        )
        .unwrap();

        assert_eq!(hydrated, "* 0 * * *");
    }

    #[test]
    fn hydrate_with_days_works() {
        let cron_expression = "* * 1 * *".to_string();
        let date = Utc.ymd(2021, 1, 1).and_hms(0, 0, 0);

        let hydrated = try_hydrate_cron_expression(
            cron_expression,
            Timestamp::from_seconds(date.timestamp() as u64),
        )
        .unwrap();

        assert_eq!(hydrated, "* * 1 * *");
    }

    #[test]
    fn hydrate_with_months_works() {
        let cron_expression = "* * * $ *".to_string();
        let date = Utc.ymd(2021, 1, 1).and_hms(0, 0, 0);

        let hydrated = try_hydrate_cron_expression(
            cron_expression,
            Timestamp::from_seconds(date.timestamp() as u64),
        )
        .unwrap();

        assert_eq!(hydrated, "* * * 1 *");
    }

    #[test]
    fn hydrate_with_weekdays_works() {
        let cron_expression = "* * * * $".to_string();
        let date = Utc.ymd(2021, 1, 1).and_hms(0, 0, 0);

        let hydrated = try_hydrate_cron_expression(
            cron_expression,
            Timestamp::from_seconds(date.timestamp() as u64),
        )
        .unwrap();

        assert_eq!(hydrated, "* * * * Fri");
    }

    #[test]
    fn try_hydrate_for_hourly_schedule_works() {
        let cron_expression = "$ * * * *".to_string();
        let date = Utc.ymd(2021, 1, 1).and_hms(3, 2, 1);

        let hydrated = try_hydrate_cron_expression(
            cron_expression,
            Timestamp::from_seconds(date.timestamp() as u64),
        )
        .unwrap();

        assert_eq!(hydrated, "2 * * * *");
    }

    #[test]
    fn try_hydrate_for_daily_schedule_works() {
        let cron_expression = "$ $ * * *".to_string();
        let date = Utc.ymd(2021, 1, 1).and_hms(3, 2, 1);

        let hydrated = try_hydrate_cron_expression(
            cron_expression,
            Timestamp::from_seconds(date.timestamp() as u64),
        )
        .unwrap();

        assert_eq!(hydrated, "2 3 * * *");
    }

    #[test]
    fn try_hydrate_for_weekly_schedule_works() {
        let cron_expression = "$ $ * * $".to_string();
        let date = Utc.ymd(2021, 1, 1).and_hms(3, 2, 1);

        let hydrated = try_hydrate_cron_expression(
            cron_expression,
            Timestamp::from_seconds(date.timestamp() as u64),
        )
        .unwrap();

        assert_eq!(hydrated, "2 3 * * Fri");
    }

    #[test]
    fn try_hydrate_for_monthly_schedule_works() {
        let cron_expression = "$ $ $ * *".to_string();
        let date = Utc.ymd(2021, 1, 1).and_hms(3, 2, 1);

        let hydrated = try_hydrate_cron_expression(
            cron_expression,
            Timestamp::from_seconds(date.timestamp() as u64),
        )
        .unwrap();

        assert_eq!(hydrated, "2 3 1 * *");
    }

    #[test]
    fn try_hydrate_for_custom_schedule_works() {
        let cron_expression = "$ $ $/1 * *".to_string();
        let date = Utc.ymd(2021, 1, 1).and_hms(3, 2, 1);

        let hydrated = try_hydrate_cron_expression(
            cron_expression,
            Timestamp::from_seconds(date.timestamp() as u64),
        )
        .unwrap();

        assert_eq!(hydrated, "2 3 1/1 * *");
    }

    #[test]
    fn try_hydrate_for_invalid_schedule_fails() {
        let cron_expression = "$$$ $ $/1 * *".to_string();
        let date = Utc.ymd(2021, 1, 1).and_hms(3, 2, 1);

        let err = try_hydrate_cron_expression(
            cron_expression,
            Timestamp::from_seconds(date.timestamp() as u64),
        )
        .unwrap_err();

        assert_eq!(
            err.to_string(),
            "Generic error: Invalid cron expression: 0 222 3 1/1 * *"
        );
    }

    #[test]
    fn get_next_execution_time_monthly_should_get_next_month() {
        let mock_current_time = Utc.ymd(2022, 1, 15).and_hms(1, 0, 1);
        let mock_current_timestamp =
            Timestamp::from_seconds(mock_current_time.timestamp().try_into().unwrap());

        let expected_next_execution_time = Utc.ymd(2022, 2, 1).and_hms(1, 0, 0);
        let expected_next_execution_timestamp =
            Timestamp::from_seconds(expected_next_execution_time.timestamp().try_into().unwrap());

        let actual_next_execution_time =
            get_next_target_time("0 1 1 * *".to_string(), mock_current_timestamp).unwrap();

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
            get_next_target_time("0 1 1 * *".to_string(), mock_current_timestamp).unwrap();

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
            get_next_target_time("0 2 1 * *".to_string(), mock_current_timestamp).unwrap();

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
            get_next_target_time("0 1 * * SAT".to_string(), mock_current_timestamp).unwrap();

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
            get_next_target_time("0 1 * * SAT".to_string(), mock_current_timestamp).unwrap();

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
            get_next_target_time("0 2 * * SAT".to_string(), mock_current_timestamp).unwrap();

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
            get_next_target_time("0 1 * * *".to_string(), mock_current_timestamp).unwrap();

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
            get_next_target_time("0 1 * * *".to_string(), mock_current_timestamp).unwrap();

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
            get_next_target_time("0 2 * * *".to_string(), mock_current_timestamp).unwrap();

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
            get_next_target_time("0 * * * *".to_string(), mock_current_timestamp).unwrap();

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
            get_next_target_time("0 * * * *".to_string(), mock_current_timestamp).unwrap();

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
            get_next_target_time("0 * * * *".to_string(), mock_current_timestamp).unwrap();

        assert_eq!(
            expected_next_execution_timestamp,
            actual_next_execution_time
        )
    }
}
