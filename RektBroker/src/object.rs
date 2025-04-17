use dashmap::DashSet;
use rekt_lib::libs::types::{ObjectId, TopicId};
use tokio::sync::RwLock;
use crate::clients::client::ConnectionId;
use crate::prelude::ClientSenderMap;

#[derive(Debug)]
pub struct Object{
    pub id: ObjectId,
    pub topic_list: RwLock<Vec<TopicId>>,
    // No need to store the client sender as in topic because we only need
    // to save subscribers. Only topics care about the sending data.
    pub subscribers: DashSet<ConnectionId>,
}