// This document contain the config system. It read values from a toml file and
// then it return it as a rust structure.
// @author : GuicLuca (lucasguichard127@gmail.com)
// date : 14/03/2023

use std::fs;
use std::io::Error;
use serde::{Deserialize, Serialize};
use toml;


// Contain the Server table of the toml file
#[derive(Serialize, Deserialize, Debug)]
struct ConfigTomlServer {
    port: Option<String>,
    packet_buffer_size: Option<u16>,
}

// Contain the Period table of the toml file
#[derive(Serialize, Deserialize, Debug)]
struct ConfigTomlPeriod {
    heartbeat_period: Option<u16>,
    ping_period: Option<u16>,
}

// Contain the Debug table of the toml file
#[derive(Serialize, Deserialize, Debug)]
struct ConfigTomlDebug {
    debug_level: Option<String>,
    debug_datagram_handler: Option<bool>,
    debug_ping_sender: Option<bool>,
    debug_data_handler: Option<bool>,
    debug_heartbeat_checker: Option<bool>,
    debug_topic_handler: Option<bool>,
    debug_client_manager: Option<bool>,
    debug_object_handler: Option<bool>,
}

// Used to load every table of the toml file
#[derive(Serialize, Deserialize, Debug)]
struct ConfigToml {
    server: Option<ConfigTomlServer>,
    debug: Option<ConfigTomlDebug>,
    period: Option<ConfigTomlPeriod>,
}

// This is the final structure that contain every
// values of the toml file.
#[derive(Debug)]
pub struct Config {
    pub port: u16,
    pub packet_buffer_size: u16,
    pub heart_beat_period: u16,
    pub ping_period: u16,
    pub debug_level: String,
    pub debug_datagram_handler: bool,
    pub debug_ping_sender: bool,
    pub debug_data_handler: bool,
    pub debug_heartbeat_checker: bool,
    pub debug_topic_handler: bool,
    pub debug_client_manager: bool,
    pub debug_object_handler: bool,
}

impl Config {
    pub fn new() -> Self {
        info!("Loading config.toml file...");
        // 1 - List of path to config files
        let config_filepath: [&str; 2] = [
            "./config.toml",
            "./src/config.toml",
        ];

        // 2 - Loop through each config file to get the first valid one
        let mut content: String = "".to_owned();

        for filepath in config_filepath {
            let result: Result<String, Error> = fs::read_to_string(filepath);

            if result.is_ok() {
                content = result.unwrap_or_else(|err|{
                    println!("Failed to unwrap content string. Error:\n{}", err);
                    "".to_owned() // return default value
                });
                break;
            }
        }

        info!("Reading config.toml file...");

        // 3 - Extract the content to a toml object
        let config_toml: ConfigToml = match toml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                println!("Failed to create ConfigToml Object out of config file. Error: {:?}", e);
                ConfigToml {
                    server: None,
                    period: None,
                    debug: None,
                }
            }
        };

        // 4 - Get every value into local variables
        info!("Creating server config table...");

        // 4.1 - Server variables
        let (port, packet_buffer_size): (u16, u16) = match config_toml.server {
            Some(server) => {
                let port: u16 =  match server.port.unwrap_or_else(|| {
                    println!("Missing field port in table server.");
                    "3838".to_owned()
                }).parse::<u16>() {
                    Ok(value) => {
                        value
                    }
                    Err(err) => {
                        println!("Failed to parse port into a u16. Error:\n{}", err);
                        3838 // return default value
                    }
                };

                let packet_buffer_size: u16 = server.packet_buffer_size.unwrap_or_else(|| {
                    println!("Missing field packet_buffer_size in table server.");
                    1000u16
                });

                (port, packet_buffer_size)
            }
            None => {
                println!("Missing table server.");
                (3838, 1000) // Default value if none found
            }
        };

        // 4.2 - Period variables
        info!("Creating period config table...");
        let (heartbeat_period, ping_period): (u16, u16) = match config_toml.period {
            Some(period) => {
                let hb_period = period.heartbeat_period.unwrap_or_else(|| {
                    println!("Missing field heartbeat_period in table period.");
                    5 // Default value if none found
                });
                let ping_period = period.ping_period.unwrap_or_else(|| {
                    println!("Missing field ping_period in table period.");
                    5 // Default value if none found
                });

                (hb_period, ping_period)
            }
            None => {
                println!("Missing table period.");
                (5, 5) // Default value if none found
            }
        };

        // 4.3 - Debug variables
        info!("Creating debug config table...");
        let (debug_level,
            debug_datagram_handler,
            debug_ping_sender,
            debug_data_handler,
            debug_heartbeat_checker,
            debug_topic_handler,
            debug_client_manager,
            debug_object_handler,): (String, bool, bool, bool, bool, bool, bool, bool) = match config_toml.debug {
            Some(debug) => {
                let d_level: String = debug.debug_level.unwrap_or_else(|| {
                    println!("Missing field debug_level in table debug.");
                    "trace".to_string() // Default value if none found
                });
                let d_ping = debug.debug_ping_sender.unwrap_or_else(|| {
                    println!("Missing field debug_ping_sender in table debug.");
                    true // Default value if none found
                });
                let d_datagram = debug.debug_datagram_handler.unwrap_or_else(|| {
                    println!("Missing field debug_datagram_handler in table debug.");
                    true // Default value if none found
                });
                let d_data = debug.debug_data_handler.unwrap_or_else(|| {
                    println!("Missing field debug_data_handler in table debug.");
                    true // Default value if none found
                });
                let d_heart = debug.debug_heartbeat_checker.unwrap_or_else(|| {
                    println!("Missing field debug_heartbeat_checker in table debug.");
                    true // Default value if none found
                });
                let d_topic = debug.debug_topic_handler.unwrap_or_else(|| {
                    println!("Missing field debug_topic_handler in table debug.");
                    true // Default value if none found
                });
                let d_manager = debug.debug_client_manager.unwrap_or_else(|| {
                    println!("Missing field debug_client_manager in table debug.");
                    true // Default value if none found
                });
                let d_object = debug.debug_object_handler.unwrap_or_else(|| {
                    println!("Missing field debug_object_handler in table debug.");
                    true // Default value if none found
                });

                (d_level, d_datagram, d_ping, d_data, d_heart, d_topic, d_manager, d_object)
            }
            None => {
                println!("Missing table debug.");
                ("trace".to_string(), true, true, true, true, true, true, true) // Default value if none found
            }
        };


        Config {
            port,
            packet_buffer_size,
            heart_beat_period: heartbeat_period,
            ping_period,
            debug_level,
            debug_datagram_handler,
            debug_ping_sender,
            debug_data_handler,
            debug_heartbeat_checker,
            debug_topic_handler,
            debug_client_manager,
            debug_object_handler
        }
    }
}