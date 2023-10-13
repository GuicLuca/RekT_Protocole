#![allow(unused)]

use std::ffi::c_char;

/**
 * DatagramType are used to translate request type
 * to the corresponding hexadecimal code.
 */
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
#[no_mangle]
pub enum DatagramType {
    Connect,
    ConnectAck,
    ConnectNack,
    Shutdown,
    OpenStream,
    ServerStatus,
    ServerStatusAck,
    Heartbeat,
    HeartbeatRequest,
    Ping,
    Pong,
    TopicRequest,
    TopicRequestAck,
    TopicRequestNack,
    ObjectRequest,
    ObjectRequestAck,
    ObjectRequestNack,
    Data,
    Unknown,
}

/**
 * This function return the string name of the DatagramType given.
 *
 * @param datagram: DatagramType, the source to translate into string.
 *
 * @return string, the corresponding name
 */
#[no_mangle]
pub extern "C" fn display_datagram_type<'a>(datagram: DatagramType) -> *const c_char{
    let result = match datagram {
        DatagramType::Connect => "Connect",
        DatagramType::ConnectAck => "Connect_ACK",
        DatagramType::ConnectNack => "Connect_NACK",
        DatagramType::Shutdown => "Shutdown",
        DatagramType::OpenStream => "Open_Stream",
        DatagramType::ServerStatus => "Server_Status",
        DatagramType::ServerStatusAck => "Server_Status_Ack",
        DatagramType::Heartbeat => "HeartBeat",
        DatagramType::HeartbeatRequest => "HeartBeat_Request",
        DatagramType::Ping => "Ping",
        DatagramType::Pong => "Pong",
        DatagramType::TopicRequest => "Topic_Request",
        DatagramType::TopicRequestAck => "Topic_Request_Ack",
        DatagramType::TopicRequestNack => "Topic_Request_Nack",
        DatagramType::ObjectRequest => "Object_Request",
        DatagramType::ObjectRequestAck => "Object_Request_Ack",
        DatagramType::ObjectRequestNack => "Object_Request_Nack",
        DatagramType::Data => "Data",
        DatagramType::Unknown => "Unknown",
    };

    std::ffi::CString::new(result).unwrap().into_raw()
}

/**
 * This function convert a u8 to a DatagramType
 *
 * @param value: u8, The source to convert
 *
 * @return DatagramType
 */

impl From<u8> for DatagramType {
    fn from(value: u8) -> Self {
        match value {
            0xF0 => DatagramType::Connect,
            0xF1 => DatagramType::ConnectAck,
            0xF2 => DatagramType::ConnectNack,
            0xFF => DatagramType::Shutdown,
            0xFA => DatagramType::OpenStream,
            0x30 => DatagramType::ServerStatus,
            0x00 => DatagramType::ServerStatusAck,
            0x60 => DatagramType::Heartbeat,
            0x61 => DatagramType::HeartbeatRequest,
            0x62 => DatagramType::Ping,
            0x72 => DatagramType::Pong,
            0x45 => DatagramType::TopicRequest,
            0x05 => DatagramType::TopicRequestAck,
            0x15 => DatagramType::TopicRequestNack,
            0x48 => DatagramType::ObjectRequest,
            0x08 => DatagramType::ObjectRequestAck,
            0x18 => DatagramType::ObjectRequestNack,
            0x42 => DatagramType::Data,
            _ => DatagramType::Unknown
        }
    }
}

/**
 * This function convert a DatagramType to an u8
 *
 * @param value: DatagramType, The source to convert
 *
 * @return u8
 */
impl From<DatagramType> for u8 {
    fn from(value: DatagramType) -> Self {
        match value {
            DatagramType::Connect => 0xF0,
            DatagramType::ConnectAck => 0xF1,
            DatagramType::ConnectNack => 0xF2,
            DatagramType::OpenStream => 0xFA,
            DatagramType::Shutdown => 0xFF,
            DatagramType::ServerStatus => 0x30,
            DatagramType::ServerStatusAck => 0x00,
            DatagramType::Heartbeat => 0x60,
            DatagramType::HeartbeatRequest => 0x61,
            DatagramType::Ping => 0x62,
            DatagramType::Pong => 0x72,
            DatagramType::TopicRequest => 0x45,
            DatagramType::TopicRequestAck => 0x05,
            DatagramType::TopicRequestNack => 0x15,
            DatagramType::ObjectRequest => 0x48,
            DatagramType::ObjectRequestAck => 0x08,
            DatagramType::ObjectRequestNack => 0x18,
            DatagramType::Data => 0x42,
            DatagramType::Unknown => 0xAA,
        }
    }
}