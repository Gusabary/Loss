use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

pub fn detect_log_timstamp_format(line: &str) -> Option<String> {
    // ordering should be strict -> loose
    const LOG_TIMESTAMP_FORMATS: &[&str] = &["%Y-%m-%d %H:%M:%S%.9f", "[%Y-%m-%d %H:%M:%S]"];
    for &fmt in LOG_TIMESTAMP_FORMATS {
        if NaiveDateTime::parse_and_remainder(line, fmt).is_ok() {
            return Some(fmt.to_string());
        }
    }
    None
}

pub fn parse_log_timestamp(content: &str) -> (Option<NaiveDate>, Option<NaiveTime>) {
    let (date, time_str) =
        if let Ok((date, remain)) = NaiveDate::parse_and_remainder(content, "%Y-%m-%d") {
            (Some(date), remain)
        } else {
            (None, content)
        };

    // NaiveTime cannot parse from a single %H
    let time_str = if time_str.contains(':') {
        time_str.to_string()
    } else {
        format!("{time_str}:00")
    };

    // ordering should be strict -> loose
    const TIME_FORMATS: &[&str] = &["%H:%M:%S%.9f", "%H:%M"];
    let mut time = None;
    for &fmt in TIME_FORMATS {
        if let Ok(parsed_time) = NaiveTime::parse_from_str(&time_str, fmt) {
            time = Some(parsed_time);
            break;
        }
    }
    (date, time)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

    #[test]
    fn test_detect_log_timstamp_format() {
        assert_eq!(
            detect_log_timstamp_format("2024-01-01 12:00:00.123 [Info] hello"),
            Some("%Y-%m-%d %H:%M:%S%.9f".to_string())
        );
        assert_eq!(
            detect_log_timstamp_format("2024-01-01 12:00:00 [Info] hello"),
            Some("%Y-%m-%d %H:%M:%S%.9f".to_string())
        );
        assert_eq!(
            detect_log_timstamp_format("20240101 12:00:00 [Info] hello"),
            None
        );
        assert_eq!(
            detect_log_timstamp_format("[2024-01-01 12:00:00] [Info] hello"),
            Some("[%Y-%m-%d %H:%M:%S]".to_string())
        );
    }

    #[test]
    fn test_parse_log_timestamp() {
        assert_eq!(
            parse_log_timestamp("2024-01-02 8:12:50.1234567"),
            (
                NaiveDate::from_ymd_opt(2024, 1, 2),
                NaiveTime::from_hms_nano_opt(8, 12, 50, 123456700)
            )
        );
        assert_eq!(
            parse_log_timestamp("2024-01-02 8:12:20"),
            (
                NaiveDate::from_ymd_opt(2024, 1, 2),
                NaiveTime::from_hms_nano_opt(8, 12, 20, 0)
            )
        );
        assert_eq!(
            parse_log_timestamp("2024-01-02 8:12"),
            (
                NaiveDate::from_ymd_opt(2024, 1, 2),
                NaiveTime::from_hms_nano_opt(8, 12, 0, 0)
            )
        );
        assert_eq!(
            parse_log_timestamp("2024-01-02 21"),
            (
                NaiveDate::from_ymd_opt(2024, 1, 2),
                NaiveTime::from_hms_nano_opt(21, 0, 0, 0)
            )
        );
        assert_eq!(
            parse_log_timestamp("8:12:50.1234567"),
            (None, NaiveTime::from_hms_nano_opt(8, 12, 50, 123456700))
        );
        assert_eq!(
            parse_log_timestamp("8:12:20"),
            (None, NaiveTime::from_hms_nano_opt(8, 12, 20, 0))
        );
        assert_eq!(
            parse_log_timestamp("8:12"),
            (None, NaiveTime::from_hms_nano_opt(8, 12, 0, 0))
        );
        assert_eq!(
            parse_log_timestamp("21"),
            (None, NaiveTime::from_hms_nano_opt(21, 0, 0, 0))
        );
    }
}
