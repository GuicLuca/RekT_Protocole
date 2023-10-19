use std::mem::size_of;
use crate::enums::datagram_type::DatagramType;
use crate::libs::types::ClientId;
use crate::libs::utils::get_u64_at_pos;

//===== Sent to know the server status
#[repr(C)]
pub struct DtgServerStatus {
    pub datagram_type: DatagramType,
}

impl DtgServerStatus {
    pub const fn new() -> DtgServerStatus {
        DtgServerStatus {
            datagram_type: DatagramType::ServerStatus
        }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        return [u8::from(self.datagram_type)].into();
    }
}

impl From<&[u8]> for DtgServerStatus {
    fn from(buffer: &[u8]) -> Self {
        DtgServerStatus {
            datagram_type: DatagramType::from(buffer[0]),
        }
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

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(1 + size_of::<ClientId>());
        bytes.push(u8::from(self.datagram_type));
        bytes.extend(self.connected_client.to_le_bytes());

        return bytes;
    }
}

impl<'a> TryFrom<&'a [u8]> for DtgServerStatusACK {
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 9 {
            return Err("Payload len is to short for a DtgServerStatusACK.");
        }
        let connected_client = get_u64_at_pos(buffer, 1)?;
        Ok(DtgServerStatusACK {
            datagram_type: DatagramType::from(buffer[0]),
            connected_client,
        })
    }
}