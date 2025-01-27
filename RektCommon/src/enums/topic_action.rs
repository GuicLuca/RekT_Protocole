/**
 * Topics action are all actions that
 * a peer can do in a TOPICS_REQUEST
 */
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
#[no_mangle]
pub enum TopicAction {
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
impl From<TopicAction> for u8 {
    fn from(value: TopicAction) -> Self {
        match value {
            TopicAction::Subscribe => 0x00,
            TopicAction::Unsubscribe => 0xFF,
            TopicAction::Unknown => 0xAA,
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
impl From<u8> for TopicAction {
    fn from(value: u8) -> Self {
        match value {
            0x00 => TopicAction::Subscribe,
            0xFF => TopicAction::Unsubscribe,
            _ => TopicAction::Unknown,
        }
    }
}
