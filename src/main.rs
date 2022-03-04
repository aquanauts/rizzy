#![deny(warnings)]

use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;
use std::process::exit;

use ::rizzy::timezones::parse_timezone;
use ::rizzy::RizzyError;
use chrono_tz::America::{Chicago, New_York};
use chrono_tz::Tz;
use clap::{crate_authors, crate_description, crate_version, AppSettings, Parser};
use eyre::Context;

use crate::rizzy::Rizzy;

mod rizzy;

#[derive(Parser, Debug)]
#[clap(
setting = AppSettings::DeriveDisplayOrder,
about = crate_description ! (),
version = crate_version ! (),
author = crate_authors ! ()
)]
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
    /// Use FORMAT as the strftime format instead of RFC3339
    #[clap(short, long)]
    format: Option<String>,
    /// Read input from FILE (reads from stdin if not supplied)
    file: Vec<String>,
}

fn get_timezone(Opts { nyc, chi, zone, .. }: &Opts) -> Result<Tz, RizzyError> {
    match (nyc, chi, zone) {
        (true, true, _) => Err(RizzyError::InvalidArg(
            "Cannot use more than one timezone override".to_string(),
        )),
        (true, false, None) => Ok(New_York),
        (false, true, None) => Ok(Chicago),
        (true, false, Some(_)) | (false, true, Some(_)) => Err(RizzyError::InvalidArg(
            "Cannot supply --zone and an override".to_string(),
        )),
        (false, false, Some(tz_string)) => parse_timezone(&tz_string),
        (false, false, None) => Ok(Chicago),
    }
}

// Shamelessly borrowed from the rust-by-example read_lines example.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn run() -> eyre::Result<()> {
    let opts: Opts = Opts::parse();
    let timezone = get_timezone(&opts)?;
    let rizzy = Rizzy::new(timezone, opts.format, opts.convert_epoch_nanos);

    if opts.file.is_empty() {
        for line in io::stdin().lock().lines() {
            println!("{}", rizzy.handle_line(&line.unwrap()));
        }
    } else {
        for file in opts.file {
            let lines =
                read_lines(&file).with_context(|| format!("Failed to open file '{}'", file))?;
            for line in lines {
                println!("{}", rizzy.handle_line(&line.unwrap()));
            }
        }
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("[rizzy] Error: {e}");
        exit(1);
    }
}
