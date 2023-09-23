use crate::enums::datagram_type::DatagramType;
use crate::libs::types::Size;
use crate::libs::utils::{get_u16_at_pos, get_u32_at_pos, get_u64_at_pos};

// The datagram data is used to embed a payload to send information through a specific topic
pub struct DtgData{
    pub datagram_type: DatagramType, // 1 byte
    pub size: Size, // 2 bytes (u16)
    pub sequence_number: u32, // 4 bytes (u32)
    pub topic_id: u64, // 8 bytes (u64)
    pub payload: Vec<u8> // size bytes
}
impl DtgData {

    pub fn new(sequence_number: u32, topic_id: u64, payload: Vec<u8>)-> DtgData {
        DtgData {
            datagram_type: DatagramType::Data,
            size: payload.len() as u16,
            sequence_number,
            topic_id,
            payload
        }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(15 + self.size as usize);
        bytes.push(u8::from(self.datagram_type));
        bytes.extend(self.size.to_le_bytes());
        bytes.extend(self.sequence_number.to_le_bytes());
        bytes.extend(self.topic_id.to_le_bytes());
        bytes.extend(self.payload.iter());
        return bytes;
    }
}

impl<'a> TryFrom<&'a [u8]> for DtgData{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 15 {
            return Err("Payload len is to short for a DtgData.");
        }
        let size = get_u16_at_pos(buffer, 1)?;
        let sequence_number = get_u32_at_pos(buffer, 3)?;
        let topic_id = get_u64_at_pos(buffer, 7)?;

        Ok(DtgData {
            datagram_type: DatagramType::Data,
            size,
            sequence_number,
            topic_id,
            payload: buffer[15..15+size as usize].into(),
        })
    }
}