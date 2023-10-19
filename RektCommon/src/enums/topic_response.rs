/**
 * Topics response are all possible responses
 * type to a TOPICS_REQUEST
 */
#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
#[no_mangle]
pub enum TopicResponse {
    SubSuccess,
    SubFailure,
    UnsubSuccess,
    UnsubFailure,
    Unknown,
}

/**
 * This function convert an TopicsResponse to an u8
 *
 * @param value: TopicsResponse, The source to convert
 *
 * @return u8
 */
impl From<TopicResponse> for u8 {
    fn from(value: TopicResponse) -> Self {
        match value {
            TopicResponse::SubSuccess => 0x00,
            TopicResponse::SubFailure => 0x0F,
            TopicResponse::UnsubSuccess => 0xF0,
            TopicResponse::UnsubFailure => 0xFF,
            TopicResponse::Unknown => 0xAA,
        }
    }
}

/**
 * This function convert an u8 to a TopicsResponse
 *
 * @param value: u8, The source to convert
 *
 * @return TopicsResponse
 */
impl From<u8> for TopicResponse {
    fn from(value: u8) -> Self {
        match value {
            0x00 => TopicResponse::SubSuccess,
            0x0F => TopicResponse::SubFailure,
            0xF0 => TopicResponse::UnsubSuccess,
            0xFF => TopicResponse::UnsubFailure,
            _ => TopicResponse::Unknown
        }
    }
}