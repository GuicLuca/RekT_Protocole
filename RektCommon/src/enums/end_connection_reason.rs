/**
 * End connexion reasons are used to
 * detail the reason of the shutdown request.
 */
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
#[no_mangle]
pub enum EndConnexionReason {
    Shutdown,
    TimeOut,
    Unknown,
}

/**
 * This function convert an u8 to an EndConnexionReason
 *
 * @param value: u8, The source to convert
 *
 * @return EndConnexionReason
 */
impl From<u8> for EndConnexionReason {
    fn from(value: u8) -> Self {
        match value {
            0x00 => EndConnexionReason::Shutdown,
            0x01 => EndConnexionReason::TimeOut,
            _ => EndConnexionReason::Unknown,
        }
    }
}

/**
 * This function convert an EndConnexionReason to an u8
 *
 * @param value: EndConnexionReason, The source to convert
 *
 * @return u8
 */
impl From<EndConnexionReason> for u8 {
    fn from(value: EndConnexionReason) -> Self {
        match value {
            EndConnexionReason::Shutdown => 0x00,
            EndConnexionReason::TimeOut => 0x01,
            EndConnexionReason::Unknown => 0xAA,
        }
    }
}
