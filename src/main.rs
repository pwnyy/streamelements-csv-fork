#![allow(unused_variables)]
extern crate chrono;
extern crate csv;
#[macro_use]
extern crate derive_error;
#[macro_use]
extern crate log;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate simplelog;
extern crate toml;

use chrono::offset::Utc;
use csv::WriterBuilder;
use std::fs::File;

mod data;
use data::{ActualConfig, Alltime, Error, User};

const BASE_URL: &str = "https://api.streamelements.com/kappa/v1";

type Result<T> = std::result::Result<T, Error>;

fn main() {
    match run() {
        Ok(_) => println!("Program completed without errors"),
        Err(e) => {eprintln!("Program errored: {}", e); error!("Error: {}", e) }
    }
}

fn run() -> Result<()> {
    let config = load_toml()?;
    let channel = config.channel();
    info!("loaded channel: {}", channel);
    let request = reqwest::Client::new();
    trace!("created reqwest client");

    let alltime_url = format!("{}/points/{}/alltime/1000", BASE_URL, channel);
    let top_url = format!("{}/points/{}/top/1000", BASE_URL, channel);
    
    let response: Alltime = request.get(&top_url).send()?.json()?;
    info!("received response from streamelements api");
    
    let today = Utc::today().format("%d-%m-%Y");
    info!("date for filename: {}", today);
    
    let mut csv = WriterBuilder::new()
        .has_headers(false)
        .from_path(format!("{}-top-points.csv", today))?;
    info!("successfully created csv writer");
    write_to_csv(&mut csv, &response.users())?;

    // We request 1000 initially, if the total is less
    // then we found them all
    if response._total < 1000 {
        return Ok(());
    }
    // Rounded up integer division, rust rounds towards
    // zero as that is what llvm does.
    // We are requesting the max number of records, 1000
    let offset_count = ((response._total - 1) / 1000) + 1;
    for offset in 2..offset_count + 1 {
        let offset = offset * 1000;
        let resp: Alltime = request
            .get(&format!("{}?offset={}", top_url, offset))
            .send()?
            .json()?;
        info!("received response from streamelements api");

        // Not sure how to build around this without duplicating code
        // looks very messy
        match config.cutoff() {
            Some(cutoff) => {
                let last_point = resp.users().last().unwrap().points;
                if last_point < cutoff {
                    let filtered: Vec<User> = resp.into_users()
                                       .into_iter()
                                       .filter(|user| user.points > cutoff)
                                       .collect();
                    write_to_csv(&mut csv, filtered.as_slice())?;
                    break
                }
            }
            None => {}
        }
        write_to_csv(&mut csv, &resp.users())?;
        info!("successfully wrote to csv");
    }
    Ok(())
}

/// Convenience function because Toml doesn't support reading from a file
/// serde_json
fn load_toml() -> Result<ActualConfig> {
    use std::io::Read;
    let mut file = File::open("./config.toml")?;
    let mut str_bufr = String::new();
    file.read_to_string(&mut str_bufr)?;
    Ok(toml::from_str(&str_bufr)?)
}

/// Convenience function to serialize to a CSV via an iterator.
/// TODO: Generics
fn write_to_csv(csv: &mut csv::Writer<File>, users: &[User]) -> Result<()> {
    for user in users {
        csv.serialize(user)?;
    }
    Ok(())
}
