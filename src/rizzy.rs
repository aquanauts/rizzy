use chrono::{DateTime, Duration, NaiveDateTime, TimeZone};
use chrono_tz::Tz;
use lazy_static::lazy_static;
use regex::Captures;
use regex::Regex;

pub struct Rizzy {
    tz: Tz,
    format: Option<String>,
    epoch_nanos: bool,
}

fn fix_subsec(dt: &NaiveDateTime, num_sub_digits: usize) -> NaiveDateTime {
    if num_sub_digits == 0 {
        return *dt;
    }
    let ns_raw = dt.timestamp_subsec_nanos() as i64;
    let multiplier = match num_sub_digits {
        3 => 1_000_000,
        6 => 1_000,
        9 => 1,
        _ => panic!("Invalid datetime with {} digits in the time", num_sub_digits)
    };
    dt
        .checked_sub_signed(Duration::nanoseconds(ns_raw)).unwrap()
        .checked_add_signed(Duration::nanoseconds(ns_raw * multiplier)).unwrap()
}

impl Rizzy {
    pub fn new(tz: Tz, format: Option<String>, epoch_nanos: bool) -> Rizzy {
        Rizzy { tz, format, epoch_nanos }
    }

    fn format_time(&self, dt: &NaiveDateTime) -> String {
        let utc_time = self.tz.from_utc_datetime(dt);
        match &self.format {
            Some(format) => format!("{}", utc_time.format(&format)),
            None => utc_time.to_rfc3339()
        }
    }

    fn replace_one_time(&self, some_ts: &str, t_string: &str, num_sub_digits: usize, tz_part: &str) -> String {
        if tz_part == "Z" {
            let format_string = format!("%Y-%m-%d{}%H:%M:%S{}Z", t_string, (if num_sub_digits > 0 { ".%f" } else { "" }));
            let parse_result = NaiveDateTime::parse_from_str(some_ts, format_string.as_str());
            match parse_result {
                Err(e) => panic!("Unable to parse datetime {}: {}", some_ts, e),
                Ok(dt) => self.format_time(&fix_subsec(&dt, num_sub_digits))
            }
        } else {
            let format_string = format!("%Y-%m-%d{}%H:%M:%S{}%z", t_string, (if num_sub_digits > 0 { ".%f" } else { "" }));
            let parse_result = DateTime::parse_from_str(some_ts, format_string.as_str());
            match parse_result {
                Err(e) => panic!("Unable to parse datetime {}: {}", some_ts, e),
                Ok(dt) => self.format_time(&fix_subsec(&dt.naive_utc(), num_sub_digits))
            }
        }
    }

    pub fn handle_line(&self, line: &str) -> String {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"\d{4}-\d{2}-\d{2}(?P<T>[ T])\d{2}:\d{2}:\d{2}(?P<sub>\.\d{3,9})?(?P<TZ>Z|[+-]\d{2}:?\d{2})").unwrap();
            static ref NS_RE: Regex = Regex::new(r"\d{19}").unwrap();
        }

        let mut replaced = RE.replace_all(line, |caps: &Captures<'_>| {
            self.replace_one_time(
                caps.get(0).unwrap().as_str(),
                caps.name("T").unwrap().as_str(),
                caps.name("sub").map(|s| s.range().len() - 1).unwrap_or(0),
                caps.name("TZ").unwrap().as_str(),
            )
        }).to_string();

        if self.epoch_nanos {
            replaced = NS_RE.replace_all(&replaced, |caps: &Captures<'_>| {
                let ts: i64 = caps.get(0).unwrap().as_str().parse().unwrap();
                let utc_time = NaiveDateTime::from_timestamp(ts / 1_000_000_000, (ts % 1_000_000_000) as u32);
                self.format_time(&utc_time)
            }).to_string();
        }

        replaced
    }
}

#[cfg(test)]
mod tests {
    use chrono_tz::America::{Chicago, New_York};
    use super::Rizzy;

    #[test]
    fn should_replace_dates_1() {
        let rizzy = Rizzy { tz: New_York, format: None, epoch_nanos: false };
        assert_eq!(
            rizzy.handle_line("2020-08-27T12:45:37Z foobar"),
            "2020-08-27T08:45:37-04:00 foobar"
        );
    }

    #[test]
    fn should_replace_dates_2() {
        let rizzy = Rizzy { tz: New_York, format: None, epoch_nanos: false };
        assert_eq!(
            rizzy.handle_line("2020-08-27 13:30:00+00:00 foobar"),
            "2020-08-27T09:30:00-04:00 foobar"
        );
    }

    #[test]
    fn should_replace_dates_3() {
        let rizzy = Rizzy { tz: New_York, format: None, epoch_nanos: false };
        assert_eq!(
            rizzy.handle_line("2020-08-27 12:45:43.728154+00:00 foobar"),
            "2020-08-27T08:45:43.728154-04:00 foobar"
        );
    }

    #[test]
    fn should_replace_dates_4() {
        let rizzy = Rizzy { tz: New_York, format: None, epoch_nanos: false };
        assert_eq!(
            rizzy.handle_line("2020-09-11 14:48:34+0000 foobar"),
            "2020-09-11T10:48:34-04:00 foobar"
        );
    }

    #[test]
    fn should_replace_dates_5() {
        let rizzy = Rizzy { tz: Chicago, format: None, epoch_nanos: false };
        assert_eq!(
            rizzy.handle_line("2021-03-25 17:39:28.167391+00:00 foobar"),
            "2021-03-25T12:39:28.167391-05:00 foobar"
        );
    }

    #[test]
    fn should_replace_dates_6() {
        let rizzy = Rizzy { tz: Chicago, format: None, epoch_nanos: false };
        assert_eq!(
            rizzy.handle_line("2020-08-27 12:45:43.728154+00:00 foobar"),
            "2020-08-27T07:45:43.728154-05:00 foobar"
        );
    }

    #[test]
    fn should_replace_multiple_matches() {
        let rizzy = Rizzy { tz: Chicago, format: None, epoch_nanos: false };
        assert_eq!(
            rizzy.handle_line("2020-08-27 12:45:43.728154+00:00 foo 2020-08-27T12:45:37Z"),
            "2020-08-27T07:45:43.728154-05:00 foo 2020-08-27T07:45:37-05:00"
        );
    }

    #[test]
    fn should_apply_format() {
        let rizzy = Rizzy { tz: New_York, format: Option::from("%H:%M:%S".to_string()), epoch_nanos: false };
        assert_eq!(rizzy.handle_line("2020-09-11 14:48:34+0000 foobar"), "10:48:34 foobar");
    }

    #[test]
    fn should_convert_epoch_nanos() {
        let rizzy = Rizzy { tz: New_York, format: None, epoch_nanos: true };
        assert_eq!(rizzy.handle_line("1607965978437104000 foobar"), "2020-12-14T12:12:58.437104-05:00 foobar");
    }
}
