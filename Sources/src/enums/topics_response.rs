/**
 * Topics response are all possible responses
 * type to a TOPICS_REQUEST
 */
#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum TopicsResponse {
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
impl From<TopicsResponse> for u8 {
    fn from(value: TopicsResponse) -> Self {
        match value {
            TopicsResponse::SubSuccess => 0x00,
            TopicsResponse::SubFailure => 0x0F,
            TopicsResponse::UnsubSuccess => 0xF0,
            TopicsResponse::UnsubFailure => 0xFF,
            TopicsResponse::Unknown => 0xAA,
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
impl From<u8> for TopicsResponse {
    fn from(value: u8) -> Self {
        match value {
            0x00 => TopicsResponse::SubSuccess,
            0x0F => TopicsResponse::SubFailure,
            0xF0 => TopicsResponse::UnsubSuccess,
            0xFF => TopicsResponse::UnsubFailure,
            _ => TopicsResponse::Unknown
        }
    }
}