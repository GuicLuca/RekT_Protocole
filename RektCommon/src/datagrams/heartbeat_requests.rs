use crate::enums::datagram_type::DatagramType;

//===== Sent to maintain the connexion
#[repr(C)]
pub struct DtgHeartbeat {
    pub datagram_type: DatagramType,
}

impl DtgHeartbeat {
    pub const fn new() -> DtgHeartbeat {
        DtgHeartbeat {
            datagram_type: DatagramType::Heartbeat
        }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        return [u8::from(self.datagram_type)].into();
    }

    pub const fn get_default_byte_size() -> usize { return 1; }
}

impl<'a> TryFrom<&'a [u8]> for DtgHeartbeat {
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < DtgHeartbeat::get_default_byte_size() {
            return Err("Payload len is to short for a DtgHeartbeat.");
        }

        Ok(DtgHeartbeat {
            datagram_type: DatagramType::from(buffer[0]),
        })
    }
}

//===== Sent to request a Heartbeat if a pear do not receive his
// normal heartbeat.
#[repr(C)]
pub struct DtgHeartbeatRequest {
    pub datagram_type: DatagramType,
}

impl DtgHeartbeatRequest {
    pub const fn new() -> DtgHeartbeatRequest {
        DtgHeartbeatRequest {
            datagram_type: DatagramType::HeartbeatRequest
        }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        return [u8::from(self.datagram_type)].into();
    }

    pub const fn get_default_byte_size() -> usize { return 1; }
}

impl<'a> TryFrom<&'a [u8]> for DtgHeartbeatRequest {
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < DtgHeartbeatRequest::get_default_byte_size() {
            return Err("Payload len is to short for a DtgHeartbeatRequest.");
        }

        Ok(DtgHeartbeatRequest {
            datagram_type: DatagramType::from(buffer[0]),
        })
    }
}