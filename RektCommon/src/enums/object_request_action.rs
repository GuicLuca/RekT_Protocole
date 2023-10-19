/**
 * ObjectRequestAction are all possible action in OBJECT_REQUEST datagram.
 */
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
#[no_mangle]
pub enum ObjectRequestAction {
    Create,
    Update,
    Delete,
    Subscribe,
    Unsubscribe,
    Unknown,
}


/**
 * This function convert an ObjectRequestAction to an u8
 *
 * @param value: ObjectRequestAction, The source to convert
 *
 * @return u8
 */
impl From<ObjectRequestAction> for u8 {
    fn from(value: ObjectRequestAction) -> Self {
        match value {
            ObjectRequestAction::Create => 0x01,
            ObjectRequestAction::Update => 0x02,
            ObjectRequestAction::Delete => 0x04,
            ObjectRequestAction::Subscribe => 0x08,
            ObjectRequestAction::Unsubscribe => 0x10,
            ObjectRequestAction::Unknown => 0xAA,
        }
    }
}

/**
 * This function convert an u8 to a ObjectRequestAction
 *
 * @param value: u8, The source to convert
 *
 * @return ObjectRequestAction
 */
impl From<u8> for ObjectRequestAction {
    fn from(value: u8) -> Self {
        match value {
            0x01 => ObjectRequestAction::Create,
            0x02 => ObjectRequestAction::Update,
            0x04 => ObjectRequestAction::Delete,
            0x08 => ObjectRequestAction::Subscribe,
            0x10 => ObjectRequestAction::Unsubscribe,
            _ => ObjectRequestAction::Unknown,
        }
    }
}