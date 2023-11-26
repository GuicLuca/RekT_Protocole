/* RektProtocol common lib - Lucas Guichard <lucasguichard127@gmail.com> - 2023 */

#pragma once

/* Warning, this file is autogenerated by cbindgen. Don't modify this manually. */

#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>


///  * DatagramType are used to translate request type  * to the corresponding hexadecimal code.
enum class DatagramType : uint8_t {
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
};

///  * End connexion reasons are used to  * detail the reason of the shutdown request.
enum class EndConnexionReason : uint8_t {
    Shutdown,
    TimeOut,
    Unknown,
};

///  * ObjectRequestAction are all possible action in OBJECT_REQUEST datagram.
enum class ObjectRequestAction : uint8_t {
    Create,
    Update,
    Delete,
    Subscribe,
    Unsubscribe,
    Unknown,
};

///  * Topics action are all actions that  * a peer can do in a TOPICS_REQUEST
enum class TopicAction : uint8_t {
    Subscribe,
    Unsubscribe,
    Unknown,
};

///  * Topics response are all possible responses  * type to a TOPICS_REQUEST
enum class TopicResponse : uint8_t {
    SubSuccess,
    SubFailure,
    UnsubSuccess,
    UnsubFailure,
    Unknown,
};

struct VecU8 {
    uint8_t *data;
    size_t length;
    size_t capacity;

    VecU8(uint8_t *const& data,
          size_t const& length,
          size_t const& capacity)
      : data(data),
        length(length),
        capacity(capacity)
    {}

};

using ClientId = uint64_t;

struct DtgConnectAck {
    DatagramType datagram_type;
    ClientId peer_id;
    uint16_t heartbeat_period;

    DtgConnectAck(DatagramType const& datagram_type,
                  ClientId const& peer_id,
                  uint16_t const& heartbeat_period)
      : datagram_type(datagram_type),
        peer_id(peer_id),
        heartbeat_period(heartbeat_period)
    {}

};

struct ByteSlice {
    const uint8_t *buffer;
    size_t len;

    ByteSlice(const uint8_t *const& buffer,
              size_t const& len)
      : buffer(buffer),
        len(len)
    {}

};

struct DtgConnect {
    DatagramType datagram_type;

    DtgConnect(DatagramType const& datagram_type)
      : datagram_type(datagram_type)
    {}

};

using Size = uint16_t;

struct CDtgConnectNack {
    DatagramType datagram_type;
    Size size;
    VecU8 payload;

    CDtgConnectNack(DatagramType const& datagram_type,
                    Size const& size,
                    VecU8 const& payload)
      : datagram_type(datagram_type),
        size(size),
        payload(payload)
    {}

};

using TopicId = uint64_t;

struct CDtgData {
    DatagramType datagram_type;
    Size size;
    uint32_t sequence_number;
    TopicId topic_id;
    VecU8 payload;

    CDtgData(DatagramType const& datagram_type,
             Size const& size,
             uint32_t const& sequence_number,
             TopicId const& topic_id,
             VecU8 const& payload)
      : datagram_type(datagram_type),
        size(size),
        sequence_number(sequence_number),
        topic_id(topic_id),
        payload(payload)
    {}

};

struct DtgHeartbeat {
    DatagramType datagram_type;

    DtgHeartbeat(DatagramType const& datagram_type)
      : datagram_type(datagram_type)
    {}

};

struct DtgHeartbeatRequest {
    DatagramType datagram_type;

    DtgHeartbeatRequest(DatagramType const& datagram_type)
      : datagram_type(datagram_type)
    {}

};

using Flag = uint8_t;

using ObjectId = uint64_t;

struct CDtgObjectRequestACK {
    DatagramType datagram_type;
    Flag flag;
    ObjectId object_id;
    const unsigned long long *final_object_id;

    CDtgObjectRequestACK(DatagramType const& datagram_type,
                         Flag const& flag,
                         ObjectId const& object_id,
                         const unsigned long long *const& final_object_id)
      : datagram_type(datagram_type),
        flag(flag),
        object_id(object_id),
        final_object_id(final_object_id)
    {}

};

struct HashSetWrapperU64 {
    const unsigned long long *data;
    size_t len;

    HashSetWrapperU64(const unsigned long long *const& data,
                      size_t const& len)
      : data(data),
        len(len)
    {}

};

struct CDtgObjectRequest {
    DatagramType datagram_type;
    ObjectRequestAction flag;
    Size size;
    ObjectId object_id;
    HashSetWrapperU64 payload;

    CDtgObjectRequest(DatagramType const& datagram_type,
                      ObjectRequestAction const& flag,
                      Size const& size,
                      ObjectId const& object_id,
                      HashSetWrapperU64 const& payload)
      : datagram_type(datagram_type),
        flag(flag),
        size(size),
        object_id(object_id),
        payload(payload)
    {}

};

using PingId = uint8_t;

struct DtgPing {
    DatagramType datagram_type;
    PingId ping_id;

    DtgPing(DatagramType const& datagram_type,
            PingId const& ping_id)
      : datagram_type(datagram_type),
        ping_id(ping_id)
    {}

};

struct DtgPong {
    DatagramType datagram_type;
    PingId ping_id;

    DtgPong(DatagramType const& datagram_type,
            PingId const& ping_id)
      : datagram_type(datagram_type),
        ping_id(ping_id)
    {}

};

struct DtgServerStatusACK {
    DatagramType datagram_type;
    ClientId connected_client;

    DtgServerStatusACK(DatagramType const& datagram_type,
                       ClientId const& connected_client)
      : datagram_type(datagram_type),
        connected_client(connected_client)
    {}

};

struct DtgServerStatus {
    DatagramType datagram_type;

    DtgServerStatus(DatagramType const& datagram_type)
      : datagram_type(datagram_type)
    {}

};

struct DtgShutdown {
    DatagramType datagram_type;
    EndConnexionReason reason;

    DtgShutdown(DatagramType const& datagram_type,
                EndConnexionReason const& reason)
      : datagram_type(datagram_type),
        reason(reason)
    {}

};


extern "C" {

DatagramType DatagramTypeFromCode(uint8_t code);

uint8_t DatagramTypeToCode(DatagramType enum_val);

const char *DisplayDatagramType(DatagramType enum_val);

VecU8 DtgConnectAckAsBytes(DtgConnectAck datagram);

DtgConnectAck DtgConnectAckNew(ClientId peer_id, uint16_t heartbeat_period);

const DtgConnectAck *DtgConnectAckTryFromBuffer(ByteSlice buffer);

VecU8 DtgConnectAsBytes(DtgConnect datagram);

VecU8 DtgConnectNackAsBytes(CDtgConnectNack datagram);

CDtgConnectNack DtgConnectNackNew(const char *msg);

const CDtgConnectNack *DtgConnectNackTryFromBuffer(ByteSlice buffer);

DtgConnect DtgConnectNew();

const DtgConnect *DtgConnectTryFromBuffer(ByteSlice buffer);

VecU8 DtgDataAsBytes(CDtgData datagram);

CDtgData DtgDataNew(uint32_t sequence_number, TopicId topic_id, VecU8 payload);

const CDtgData *DtgDataTryFromBuffer(ByteSlice buffer);

VecU8 DtgHeartbeatAsBytes(DtgHeartbeat datagram);

DtgHeartbeat DtgHeartbeatNew();

VecU8 DtgHeartbeatRequestAsBytes(DtgHeartbeatRequest datagram);

DtgHeartbeatRequest DtgHeartbeatRequestNew();

const DtgHeartbeatRequest *DtgHeartbeatRequestTryFromBuffer(ByteSlice buffer);

const DtgHeartbeat *DtgHeartbeatTryFromBuffer(ByteSlice buffer);

VecU8 DtgObjectRequestACKAsBytes(CDtgObjectRequestACK datagram);

CDtgObjectRequestACK DtgObjectRequestACKNew(Flag flag,
                                            ObjectId object_id,
                                            const unsigned long long *final_object_id);

const CDtgObjectRequestACK *DtgObjectRequestACKTryFromBuffer(ByteSlice buffer);

VecU8 DtgObjectRequestAsBytes(CDtgObjectRequest datagram);

CDtgObjectRequest DtgObjectRequestNew(ObjectRequestAction flag,
                                      ObjectId object_id,
                                      HashSetWrapperU64 topics);

const CDtgObjectRequest *DtgObjectRequestTryFromBuffer(ByteSlice buffer);

VecU8 DtgPingAsBytes(DtgPing datagram);

DtgPing DtgPingNew(PingId ping_id);

const DtgPing *DtgPingTryFromBuffer(ByteSlice buffer);

VecU8 DtgPongAsBytes(DtgPong datagram);

DtgPong DtgPongNew(PingId pong_id);

const DtgPong *DtgPongTryFromBuffer(ByteSlice buffer);

VecU8 DtgServerStatusACKAsBytes(DtgServerStatusACK datagram);

DtgServerStatusACK DtgServerStatusACKNew(ClientId nb_client);

const DtgServerStatusACK *DtgServerStatusACKTryFromBuffer(ByteSlice buffer);

VecU8 DtgServerStatusAsBytes(DtgServerStatus datagram);

DtgServerStatus DtgServerStatusNew();

const DtgServerStatus *DtgServerStatusTryFromBuffer(ByteSlice buffer);

VecU8 DtgShutdownAsBytes(DtgShutdown datagram);

DtgShutdown DtgShutdownNew(EndConnexionReason reason);

const DtgShutdown *DtgShutdownTryFromBuffer(ByteSlice buffer);

EndConnexionReason EndConnexionReasonFromCode(uint8_t code);

uint8_t EndConnexionReasonToCode(EndConnexionReason enum_val);

VecU8 GetBytesFromSlice(ByteSlice buffer, size_t from, size_t to);

uint16_t GetU16AtPosition(ByteSlice buffer, size_t position);

uint32_t GetU32AtPosition(ByteSlice buffer, size_t position);

uint64_t GetU64AtPosition(ByteSlice buffer, size_t position);

ObjectRequestAction ObjectRequestActionFromCode(uint8_t code);

uint8_t ObjectRequestActionToCode(ObjectRequestAction enum_val);

TopicAction TopicActionFromCode(uint8_t code);

uint8_t TopicActionToCode(TopicAction enum_val);

TopicResponse TopicResponseFromCode(uint8_t code);

uint8_t TopicResponseToCode(TopicResponse enum_val);

void vec_u8_free(VecU8 v);

void vec_u8_push_bytes(VecU8 *v, ByteSlice bytes);

void vec_u8_reserve(VecU8 *v, size_t len);

} // extern "C"