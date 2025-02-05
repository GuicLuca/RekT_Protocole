use dashmap::DashMap;
pub use rekt_lib::libs::types::ClientId;
use std::sync::Arc;

use crate::clients::client::{Client, ConnectionId};
pub use crate::config::Config;
pub use crate::errors::Error;

pub type Result<T> = core::result::Result<T, Error>;

/*=== Server types ===*/
pub type ServerSocket = Arc<tokio::net::UdpSocket>;
pub type ClientMap = Arc<DashMap<ConnectionId, Arc<Client>>>;
// pub type ClientsHashMap<T> = Arc<RwLock<HashMap<ClientId, T>>>;
// pub type TopicsHashMap<T> = Arc<RwLock<HashMap<TopicId, T>>>;
// pub type PingsHashMap = Arc<Mutex<HashMap<PingId, u128>>>;
// pub type ObjectHashMap<T> = Arc<RwLock<HashMap<ObjectId, T>>>;

/*=== Clients types ===*/
// pub type Responder<T> = oneshot::Sender<Result<T, Error>>;
// pub type ClientSender = Sender<ClientActions>;
