use std::collections::HashSet;
use std::mem::size_of;
use crate::datagrams::miscellaneous_requests::DtgServerStatusACK;
use crate::enums::datagram_type::DatagramType;
use crate::enums::object_request_action::ObjectRequestAction;
use crate::libs::types::{Flag, ObjectId, Size, TopicId};
use crate::libs::utils::{get_bytes_from_slice, get_u16_at_pos, get_u64_at_pos, u8_to_vec_be};

//===== Sent to acknowledge a TOPIC_REQUEST
#[repr(C)]
pub struct DtgObjectRequest {
    pub datagram_type: DatagramType,
    pub size: Size,
    pub flag: ObjectRequestAction,
    pub object_id: ObjectId,
    pub payload: HashSet<TopicId>,
}

impl DtgObjectRequest {
    pub fn new(flag: ObjectRequestAction, object_id: ObjectId, topics: HashSet<TopicId>) -> DtgObjectRequest {
        let size: Size = (topics.len() * size_of::<TopicId>()) as Size; // x = size_of(topics)
        DtgObjectRequest {
            datagram_type: DatagramType::ObjectRequest,
            size,
            flag,
            object_id,
            payload: topics,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(DtgObjectRequest::get_default_byte_size() + self.size as usize); // 3 = DatagramType + size
        bytes.push(u8::from(self.datagram_type));
        bytes.extend(self.size.to_le_bytes());
        bytes.push(u8::from(self.flag));
        bytes.extend(self.object_id.to_le_bytes());
        // The following line convert a Vec<u64> to his representation as bytes (Vec<u8>)
        bytes.extend(self.payload.iter()
            .flat_map(|&x| {
                let bytes: [u8; 8] = x.to_le_bytes();
                bytes.into_iter()
            })
            .collect::<Vec<u8>>());
        return bytes;
    }

    pub const fn get_default_byte_size() -> usize { return 12; }
}

impl<'a> TryFrom<&'a [u8]> for DtgObjectRequest {
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < DtgObjectRequest::get_default_byte_size() {
            return Err("Payload len is to short for a DtgObjectRequest.");
        }
        let size = get_u16_at_pos(buffer, 1)?;

        let mut topics: HashSet<TopicId>;
        if size != 0 {
            topics = get_bytes_from_slice(buffer, DtgObjectRequest::get_default_byte_size(), (DtgObjectRequest::get_default_byte_size() + size as usize - 1))
                // Convert the bytes vector to a vector of topics id by grouping u8 into u64
                .chunks_exact(8)
                .map(|chunk| {
                    u64::from_le_bytes(chunk.try_into().unwrap())
                })
                .collect();
        } else {
            topics = HashSet::default();
        }

        let object_id = get_u64_at_pos(buffer, 4)?;

        Ok(DtgObjectRequest {
            datagram_type: DatagramType::from(buffer[0]),
            flag: ObjectRequestAction::from(buffer[3]),
            size,
            object_id,
            payload: topics,
        })
    }
}

//===== Sent to acknowledge a OBJECT_REQUEST create
#[repr(C)]
pub struct DtgObjectRequestACK {
    pub datagram_type: DatagramType,
    pub flag: Flag, // Bit field XXXA UDMC (X: Unused, D: delete, M : modify, C: Create, A: subscribe, U: unsubscribe)
    pub object_id: ObjectId,
    pub final_object_id: ObjectId,
}

impl DtgObjectRequestACK {
    pub fn new(flag: Flag, object_id: ObjectId, final_object_id: ObjectId) -> DtgObjectRequestACK {
        DtgObjectRequestACK {
            datagram_type: DatagramType::ObjectRequestAck,
            flag,
            object_id,
            final_object_id,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(DtgObjectRequestACK::get_default_byte_size());

        bytes.push(u8::from(self.datagram_type));
        bytes.push(u8::from(self.flag));
        bytes.extend(self.object_id.to_le_bytes());
        bytes.extend(self.final_object_id.to_le_bytes());

        return bytes;
    }

    pub const fn get_default_byte_size() -> usize { return 18; }
}

impl<'a> TryFrom<&'a [u8]> for DtgObjectRequestACK {
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < DtgObjectRequestACK::get_default_byte_size() {
            return Err("Payload len is to short for a DtgObjectRequestACK.");
        }
        let object_id = get_u64_at_pos(buffer, 2)?;
        let mut final_object_id = 0;

        if u8_to_vec_be(buffer[1])[7] == 1 {
            final_object_id = get_u64_at_pos(buffer, 10)?;
        }

        Ok(DtgObjectRequestACK {
            datagram_type: DatagramType::from(buffer[0]),
            flag: buffer[1],
            object_id,
            final_object_id,
        })
    }
}


// ===== Sent in case of error for all action (Create update delete)
#[repr(C)]
pub struct DtgObjectRequestNACK {
    pub datagram_type: DatagramType,
    pub size: Size,
    pub flag: u8, // Bitfield XXXA UDMC (X: Unused, D: delete, M : modify, C: Create, A: subscribe, U: unsubscribe)
    pub object_id: u64,
    pub payload: Vec<u8>,
}

impl DtgObjectRequestNACK {
    pub fn new(flag: u8, object_id: u64, reason: &str) -> DtgObjectRequestNACK {
        let reason_vec: Vec<u8> = reason.as_bytes().into();
        DtgObjectRequestNACK {
            datagram_type: DatagramType::ObjectRequestNack,
            size: reason_vec.len() as Size,
            flag,
            object_id,
            payload: reason_vec,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(DtgObjectRequestNACK::get_default_byte_size() + self.size as usize);
        bytes.push(u8::from(self.datagram_type));
        bytes.extend(self.size.to_le_bytes());
        bytes.push(u8::from(self.flag));
        bytes.extend(self.object_id.to_le_bytes());
        bytes.extend(self.payload.iter());

        return bytes;
    }

    pub const fn get_default_byte_size() -> usize { return 12; }
}

impl<'a> TryFrom<&'a [u8]> for DtgObjectRequestNACK {
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < DtgObjectRequestNACK::get_default_byte_size() {
            return Err("Payload len is to short for a DtgObjectRequestNACK.");
        }
        let size = get_u16_at_pos(buffer, 1)?;
        let object_id = get_u64_at_pos(buffer, 4)?;

        Ok(DtgObjectRequestNACK {
            datagram_type: DatagramType::from(buffer[0]),
            flag: buffer[3],
            size,
            object_id,
            payload: get_bytes_from_slice(buffer, DtgObjectRequestNACK::get_default_byte_size(), (DtgObjectRequestNACK::get_default_byte_size() + size as usize - 1)),
        })
    }
}