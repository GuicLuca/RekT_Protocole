/**
 * # Common used types
 */

/// This is the type used to represent the size of a message.
pub type Size = u16;
/// This is the type used to represent the flag of a message.
pub type Flag = u8;
/// This is the type used to normalize the size of the enum used as flag
pub type TopicId = u64;
/// This is the type used to represent a PingID.
pub type PingId = u8;
/// This is the type used to represent an ObjectId.
pub type ObjectId = u64; // 0..2 for type identifier (User generated, broker, temporary)  2..64 identifier
/// This is the type used to represent a ClientId.
pub type ClientId = u64;
