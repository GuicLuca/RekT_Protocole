use crate::enums::datagram_type::DatagramType;
use crate::enums::end_connection_reason::EndConnexionReason;

//===== Sent to close the connexion between peer and broker
#[repr(C)]
pub struct DtgShutdown {
    pub datagram_type: DatagramType,
    pub reason: EndConnexionReason,
}

impl DtgShutdown {
    pub const fn new(reason: EndConnexionReason) -> DtgShutdown {
        DtgShutdown {
            datagram_type: DatagramType::Shutdown,
            reason
        }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(DtgShutdown::get_default_byte_size());
        bytes.push(u8::from(self.datagram_type));
        bytes.push(u8::from(self.reason));
        return bytes;
    }

    pub const fn get_default_byte_size() -> usize { return 2; }
}
impl<'a> TryFrom<&'a [u8]> for DtgShutdown{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < DtgShutdown::get_default_byte_size() {
            return Err("Payload len is to short for a DtgShutdown.");
        }

        Ok(DtgShutdown {
            datagram_type: DatagramType::from(buffer[0]),
            reason: EndConnexionReason::from(buffer[1])
        })
    }
}