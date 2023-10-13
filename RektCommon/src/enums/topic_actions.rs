/**
 * Topics action are all actions that
 * a peer can do in a TOPICS_REQUEST
 */
#[derive(Copy, Clone)]
#[repr(u8)]
#[no_mangle]
pub enum TopicsAction {
    Subscribe,
    Unsubscribe,
    Unknown,
}

/**
 * This function convert a TopicsAction to an u8
 *
 * @param value: TopicsActions, The source to convert
 *
 * @return u8
 */
impl From<TopicsAction> for u8 {
    fn from(value: TopicsAction) -> Self {
        match value {
            TopicsAction::Subscribe => 0x00,
            TopicsAction::Unsubscribe => 0xFF,
            TopicsAction::Unknown => 0xAA,
        }
    }
}

/**
 * This function convert an u8 to a TopicsActions
 *
 * @param value: u8, The source to convert
 *
 * @return TopicsActions
 */
impl From<u8> for TopicsAction {
    fn from(value: u8) -> Self {
        match value {
            0x00 => TopicsAction::Subscribe,
            0xFF => TopicsAction::Unsubscribe,
            _ => TopicsAction::Unknown
        }
    }
}