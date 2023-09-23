use std::error::Error;

// ===================
//  Common used types
// ===================
pub type Size = u16;
pub type Flag = u8; // used to normalize the size of the enum used as flag
pub type TopicId = u64;
pub type PingId = u8;
pub type ObjectId = u64; // 0..2 for type identifier (User generated, broker, temporary)  2..64 identifier

// ===================
//    Server types
// ===================
// pub type ServerSocket = Arc<UdpSocket>;
// pub type ClientsHashMap<T> = Arc<RwLock<HashMap<ClientId, T>>>;
// pub type TopicsHashMap<T> = Arc<RwLock<HashMap<TopicId, T>>>;
// pub type PingsHashMap = Arc<Mutex<HashMap<PingId, u128>>>;
// pub type ObjectHashMap<T> = Arc<RwLock<HashMap<ObjectId, T>>>;
//
//
// ===================
//   Clients types
// ===================
pub type ClientId = u64;
// pub type Responder<T> = oneshot::Sender<Result<T, Error>>;
// pub type ClientSender = Sender<ClientActions>;