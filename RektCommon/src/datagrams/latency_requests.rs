use crate::enums::datagram_type::DatagramType;
use crate::libs::types::PingId;

//===== Sent to measure the latency between peer and broker
#[repr(C)]
pub struct DtgPing {
    pub datagram_type: DatagramType,
    pub ping_id: PingId,
}

impl DtgPing {
    pub const fn new(ping_id: PingId) -> DtgPing {
        DtgPing {
            datagram_type: DatagramType::Ping,
            ping_id,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(DtgPing::get_default_byte_size());
        bytes.push(u8::from(self.datagram_type));
        bytes.push(self.ping_id);
        bytes
    }

    pub const fn get_default_byte_size() -> usize {
        2
    }
}

impl<'a> TryFrom<&'a [u8]> for DtgPing {
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < DtgPing::get_default_byte_size() {
            return Err("Payload len is to short for a DtgPing.");
        }

        Ok(DtgPing {
            datagram_type: DatagramType::from(buffer[0]),
            ping_id: buffer[1],
        })
    }
}

//===== Sent to answer a ping request.
#[repr(C)]
pub struct DtgPong {
    pub datagram_type: DatagramType,
    pub ping_id: PingId,
}

impl DtgPong {
    pub const fn new(ping_id: PingId) -> DtgPong {
        DtgPong {
            datagram_type: DatagramType::Pong,
            ping_id,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(DtgPong::get_default_byte_size());
        bytes.push(u8::from(self.datagram_type));
        bytes.push(self.ping_id);
        bytes
    }

    pub const fn get_default_byte_size() -> usize {
        2
    }
}

impl<'a> TryFrom<&'a [u8]> for DtgPong {
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < DtgPong::get_default_byte_size() {
            return Err("Payload len is to short for a DtgPong.");
        }

        Ok(DtgPong {
            datagram_type: DatagramType::from(buffer[0]),
            ping_id: buffer[1],
        })
    }
}
