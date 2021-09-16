use std::io;
use std::io::BufRead;

use chrono::{DateTime, Duration, NaiveDateTime, TimeZone};
use chrono_tz::America::{Chicago, New_York};
use chrono_tz::Tz;
use clap::{AppSettings, Clap};
use lazy_static::lazy_static;
use regex::Captures;
use regex::Regex;

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(short, long)]
    nyc: bool,
    #[clap(short, long)]
    chi: bool,
    #[clap(short, long)]
    zone: Option<String>,
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

fn replace_one_time(tz: &Tz, some_ts: &str, t_string: &str, num_sub_digits: usize, tz_part: &str) -> String {
    if tz_part == "Z" {
        let format_string = format!("%Y-%m-%d{}%H:%M:%S{}Z", t_string, (if num_sub_digits > 0 { ".%f" } else { "" }));
        let parse_result = NaiveDateTime::parse_from_str(some_ts, format_string.as_str());
        match parse_result {
            Err(e) => panic!("Unable to parse datetime {}: {}", some_ts, e),
            Ok(dt) => tz.from_utc_datetime(&fix_subsec(&dt, num_sub_digits)).to_rfc3339()
        }
    } else {
        let format_string = format!("%Y-%m-%d{}%H:%M:%S{}%z", t_string, (if num_sub_digits > 0 { ".%f" } else { "" }));
        let parse_result = DateTime::parse_from_str(some_ts, format_string.as_str());
        match parse_result {
            Err(e) => panic!("Unable to parse datetime {}: {}", some_ts, e),
            Ok(dt) => tz.from_utc_datetime(&fix_subsec(&dt.naive_utc(), num_sub_digits)).to_rfc3339()
        }
    }
}

fn timify_line(tz: &Tz, line: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\d{4}-\d{2}-\d{2}(?P<T>[ T])\d{2}:\d{2}:\d{2}(?P<sub>\.\d{3,9})?(?P<TZ>Z|[+-]\d{2}:?\d{2})").unwrap();
    }

    RE.replace_all(line, |caps: &Captures<'_>| {
        replace_one_time(
            tz,
            caps.get(0).unwrap().as_str(),
            caps.name("T").unwrap().as_str(),
            caps.name("sub").map(|s| s.range().len() - 1).unwrap_or(0),
            caps.name("TZ").unwrap().as_str(),
        )
    })
        .to_string()
}

fn parse_timezone(time_zone_str: &String) -> Tz {
    match time_zone_str.parse() {
        Ok(res) => res,
        Err(e) => panic!("Could not parse timezone: {}, Error: {}", time_zone_str, e.to_string())
    }
}

fn get_timezone(opts: Opts) -> Tz {
    match opts {
        Opts { nyc: true, chi: true, zone: _ } => panic!("cannot use more than one timezone override"),
        Opts { nyc: true, chi: false, zone: None } => New_York,
        Opts { nyc: false, chi: true, zone: None } => Chicago,
        Opts { nyc: true, chi: false, zone: Some(_) } => panic!("cannot supply --zone and an override"),
        Opts { nyc: false, chi: true, zone: Some(_) } => panic!("cannot supply --zone and an override"),
        Opts { nyc: false, chi: false, zone: Some(tz_string) } => parse_timezone(&tz_string),
        Opts { nyc: false, chi: false, zone: _ } => Chicago,
    }
}

#[cfg(test)]
mod tests {
    use super::{Chicago, New_York};
    use super::timify_line;

    #[test]
    fn should_replace_dates_1() {
        assert_eq!(
            timify_line(&New_York, "2020-08-27T12:45:37Z foobar"),
            "2020-08-27T08:45:37-04:00 foobar"
        );
    }

    #[test]
    fn should_replace_dates_2() {
        assert_eq!(
            timify_line(&New_York, "2020-08-27 13:30:00+00:00 foobar"),
            "2020-08-27T09:30:00-04:00 foobar"
        );
    }

    #[test]
    fn should_replace_dates_3() {
        assert_eq!(
            timify_line(&New_York, "2020-08-27 12:45:43.728154+00:00 foobar"),
            "2020-08-27T08:45:43.728154-04:00 foobar"
        );
    }

    #[test]
    fn should_replace_dates_4() {
        assert_eq!(
            timify_line(&New_York, "2020-09-11 14:48:34+0000 foobar"),
            "2020-09-11T10:48:34-04:00 foobar"
        );
    }

    #[test]
    fn should_replace_dates_5() {
        assert_eq!(
            timify_line(&Chicago, "2021-03-25 17:39:28.167391+00:00 foobar"),
            "2021-03-25T12:39:28.167391-05:00 foobar"
        );
    }

    #[test]
    fn should_replace_dates_6() {
        assert_eq!(
            timify_line(&Chicago, "2020-08-27 12:45:43.728154+00:00 foobar"),
            "2020-08-27T07:45:43.728154-05:00 foobar"
        );
    }

    #[test]
    fn should_replace_multiple_matches() {
        assert_eq!(
            timify_line(&Chicago, "2020-08-27 12:45:43.728154+00:00 foo 2020-08-27T12:45:37Z"),
            "2020-08-27T07:45:43.728154-05:00 foo 2020-08-27T07:45:37-05:00"
        );
    }
    //
    // def test_should_apply_format():
    // tizzy = Tizzy(pytz.timezone("America/New_York"), output_format="%H:%M:%S")
    // assert tizzy.handle_line("2020-09-11 14:48:34+0000 foobar") == "10:48:34 foobar"
    //
    //
    // def test_should_convert_epoch_nanos():
    // tizzy = Tizzy(pytz.timezone("America/New_York"), convert_epoch_nanos=True)
    // assert tizzy.handle_line("1607965978437104000") == "2020-12-14T12:12:58.000437-05:00"
}


fn main() {
    let opts: Opts = Opts::parse();
    let tz = get_timezone(opts);

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        println!("{}", timify_line(&tz, &line.unwrap()));
    }
}
