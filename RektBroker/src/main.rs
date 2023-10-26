// This document contain the main task of the broker. The task datagram_handler
// must never be blocked by any method ! The whole project use tokio and work
// asynchronously to allow a maximum bandwidth computing. The goal of the broker is
// to handle 100Gb/s.
// For each features the memory management and the cpu usage should be in the middle of the reflexion.
//
// @author : GuicLuca (lucasguichard127@gmail.com)
// date : 21/10/2023\

extern crate pretty_env_logger;
#[macro_use] extern crate log;

use lazy_static::lazy_static;
use local_ip_address::local_ip;
use crate::config::Config;


mod config;


lazy_static! {
    //static ref ISRUNNING: Arc<RwLock<bool>> = Arc::from(RwLock::from(false)); // Flag showing if the server is running or not
    static ref CONFIG: Config = Config::new(); // Unique reference to the config object
/*
    // List of client's :
    static ref CLIENTS_SENDERS_REF: ClientsHashMap<ClientSender> = Arc::new(RwLock::new(HashMap::default())); // <Client ID, Sender> -> the sender is used to sent command through the mpsc channels
    static ref CLIENTS_STRUCTS_REF: ClientsHashMap<Arc<Mutex<Client>>> = Arc::new(RwLock::new(HashMap::default())); // <Client ID, Struct> -> used only to keep struct alive
    static ref CLIENTS_ADDRESSES_REF: ClientsHashMap<SocketAddr> = Arc::new(RwLock::new(HashMap::default())); // <Client ID, address> -> Used to send data

    // List of time reference for ping requests
    static ref PINGS_REF: PingsHashMap = Arc::new(Mutex::new(HashMap::default())); // <Ping ID, reference time in ms>

    // List of topic subscribers
    static ref TOPICS_SUBSCRIBERS_REF: TopicsHashMap<HashSet<ClientId>> = Arc::new(RwLock::new(HashMap::default())); // <Topic ID, [Clients ID]>

    // List of Objects (group of topics)
    static ref OBJECTS_TOPICS_REF: ObjectHashMap<HashSet<TopicId>> = Arc::new(RwLock::new(HashMap::default())); // <ObjectId, [TopicId]>
    static ref OBJECT_SUBSCRIBERS_REF: ObjectHashMap<HashSet<ClientId>>  = Arc::new(RwLock::new(HashMap::default())); // <ObjectId, [ClientId]>*/
}

fn main(){
    // Set the rust log environment variable and then init the rust logger
    std::env::set_var("RUST_LOG", &CONFIG.debug_level);
    pretty_env_logger::init();

    info!("Static variables and configuration initialized ...");
    info!("Log level set to {} ...", &CONFIG.debug_level);
    info!("- Check config.toml file to change the config. -");

    // Fetch the local ip address and config port to start the server
    match local_ip() {
        Ok(ip) => info!("Server starting on {}:{} ...", ip, &CONFIG.port),
        Err(err) => {
            error!("Failed to get local IP:\n{}", err);
            error!("\n\nServer stopping!");
            return; // can't start the server if the local ip can't be reach
        }
    };


}
