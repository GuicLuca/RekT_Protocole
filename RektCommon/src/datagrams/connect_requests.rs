use std::mem::size_of;

use crate::enums::datagram_type::DatagramType;
use crate::libs::types::{ClientId, Size};
use crate::libs::utils::{get_bytes_from_slice, get_u16_at_pos, get_u64_at_pos};

// Sent to the broker to start a connection
#[repr(C)]
pub struct DtgConnect {
    pub datagram_type: DatagramType,
}

impl DtgConnect {
    pub const fn new() -> DtgConnect {
        DtgConnect {
            datagram_type: DatagramType::Connect,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        [u8::from(self.datagram_type)].into()
    }

    pub const fn get_default_byte_size() -> usize {
        1
    }
}

impl<'a> TryFrom<&'a [u8]> for DtgConnect {
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < DtgConnect::get_default_byte_size() {
            return Err("Payload len is to short for a DtgConnect.");
        }

        Ok(DtgConnect {
            datagram_type: DatagramType::from(buffer[0]),
        })
    }
}

//===== Sent to acknowledge the connexion with success

#[repr(C)]
pub struct DtgConnectAck {
    pub datagram_type: DatagramType,
    pub peer_id: ClientId,
    pub heartbeat_period: u16,
}

impl DtgConnectAck {
    pub const fn new(peer_id: ClientId, heartbeat_period: u16) -> DtgConnectAck {
        DtgConnectAck {
            datagram_type: DatagramType::ConnectAck,
            peer_id,
            heartbeat_period,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(DtgConnectAck::get_default_byte_size());
        bytes.push(u8::from(self.datagram_type));
        bytes.extend(self.peer_id.to_le_bytes().into_iter());
        bytes.extend(self.heartbeat_period.to_le_bytes().into_iter());

        bytes
    }

    pub fn get_default_byte_size() -> usize {
        11
    }
}

impl<'a> TryFrom<&'a [u8]> for DtgConnectAck {
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < DtgConnectAck::get_default_byte_size() {
            return Err("Payload len is to short for a DtgConnectAck.");
        }

        let peer_id = get_u64_at_pos(buffer, 1)?;
        let heartbeat_period = get_u16_at_pos(buffer, 9)?;

        Ok(DtgConnectAck {
            datagram_type: DatagramType::from(buffer[0]),
            peer_id,
            heartbeat_period,
        })
    }
}

#[repr(C)]
pub struct DtgConnectNack {
    pub datagram_type: DatagramType,
    pub size: Size,
    pub payload: Vec<u8>,
}

impl DtgConnectNack {
    pub fn new(message: &str) -> DtgConnectNack {
        let reason: Vec<u8> = message.as_bytes().into();
        let message_size = reason.len() as Size;

        DtgConnectNack {
            datagram_type: DatagramType::ConnectNack,
            size: message_size,
            payload: reason,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> =
            Vec::with_capacity(DtgConnectNack::get_default_byte_size() + self.size as usize);
        bytes.push(u8::from(self.datagram_type));
        bytes.extend(self.size.to_le_bytes().into_iter());
        bytes.extend(&mut self.payload.iter());

        bytes
    }

    pub const fn get_default_byte_size() -> usize {
        3
    }
}

impl<'a> TryFrom<&'a [u8]> for DtgConnectNack {
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < DtgConnectNack::get_default_byte_size() {
            return Err("Payload len is to short for a RQ_Connect_Ack_Error.");
        }
        let size = get_u16_at_pos(buffer, 1)?;

        Ok(DtgConnectNack {
            datagram_type: DatagramType::from(buffer[0]),
            size,
            payload: get_bytes_from_slice(
                buffer,
                DtgConnectNack::get_default_byte_size(),
                (DtgConnectNack::get_default_byte_size() + size as usize),
            ),
        })
    }
}
