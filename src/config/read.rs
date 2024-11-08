use std::{fs::{self, File}, io::Write, path::Path};

use serde_json::Error;

use super::structs::Config;

use std::env;

const KEY: &str = "HOME";
const DEFAULT_CONFIG: &[u8] = include_bytes!("config.json");

pub fn read_config() -> Config {
    let home_dir = env::var_os(KEY).unwrap();
    let home_dir = Path::new(&home_dir);

    // Attempt to read from the config file
    let contents = fs::read_to_string(home_dir.join(".config/SolarCar/config.json"));

    // If config file does not exist, create one from the default one
    if contents.is_err() {
        let config_directory = home_dir.join(".config/SolarCar/");
        let _ = fs::create_dir(&config_directory);
    
        let mut file = File::create(Path::join(&config_directory, "config.json")).unwrap();
        file.write(DEFAULT_CONFIG).expect("Config could not be found or created.");
    }

    let contents = contents.unwrap_or(String::from_utf8(DEFAULT_CONFIG.to_vec()).unwrap());
    let json_output: Result<Config, Error> = serde_json::from_str(&contents);
    return json_output.expect("Invalid config file");
}