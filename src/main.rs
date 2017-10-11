#![allow(unused_variables)]
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate derive_error;
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate toml;
extern crate reqwest;
extern crate serde_json;
extern crate csv;
extern crate chrono;

use chrono::offset::Utc;
use csv::WriterBuilder;
use std::fs::File;

mod data;
use data::{Alltime, ActualConfig, Error, User};

const BASE_URL: &str = "https://api.streamelements.com/kappa/v1";

type Result<T> = std::result::Result<T, Error>;

fn main() {
    match run() {
    	Ok(_) => println!("Program completed without errors"),
    	Err(e) => eprintln!("Program errored: {}", e)
    }
}

fn run() -> Result<()> {
	let channel = load_toml()?.into_channel();
	info!("loaded channel: {}", channel);
	let request = reqwest::Client::new();
	trace!("created reqwest client");
	let alltime_url = format!("{}/points/{}/alltime/1000", BASE_URL, channel);
	let top_url  = format!("{}/points/{}/top/1000", BASE_URL, channel);
	let response: Alltime = request.get(&top_url)
								   .send()?
								   .json()?;
	info!("received response from streamelements api");
	let today = Utc::today()
					.format("%d-%m-%Y");

	info!("date for filename: {}", today);
	let mut csv = WriterBuilder::new()
								.has_headers(false)
								.from_path(format!("{}-top-points.csv", today))?;
	info!("successfully created csv writer");
	write_to_csv(&mut csv, &response.users())?;
	if response._total < 1000 {
		return Ok(())
	}
	// Rounded up integer division, rust rounds towards
	// zero as that is what llvm does.
	// We are requesting the max number of records, 1000
	let offset_count = ((response._total - 1) / 1000) + 1;
	for offset in 2..offset_count+1 {
		let resp: Alltime = request.get(&format!("{}?offset={}", top_url, offset))
			   			  		   .send()?
			   			  		   .json()?;
		info!("received response from streamelements api");
		write_to_csv(&mut csv, &resp.users())?;
		info!("successfully wrote to csv");
	}
	Ok(())
}

fn load_toml() -> Result<ActualConfig> {
	use std::io::Read;
	let mut file = File::open("./config.toml")?;
	let mut str_bufr = String::new();
	file.read_to_string(&mut str_bufr)?;
	Ok(toml::from_str(&str_bufr)?)
}

fn write_to_csv(csv: &mut csv::Writer<File>, users: &[User]) -> Result<()> {
	for user in users {
		csv.serialize(user)?;
	}
	Ok(())
}