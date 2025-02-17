#![allow(unused)]

use std::ffi::c_char;

/**
 * ConnectionStatus is the enum used to statute the connection of a client.
 * Depending on the status, the client can do different actions or the server will do different actions.
 */
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
#[no_mangle]
pub enum ConnectionStatus {
    Connecting, // The client is connecting to the server ( RekT Handshake )
    Connected, // The client is connected to the server
    Spurious, // The client is connected, but he is late on the heartbeat
    
    Unknown, // Not a valid status, used for error handling
}

/**
 * This function return the string name of the ConnectionStatus given.
 *
 * @param status: ConnectionStatus, the source to translate into string.
 *
 * @return string, the corresponding name
 */
pub fn display_connection_status<'a>(status: ConnectionStatus) -> &'a str {
    match status {
        ConnectionStatus::Connecting => "Connecting",
        ConnectionStatus::Connected => "Connected",
        ConnectionStatus::Spurious => "Spurious",
        ConnectionStatus::Unknown => "Unknown",
    }
}

/**
 * This function convert an u8 to a ConnectionStatus
 *
 * @param value: u8, The source to convert
 *
 * @return ConnectionStatus
 */

impl From<u8> for ConnectionStatus {
    fn from(value: u8) -> Self {
        match value {
            0x00 => ConnectionStatus::Connecting,
            0x01 => ConnectionStatus::Connected,
            0xF1 => ConnectionStatus::Spurious,
            _ => ConnectionStatus::Unknown,
        }
    }
}

/**
 * This function convert a ConnectionStatus to an u8
 *
 * @param value: ConnectionStatus, The source to convert
 *
 * @return u8
 */
impl From<ConnectionStatus> for u8 {
    fn from(value: ConnectionStatus) -> Self {
        match value {
            ConnectionStatus::Connecting => 0x00,
            ConnectionStatus::Connected => 0x01,
            ConnectionStatus::Spurious => 0xF1,
            ConnectionStatus::Unknown => 0xFF,
        }
    }
}
