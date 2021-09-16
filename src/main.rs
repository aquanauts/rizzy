use std::io;
use std::io::BufRead;

use chrono::{DateTime, TimeZone};
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

fn replace_one_time(tz: &Tz, some_ts: &str) -> String {
    let moo = DateTime::parse_from_str(some_ts, "%Y-%m-%d %H:%M:%S%z");
    // let moo = DateTime::parse_from_str(some_ts, "%Y-%m-%d %H:%M:%S.%f%z");
    match moo {
        Err(_) => some_ts.to_string(),
        Ok(dt) => tz.from_utc_datetime(&dt.naive_utc()).to_rfc3339()
    }
}

fn timify_line(tz: &Tz, line: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}[+-]\d{2}:?\d{2}").unwrap();
    }

    RE.replace_all(line, |caps: &Captures<'_>| {
        format!("{:#}", replace_one_time(tz, caps.get(0).unwrap().as_str()))
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

fn main() {
    let opts: Opts = Opts::parse();
    let tz = get_timezone(opts);

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        println!("{}", timify_line(&tz, &line.unwrap()));
    }
}
