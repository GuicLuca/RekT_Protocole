#![allow(non_snake_case)]

use std::collections::HashSet;
use std::mem::size_of;
use std::sync::Arc;
use crate::datagrams::connect_requests::DtgConnect;
use crate::datagrams::data_request::DtgData;
use crate::datagrams::heartbeat_requests::{DtgHeartbeat, DtgHeartbeatRequest};
use crate::datagrams::latency_requests::{DtgPing, DtgPong};
use crate::datagrams::miscellaneous_requests::{DtgServerStatus, DtgServerStatusACK};
use crate::datagrams::object_requests::{DtgObjectRequest, DtgObjectRequestACK, DtgObjectRequestNACK};
use crate::datagrams::shutdown_request::DtgShutdown;
use crate::datagrams::topic_request::{DtgTopicRequest, DtgTopicRequestAck, DtgTopicRequestNack};
use crate::enums::datagram_type::DatagramType;
use crate::enums::end_connection_reason::EndConnexionReason;
use crate::enums::end_connection_reason::EndConnexionReason::{Shutdown, TimeOut};
use crate::enums::object_request_action::ObjectRequestAction;
use crate::enums::topic_action::TopicAction;
use crate::enums::topic_response::TopicResponse;
use crate::libs::types::{ClientId, ObjectId, PingId, Size, TopicId};
use crate::libs::utils::vec_to_u8;

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

// -------------------------------------------------------
//   Data datagrams
// -------------------------------------------------------
#[test]
fn test_DtgData_as_bytes() {
    let content = b"Message de test pour la methode as bytes";
    let sequenceNB: u32 = 42;
    let topicID = 444 as TopicId;

    let mut bytes: Vec<u8> = Vec::new();
    bytes.push(u8::from(DatagramType::Data));
    bytes.extend((content.len() as Size).to_le_bytes());
    bytes.extend(sequenceNB.to_le_bytes());
    bytes.extend(topicID.to_le_bytes());
    bytes.extend(content);

    let dtg = DtgData::new(sequenceNB, topicID, content.to_vec());
    assert_eq!(dtg.as_bytes(), bytes);
}

#[test]
fn test_DtgData_try_from() {
    let content = b"Message de testpour la methode try from";
    let sequenceNB: u32 = 654674698;
    let topicID = 44687687696844 as TopicId;
    let dtg = Arc::from(DtgData::new(sequenceNB, topicID, content.to_vec()));
    let dtg_ref = dtg.clone().as_bytes();
    let dtg_from = DtgData::try_from(&*dtg_ref);

    if dtg_from.is_ok() {
        assert_eq!(dtg_from.unwrap().as_bytes(), dtg.as_bytes());
    }else {
        assert!(false, "dtg_from is invalid");
    }
}

// -------------------------------------------------------
//   ObjectRequest datagrams
// -------------------------------------------------------
#[test]
fn test_DtgObjectRequest_as_bytes() {
    let ObjectAction = ObjectRequestAction::Create;
    let ObjectId = 641635874654 as ObjectId;
    let topics = HashSet::from([64658746584,6546654,4654654654,98986354,65465468]);

    let mut bytes: Vec<u8> = Vec::new();
    bytes.push(u8::from(DatagramType::ObjectRequest));
    bytes.extend(((topics.len() * size_of::<TopicId>())as Size).to_le_bytes());
    bytes.push(u8::from(ObjectAction));
    bytes.extend(ObjectId.to_le_bytes());
    bytes.extend(topics.iter().flat_map(|&x: &TopicId| {
        let bytes: [u8; 8] = x.to_le_bytes();
        bytes.into_iter()
    }).collect::<Vec<u8>>());

    let dtg = DtgObjectRequest::new(ObjectAction, ObjectId, topics);
    assert_eq!(dtg.as_bytes(), bytes);
}

#[test]
fn test_DtgObjectRequest_try_from() {
    let ObjectAction = ObjectRequestAction::Subscribe;
    let ObjectId = 68746541687496 as ObjectId;
    let topics = HashSet::from([1,2,3,4,5]);

    let dtg = Arc::from(DtgObjectRequest::new(ObjectAction, ObjectId, topics));
    let dtg_ref = dtg.clone().as_bytes();
    let ResultDtg_from = DtgObjectRequest::try_from(&*dtg_ref);

    if ResultDtg_from.is_ok() {
        let dtg_from = ResultDtg_from.unwrap();
        // check payload manually because hashset has not determined bytes order for same content.
        assert_eq!(dtg_from.payload, dtg.payload);
        assert_eq!(dtg_from.flag, dtg.flag);
        assert_eq!(dtg_from.datagram_type, dtg.datagram_type);
        assert_eq!(dtg_from.size, dtg.size);
    }else {
        assert!(false, "dtg_from is invalid");
    }
}

#[test]
fn test_DtgObjectRequestACK_as_bytes() {
    let flag = vec_to_u8(vec!(0,0,0,0,0,1,0,0)); // delete action
    let objectId = 941636875874654 as ObjectId;

    let mut bytes: Vec<u8> = Vec::new();
    bytes.push(u8::from(DatagramType::ObjectRequestAck));
    bytes.push(flag);
    bytes.extend(objectId.to_le_bytes());
    bytes.extend(0u64.to_le_bytes());

    let dtg = DtgObjectRequestACK::new(flag, objectId, 0);
    assert_eq!(dtg.as_bytes(), bytes);

    let flag = vec_to_u8(vec!(0,0,0,0,0,0,0,1)); // delete action
    let finalObjectId = 648687 as ObjectId;

    let mut bytes: Vec<u8> = Vec::new();
    bytes.push(u8::from(DatagramType::ObjectRequestAck));
    bytes.push(flag);
    bytes.extend(objectId.to_le_bytes());
    bytes.extend(finalObjectId.to_le_bytes());

    let dtg = DtgObjectRequestACK::new(flag, objectId, finalObjectId);
    assert_eq!(dtg.as_bytes(), bytes);
}

#[test]
fn test_DtgObjectRequestACK_try_from() {
    let flag = vec_to_u8(vec!(0,0,0,1,0,0,0,0)); // subscribe action
    let ObjectId = 98712341687496 as ObjectId;

    let dtg = Arc::from(DtgObjectRequestACK::new(flag, ObjectId, 0));
    let dtg_ref = dtg.clone().as_bytes();
    let ResultDtg_from = DtgObjectRequestACK::try_from(&*dtg_ref);

    if ResultDtg_from.is_ok() {
        assert_eq!(ResultDtg_from.unwrap().as_bytes(), dtg.as_bytes());
    }else {
        assert!(false, "dtg_from is invalid");
    }

    let flag = vec_to_u8(vec!(0,0,0,0,0,0,0,1)); // create action
    let finalObjectId = 3468746 as ObjectId;

    let dtg = Arc::from(DtgObjectRequestACK::new(flag, ObjectId, finalObjectId));
    let dtg_ref = dtg.clone().as_bytes();
    let ResultDtg_from = DtgObjectRequestACK::try_from(&*dtg_ref);

    if ResultDtg_from.is_ok() {
        assert_eq!(ResultDtg_from.unwrap().as_bytes(), dtg.as_bytes());
    }else {
        assert!(false, "dtg_from is invalid");
    }
}

#[test]
fn test_DtgObjectRequestNACK_as_bytes() {
    let flag = vec_to_u8(vec!(0,0,0,0,0,0,1,0)); // modify action
    let objectId = 4654 as ObjectId;
    let reason = "Fail to modify the object because the object id is invalid.";

    let mut bytes: Vec<u8> = Vec::new();
    bytes.push(u8::from(DatagramType::ObjectRequestNack));
    bytes.extend((reason.len() as Size).to_le_bytes());
    bytes.push(flag);
    bytes.extend(objectId.to_le_bytes());
    bytes.extend(reason.as_bytes());

    let dtg = DtgObjectRequestNACK::new(flag, objectId, reason);
    assert_eq!(dtg.as_bytes(), bytes);
}

#[test]
fn test_DtgObjectRequestNACK_try_from() {
    let flag = vec_to_u8(vec!(0,0,0,0,0,0,1,0)); // modify action
    let ObjectId = 98712341687496 as ObjectId;
    let reason = "Fail to modify the object because the object id is invalid.";

    let dtg = Arc::from(DtgObjectRequestNACK::new(flag, ObjectId, reason));
    let dtg_ref = dtg.clone().as_bytes();
    let ResultDtg_from = DtgObjectRequestNACK::try_from(&*dtg_ref);

    if ResultDtg_from.is_ok() {
        assert_eq!(ResultDtg_from.unwrap().as_bytes(), dtg.as_bytes());
    }else {
        assert!(false, "dtg_from is invalid");
    }
}

// -------------------------------------------------------
//   TopicRequest datagrams
// -------------------------------------------------------
#[test]
fn test_DtgTopicRequest_as_bytes() {
    let TopicAction = TopicAction::Subscribe;
    let TopicsId = 641635874654 as TopicId;

    let mut bytes: Vec<u8> = Vec::new();
    bytes.push(u8::from(DatagramType::TopicRequest));
    bytes.push(u8::from(TopicAction));
    bytes.extend(TopicsId.to_le_bytes());

    let dtg = DtgTopicRequest::new(TopicAction, TopicsId);
    assert_eq!(dtg.as_bytes(), bytes);
}

#[test]
fn test_DtgOTopicRequest_try_from() {
    let TopicAction = TopicAction::Subscribe;
    let TopicsId = 641635874654 as TopicId;

    let dtg = Arc::from(DtgTopicRequest::new(TopicAction, TopicsId));
    let dtg_ref = dtg.clone().as_bytes();
    let ResultDtg_from = DtgTopicRequest::try_from(&*dtg_ref);

    if ResultDtg_from.is_ok() {
        assert_eq!(ResultDtg_from.unwrap().as_bytes(), dtg.as_bytes());
    }else {
        assert!(false, "dtg_from is invalid");
    }
}

#[test]
fn test_DtgTopicRequestACK_as_bytes() {
    let flag = TopicResponse::SubFailure;
    let TopicId = 941636875874654 as TopicId;

    let mut bytes: Vec<u8> = Vec::new();
    bytes.push(u8::from(DatagramType::TopicRequestAck));
    bytes.push(u8::from(flag));
    bytes.extend(TopicId.to_le_bytes());

    let dtg = DtgTopicRequestAck::new(TopicId, flag);
    assert_eq!(dtg.as_bytes(), bytes);
}

#[test]
fn test_DtgTopicRequestACK_try_from() {
    let flag = TopicResponse::UnsubSuccess;
    let TopicId = 321789456398724 as TopicId;

    let dtg = Arc::from(DtgTopicRequestAck::new(TopicId, flag));
    let dtg_ref = dtg.clone().as_bytes();
    let ResultDtg_from = DtgTopicRequestAck::try_from(&*dtg_ref);

    if ResultDtg_from.is_ok() {
        assert_eq!(ResultDtg_from.unwrap().as_bytes(), dtg.as_bytes());
    }else {
        assert!(false, "dtg_from is invalid");
    }
}

#[test]
fn test_DtgTopicRequestNACK_as_bytes() {
    let flag = TopicResponse::UnsubSuccess;
    let reason = "Fail to unsubscribe the topic XXX because the id is invalid.";

    let mut bytes: Vec<u8> = Vec::new();
    bytes.push(u8::from(DatagramType::TopicRequestNack));
    bytes.extend((reason.len() as Size).to_le_bytes());
    bytes.push(u8::from(flag));
    bytes.extend(reason.as_bytes());

    let dtg = DtgTopicRequestNack::new(flag, reason);
    assert_eq!(dtg.as_bytes(), bytes);
}

#[test]
fn test_DtgTopicRequestNACK_try_from() {
    let flag = TopicResponse::UnsubSuccess;
    let reason = "Fail to unsubscribe the topic XXX because the id is invalid.";

    let dtg = Arc::from(DtgTopicRequestNack::new(flag, reason));
    let dtg_ref = dtg.clone().as_bytes();
    let ResultDtg_from = DtgTopicRequestNack::try_from(&*dtg_ref);

    if ResultDtg_from.is_ok() {
        assert_eq!(ResultDtg_from.unwrap().as_bytes(), dtg.as_bytes());
    }else {
        assert!(false, "dtg_from is invalid");
    }
}