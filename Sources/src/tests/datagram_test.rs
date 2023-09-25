#![allow(non_snake_case)]
use std::sync::Arc;
use crate::datagrams::connect_requests::DtgConnect;
use crate::datagrams::heartbeat_requests::{DtgHeartbeat, DtgHeartbeatRequest};
use crate::datagrams::latency_requests::{DtgPing, DtgPong};
use crate::datagrams::miscellaneous_requests::{DtgServerStatus, DtgServerStatusACK};
use crate::datagrams::shutdown_request::DtgShutdown;
use crate::enums::datagram_type::DatagramType;
use crate::enums::end_connection_reason::EndConnexionReason;
use crate::enums::end_connection_reason::EndConnexionReason::{Shutdown, TimeOut};
use crate::libs::types::{ClientId, PingId};

// -------------------------------------------------------
//   Connect
// -------------------------------------------------------
#[test]
fn test_DtgConnect_as_bytes() {
    let bytes: Vec<u8> = vec!(u8::from(DatagramType::Connect));
    let dtg = DtgConnect::new();
    assert_eq!(dtg.as_bytes(), bytes);
}

#[test]
fn test_DtgConnect_try_from() {
    let dtg = Arc::from(DtgConnect::new());
    let dtg_ref = dtg.clone().as_bytes();
    let dtg_from = DtgConnect::try_from(&*dtg_ref);

    if dtg_from.is_ok() {
        assert_eq!(dtg_from.unwrap().as_bytes(), dtg.as_bytes());
    }else {
        assert!(false, "dtg_from is invalid");
    }
}

// -------------------------------------------------------
//   Shutdown
// -------------------------------------------------------
#[test]
fn test_DtgShutdown_as_bytes() {
    let bytes: Vec<u8> = vec!(u8::from(DatagramType::Shutdown), u8::from(Shutdown));
    let dtg = DtgShutdown::new(Shutdown);
    assert_eq!(dtg.as_bytes(), bytes);
    let bytes: Vec<u8> = vec!(u8::from(DatagramType::Shutdown), u8::from(TimeOut));
    let dtg = DtgShutdown::new(TimeOut);
    assert_eq!(dtg.as_bytes(), bytes);
}

#[test]
fn test_DtgShutdown_try_from() {
    let dtg = Arc::from(DtgShutdown::new(Shutdown));
    let dtg_ref = dtg.clone().as_bytes();
    let dtg_from = DtgShutdown::try_from(&*dtg_ref);

    if dtg_from.is_ok() {
        assert_eq!(dtg_from.unwrap().as_bytes(), dtg.as_bytes());
    }else {
        assert!(false, "dtg_from is invalid");
    }
}

// -------------------------------------------------------
//   Heartbeat
// -------------------------------------------------------
#[test]
fn test_DtgHeartbeat_as_bytes() {
    let bytes: Vec<u8> = vec!(u8::from(DatagramType::Heartbeat));
    let dtg = DtgHeartbeat::new();
    assert_eq!(dtg.as_bytes(), bytes);
}

#[test]
fn test_DtgHeartbeat_try_from() {
    let dtg = Arc::from(DtgHeartbeat::new());
    let dtg_ref = dtg.clone().as_bytes();
    let dtg_from = DtgHeartbeat::try_from(&*dtg_ref);

    if dtg_from.is_ok() {
        assert_eq!(dtg_from.unwrap().as_bytes(), dtg.as_bytes());
    }else {
        assert!(false, "dtg_from is invalid");
    }
}

#[test]
fn test_DtgHeartbeatRequest_as_bytes() {
    let bytes: Vec<u8> = vec!(u8::from(DatagramType::HeartbeatRequest));
    let dtg = DtgHeartbeatRequest::new();
    assert_eq!(dtg.as_bytes(), bytes);
}

#[test]
fn test_DtgHeartbeatRequest_try_from() {
    let dtg = Arc::from(DtgHeartbeatRequest::new());
    let dtg_ref = dtg.clone().as_bytes();
    let dtg_from = DtgHeartbeatRequest::try_from(&*dtg_ref);

    if dtg_from.is_ok() {
        assert_eq!(dtg_from.unwrap().as_bytes(), dtg.as_bytes());
    }else {
        assert!(false, "dtg_from is invalid");
    }
}

// -------------------------------------------------------
//   Latency measurement
// -------------------------------------------------------
#[test]
fn test_DtgPing_as_bytes() {
    let bytes: Vec<u8> = vec!(u8::from(DatagramType::Ping), 127u8);
    let dtg = DtgPing::new(127);
    assert_eq!(dtg.as_bytes(), bytes);
}

#[test]
fn test_DtgPing_try_from() {
    let dtg = Arc::from(DtgPing::new(127));
    let dtg_ref = dtg.clone().as_bytes();
    let dtg_from = DtgPing::try_from(&*dtg_ref);

    if dtg_from.is_ok() {
        assert_eq!(dtg_from.unwrap().as_bytes(), dtg.as_bytes());
    }else {
        assert!(false, "dtg_from is invalid");
    }
}

#[test]
fn test_DtgPong_as_bytes() {
    let bytes: Vec<u8> = vec!(u8::from(DatagramType::Pong), 127u8);
    let dtg = DtgPong::new(127);
    assert_eq!(dtg.as_bytes(), bytes);
}

#[test]
fn test_DtgPong_try_from() {
    let dtg = Arc::from(DtgPong::new(127));
    let dtg_ref = dtg.clone().as_bytes();
    let dtg_from = DtgPong::try_from(&*dtg_ref);

    if dtg_from.is_ok() {
        assert_eq!(dtg_from.unwrap().as_bytes(), dtg.as_bytes());
    }else {
        assert!(false, "dtg_from is invalid");
    }
}

// -------------------------------------------------------
//   Misc datagrams
// -------------------------------------------------------
#[test]
fn test_DtgServerStatus_as_bytes() {
    let bytes: Vec<u8> = vec!(u8::from(DatagramType::ServerStatus));
    let dtg = DtgServerStatus::new();
    assert_eq!(dtg.as_bytes(), bytes);
}

#[test]
fn test_DtgServerStatus_try_from() {
    let dtg = Arc::from(DtgServerStatus::new());
    let dtg_ref = dtg.clone().as_bytes();
    let dtg_from = DtgServerStatus::try_from(&*dtg_ref);

    if dtg_from.is_ok() {
        assert_eq!(dtg_from.unwrap().as_bytes(), dtg.as_bytes());
    }else {
        assert!(false, "dtg_from is invalid");
    }
}

#[test]
fn test_DtgServerStatusAck_as_bytes() {
    let mut bytes: Vec<u8> = vec!(u8::from(DatagramType::ServerStatusAck));
    bytes.extend(ClientId::MAX.to_le_bytes());
    let dtg = DtgServerStatusACK::new(ClientId::MAX);
    assert_eq!(dtg.as_bytes(), bytes);
}

#[test]
fn test_DtgServerStatusAck_try_from() {
    let dtg = Arc::from(DtgServerStatusACK::new(ClientId::MAX));
    let dtg_ref = dtg.clone().as_bytes();
    let dtg_from = DtgServerStatusACK::try_from(&*dtg_ref);

    if dtg_from.is_ok() {
        assert_eq!(dtg_from.unwrap().as_bytes(), dtg.as_bytes());
    }else {

        assert!(false, "dtg_from is invalid : {}",  dtg_from.err().unwrap());
    }
}