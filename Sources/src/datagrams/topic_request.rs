use crate::enums::datagram_type::DatagramType;
use crate::enums::topic_actions::TopicsAction;
use crate::enums::topics_response::TopicsResponse;
use crate::libs::types::Size;
use crate::libs::utils::{get_bytes_from_slice, get_u16_at_pos, get_u64_at_pos};

//===== Sent to subscribe/unsubscribe to a topic
pub struct DtgTopicRequest {
    pub datagram_type: DatagramType, // 1 byte
    pub flag: TopicsAction, // 1 byte
    pub topic_id: u64, // 8 bytes
}

//===== Sent to subscribe a topic
impl DtgTopicRequest {
    pub fn new(action: TopicsAction, topic_id: u64) -> DtgTopicRequest {
        DtgTopicRequest {
            datagram_type: DatagramType::TopicRequest,
            flag: action,
            topic_id
        }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(10);
        bytes.push(u8::from(self.datagram_type));
        bytes.push(u8::from(self.flag));
        bytes.extend(self.topic_id.to_le_bytes());
        return bytes;
    }
}

impl<'a> TryFrom<&'a [u8]> for DtgTopicRequest{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 10 {
            return Err("Payload len is to short for a DtgTopicRequest.");
        }
        let topic_id = get_u64_at_pos(buffer, 2)?;

        Ok(DtgTopicRequest {
            datagram_type: DatagramType::from(buffer[0]),
            flag: TopicsAction::from(buffer[1]),
            topic_id
        })
    }
}

//===== Sent to acknowledge a TOPIC_REQUEST
pub struct DtgTopicRequestAck {
    pub datagram_type: DatagramType,
    pub flag: TopicsResponse,
    pub topic_id: u64,
}

impl DtgTopicRequestAck {
    pub const fn new(topic_id: u64, status: TopicsResponse) -> DtgTopicRequestAck {
        DtgTopicRequestAck {
            datagram_type: DatagramType::TopicRequestAck,
            flag: status,
            topic_id
        }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(10);
        bytes.push(u8::from(self.datagram_type));
        bytes.push(u8::from(self.flag));
        bytes.extend(self.topic_id.to_le_bytes());
        return bytes;
    }
}

impl<'a> TryFrom<&'a [u8]> for DtgTopicRequestAck{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 10 {
            return Err("Payload len is to short for a DtgTopicRequestAck.");
        }
        let topic_id = get_u64_at_pos(buffer, 2)?;

        Ok(DtgTopicRequestAck {
            datagram_type: DatagramType::from(buffer[0]),
            flag: TopicsResponse::from(buffer[1]),
            topic_id
        })
    }
}

pub struct DtgTopicRequestNack {
    pub datagram_type: DatagramType,
    pub flag: TopicsResponse,
    pub size: Size,
    pub payload: Vec<u8>
}

impl DtgTopicRequestNack{

    pub fn new(status: TopicsResponse, error_message: &str) -> DtgTopicRequestNack {
        let size = error_message.len() as u16; // string length + 1 for the action
        DtgTopicRequestNack {
            datagram_type: DatagramType::TopicRequestNack,
            flag: status,
            size,
            payload: error_message.as_bytes().into()
        }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(4 + self.size as usize);
        bytes.push(u8::from(self.datagram_type));
        bytes.push(u8::from(self.flag));
        bytes.extend(self.size.to_le_bytes());
        bytes.extend(self.payload.iter());

        return bytes;
    }
}

impl<'a> TryFrom<&'a [u8]> for DtgTopicRequestNack{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 10 {
            return Err("Payload len is to short for a DtgTopicRequestNack.");
        }
        let size = get_u16_at_pos(buffer, 2)?;

        Ok(DtgTopicRequestNack {
            datagram_type: DatagramType::from(buffer[0]),
            flag: TopicsResponse::from(buffer[1]),
            size,
            payload: get_bytes_from_slice(buffer, 4, (4 + size) as usize)
        })
    }
}