use crate::enums::datagram_type::DatagramType;
use crate::enums::end_connection_reason::EndConnexionReason;

//===== Sent to close the connexion between peer and broker
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
        let mut bytes: Vec<u8> = Vec::with_capacity(2);
        bytes.push(u8::from(self.datagram_type));
        bytes.push(u8::from(self.reason));
        return bytes;
    }
}
impl<'a> TryFrom<&'a [u8]> for DtgShutdown{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 2 {
            return Err("Payload len is to short for a DtgShutdown.");
        }

        Ok(DtgShutdown {
            datagram_type: DatagramType::from(buffer[0]),
            reason: EndConnexionReason::from(buffer[1])
        })
    }
}