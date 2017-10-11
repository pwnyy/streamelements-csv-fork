use reqwest;
use toml;
use csv;

#[derive(Debug, Deserialize, Serialize)]
pub struct Alltime {
    pub _total: u64,
    users: Vec<User>
}

impl Alltime {
	pub fn users(&self) -> &Vec<User> {
		&self.users
	}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    username: String,
    points: u64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
	channel: String
}

impl ActualConfig {
	pub fn into_channel(self) -> String {
		self.info.channel
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActualConfig {
	info: Config,
}


#[derive(Debug, Error)]
pub enum Error {
	/// There was an error sending a request to the site, 
	/// possibly check your internet connection
	Reqwest(reqwest::Error),
	/// There was an error with the I/O of your system
	Io(::std::io::Error),
	/// Unable to read config
	TomlDeserialized(toml::de::Error),
	/// Error with CSV info
	Csv(csv::Error),
}