#![deny(warnings)]

use std::io;
use std::io::BufRead;

use chrono_tz::America::{Chicago, New_York};
use chrono_tz::Tz;
use clap::{AppSettings, Clap};

use crate::rizzy::Rizzy;

mod rizzy;


#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Use New York timezone
    #[clap(long)]
    nyc: bool,
    /// Use Chicago timestamp
    #[clap(long)]
    chi: bool,
    /// use ZONE instead of local zone
    #[clap(short, long)]
    zone: Option<String>,
    /// Convert nanos since epoch
    #[clap(short = 'n', long)]
    convert_epoch_nanos: bool,
    /// Use FORMAT as the strftime format instead of RFC3322
    #[clap(short, long)]
    format: Option<String>,
}

fn parse_timezone(time_zone_str: &String) -> Tz {
    match time_zone_str.parse() {
        Ok(res) => res,
        Err(e) => panic!("Could not parse timezone: {}, Error: {}", time_zone_str, e.to_string())
    }
}

fn get_timezone(opts: &Opts) -> Tz {
    match opts {
        Opts { nyc: true, chi: true, .. } => panic!("cannot use more than one timezone override"),
        Opts { nyc: true, chi: false, zone: None, .. } => New_York,
        Opts { nyc: false, chi: true, zone: None, .. } => Chicago,
        Opts { nyc: true, chi: false, zone: Some(_), .. } => panic!("cannot supply --zone and an override"),
        Opts { nyc: false, chi: true, zone: Some(_), .. } => panic!("cannot supply --zone and an override"),
        Opts { nyc: false, chi: false, zone: Some(tz_string), .. } => parse_timezone(&tz_string),
        Opts { nyc: false, chi: false, .. } => Chicago,
    }
}

fn main() {
    let opts: Opts = Opts::parse();
    let rizzy = Rizzy::new(get_timezone(&opts), opts.format, opts.convert_epoch_nanos);

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        println!("{}", rizzy.handle_line(&line.unwrap()));
    }
}
