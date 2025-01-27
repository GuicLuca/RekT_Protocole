use crate::enums::datagram_type::DatagramType;
use crate::libs::types::ClientId;
use crate::libs::utils::get_u64_at_pos;
use std::mem::size_of;

//===== Sent to know the server status
#[repr(C)]
pub struct DtgServerStatus {
    pub datagram_type: DatagramType,
}

impl DtgServerStatus {
    pub const fn new() -> DtgServerStatus {
        DtgServerStatus {
            datagram_type: DatagramType::ServerStatus,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        [u8::from(self.datagram_type)].into()
    }

    pub const fn get_default_byte_size() -> usize {
        1
    }
}

impl<'a> TryFrom<&'a [u8]> for DtgServerStatus {
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < DtgServerStatus::get_default_byte_size() {
            return Err("Payload len is to short for a DtgServerStatus.");
        }

        Ok(DtgServerStatus {
            datagram_type: DatagramType::from(buffer[0]),
        })
    }
}

//===== Sent to answer a ServerStatus request
#[repr(C)]
pub struct DtgServerStatusACK {
    pub datagram_type: DatagramType,
    pub connected_client: ClientId, // Amount of connected client. It use the same type as client_id to ensure sufficient capacity
}

impl DtgServerStatusACK {
    pub const fn new(nb_client: ClientId) -> DtgServerStatusACK {
        DtgServerStatusACK {
            datagram_type: DatagramType::ServerStatusAck,
            connected_client: nb_client,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(DtgServerStatusACK::get_default_byte_size());
        bytes.push(u8::from(self.datagram_type));
        bytes.extend(self.connected_client.to_le_bytes());

        bytes
    }

    pub const fn get_default_byte_size() -> usize {
        9
    }
}

impl<'a> TryFrom<&'a [u8]> for DtgServerStatusACK {
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < DtgServerStatusACK::get_default_byte_size() {
            return Err("Payload len is to short for a DtgServerStatusACK.");
        }
        let connected_client = get_u64_at_pos(buffer, 1)?;
        Ok(DtgServerStatusACK {
            datagram_type: DatagramType::from(buffer[0]),
            connected_client,
        })
    }
}
