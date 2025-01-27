use crate::enums::datagram_type::DatagramType;
use crate::enums::topic_action::TopicAction;
use crate::enums::topic_response::TopicResponse;
use crate::libs::types::{Size, TopicId};
use crate::libs::utils::{get_bytes_from_slice, get_u16_at_pos, get_u64_at_pos};

//===== Sent to subscribe/unsubscribe to a topic
pub struct DtgTopicRequest {
    pub datagram_type: DatagramType, // 1 byte
    pub flag: TopicAction,           // 1 byte
    pub topic_id: TopicId,           // 8 bytes
}

//===== Sent to subscribe a topic
impl DtgTopicRequest {
    pub fn new(action: TopicAction, topic_id: TopicId) -> DtgTopicRequest {
        DtgTopicRequest {
            datagram_type: DatagramType::TopicRequest,
            flag: action,
            topic_id,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(DtgTopicRequest::get_default_byte_size());
        bytes.push(u8::from(self.datagram_type));
        bytes.push(u8::from(self.flag));
        bytes.extend(self.topic_id.to_le_bytes());
        bytes
    }

    pub const fn get_default_byte_size() -> usize {
        10
    }
}

impl<'a> TryFrom<&'a [u8]> for DtgTopicRequest {
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < DtgTopicRequest::get_default_byte_size() {
            return Err("Payload len is to short for a DtgTopicRequest.");
        }
        let topic_id = get_u64_at_pos(buffer, 2)?;

        Ok(DtgTopicRequest {
            datagram_type: DatagramType::from(buffer[0]),
            flag: TopicAction::from(buffer[1]),
            topic_id,
        })
    }
}

//===== Sent to acknowledge a TOPIC_REQUEST
pub struct DtgTopicRequestAck {
    pub datagram_type: DatagramType,
    pub flag: TopicResponse,
    pub topic_id: TopicId,
}

impl DtgTopicRequestAck {
    pub const fn new(topic_id: TopicId, status: TopicResponse) -> DtgTopicRequestAck {
        DtgTopicRequestAck {
            datagram_type: DatagramType::TopicRequestAck,
            flag: status,
            topic_id,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(DtgTopicRequestAck::get_default_byte_size());
        bytes.push(u8::from(self.datagram_type));
        bytes.push(u8::from(self.flag));
        bytes.extend(self.topic_id.to_le_bytes());
        bytes
    }

    pub const fn get_default_byte_size() -> usize {
        10
    }
}

impl<'a> TryFrom<&'a [u8]> for DtgTopicRequestAck {
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < DtgTopicRequestAck::get_default_byte_size() {
            return Err("Payload len is to short for a DtgTopicRequestAck.");
        }
        let topic_id = get_u64_at_pos(buffer, 2)?;

        Ok(DtgTopicRequestAck {
            datagram_type: DatagramType::from(buffer[0]),
            flag: TopicResponse::from(buffer[1]),
            topic_id,
        })
    }
}

pub struct DtgTopicRequestNack {
    pub datagram_type: DatagramType,
    pub size: Size,
    pub flag: TopicResponse,
    pub payload: Vec<u8>,
}

impl DtgTopicRequestNack {
    pub fn new(status: TopicResponse, error_message: &str) -> DtgTopicRequestNack {
        let size = error_message.len() as Size; // string length + 1 for the action
        DtgTopicRequestNack {
            datagram_type: DatagramType::TopicRequestNack,
            size,
            flag: status,
            payload: error_message.as_bytes().into(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> =
            Vec::with_capacity(DtgTopicRequestNack::get_default_byte_size() + self.size as usize);
        bytes.push(u8::from(self.datagram_type));
        bytes.extend(self.size.to_le_bytes());
        bytes.push(u8::from(self.flag));
        bytes.extend(self.payload.iter());

        bytes
    }

    pub const fn get_default_byte_size() -> usize {
        4
    }
}

impl<'a> TryFrom<&'a [u8]> for DtgTopicRequestNack {
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < DtgTopicRequestNack::get_default_byte_size() {
            return Err("Payload len is to short for a DtgTopicRequestNack.");
        }
        let size = get_u16_at_pos(buffer, 1)?;

        Ok(DtgTopicRequestNack {
            datagram_type: DatagramType::from(buffer[0]),
            flag: TopicResponse::from(buffer[3]),
            size,
            payload: get_bytes_from_slice(
                buffer,
                DtgTopicRequestNack::get_default_byte_size(),
                (DtgTopicRequestNack::get_default_byte_size() + size as usize - 1),
            ),
        })
    }
}
