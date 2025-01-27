use std::collections::HashSet;
use std::ffi::{c_char, c_ulonglong, CStr, CString};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::null;
use std::str::{from_utf8, Utf8Error};
use std::{mem, ptr, slice};

use crate::datagrams::connect_requests::{DtgConnect, DtgConnectAck, DtgConnectNack};
use crate::datagrams::data_request::DtgData;
use crate::datagrams::heartbeat_requests::{DtgHeartbeat, DtgHeartbeatRequest};
use crate::datagrams::latency_requests::{DtgPing, DtgPong};
use crate::datagrams::miscellaneous_requests::{DtgServerStatus, DtgServerStatusACK};
use crate::datagrams::object_requests::{
    DtgObjectRequest, DtgObjectRequestACK, DtgObjectRequestNACK,
};
use crate::datagrams::shutdown_request::DtgShutdown;
use crate::enums::datagram_type::{display_datagram_type, DatagramType};
use crate::enums::end_connection_reason::EndConnexionReason;
use crate::enums::object_request_action::ObjectRequestAction;
use crate::enums::topic_action::TopicAction;
use crate::enums::topic_response::TopicResponse;
use crate::libs::types::{ClientId, Flag, ObjectId, PingId, Size, TopicId};
use crate::libs::utils::{get_bytes_from_slice, get_u16_at_pos, get_u32_at_pos, get_u64_at_pos};

// Command to generate bindings : cbindgen --config cbindgen.toml --crate rekt-common --output bindings.h
// ------------------------------------------------------------
// FFI types
// ------------------------------------------------------------

#[repr(C)]
pub struct VecU8 {
    data: *mut u8,
    length: usize,
    capacity: usize,
}

impl VecU8 {
    fn into_vec(self) -> Vec<u8> {
        unsafe { Vec::from_raw_parts(self.data, self.length, self.capacity) }
    }

    // Equivalent to `into_vec` but clears self instead of consuming the value.
    fn flush_into_vec(&mut self) -> Vec<u8> {
        self.convert_into_vec::<u8>()
    }

    // Like flush_into_vec, but also does an unsafe conversion to the desired type.
    fn convert_into_vec<T>(&mut self) -> Vec<T> {
        let vec = unsafe {
            Vec::from_raw_parts(
                self.data as *mut T,
                self.length / mem::size_of::<T>(),
                self.capacity / mem::size_of::<T>(),
            )
        };
        self.data = ptr::null_mut();
        self.length = 0;
        self.capacity = 0;
        vec
    }

    fn from_vec(mut v: Vec<u8>) -> VecU8 {
        let w = VecU8 {
            data: v.as_mut_ptr(),
            length: v.len(),
            capacity: v.capacity(),
        };
        mem::forget(v);
        w
    }

    fn reserve(&mut self, len: usize) {
        let mut vec = self.flush_into_vec();
        vec.reserve(len);
        *self = Self::from_vec(vec);
    }

    fn push_bytes(&mut self, bytes: &[u8]) {
        let mut vec = self.flush_into_vec();
        vec.extend_from_slice(bytes);
        *self = Self::from_vec(vec);
    }
}

#[no_mangle]
pub extern "C" fn vec_u8_push_bytes(v: &mut VecU8, bytes: ByteSlice) {
    v.push_bytes(bytes.as_slice());
}

#[no_mangle]
pub extern "C" fn vec_u8_reserve(v: &mut VecU8, len: usize) {
    v.reserve(len);
}

#[no_mangle]
pub extern "C" fn vec_u8_free(v: VecU8) {
    v.into_vec();
}

unsafe fn make_slice<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    if ptr.is_null() {
        &[]
    } else {
        slice::from_raw_parts(ptr, len)
    }
}

unsafe fn make_slice_mut<'a, T>(ptr: *mut T, len: usize) -> &'a mut [T] {
    if ptr.is_null() {
        &mut []
    } else {
        slice::from_raw_parts_mut(ptr, len)
    }
}

#[repr(C)]
pub struct ByteSlice<'a> {
    buffer: *const u8,
    len: usize,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> ByteSlice<'a> {
    pub fn new(slice: &'a [u8]) -> ByteSlice<'a> {
        ByteSlice {
            buffer: slice.as_ptr(),
            len: slice.len(),
            _phantom: PhantomData,
        }
    }

    pub fn as_slice(&self) -> &'a [u8] {
        unsafe { make_slice(self.buffer, self.len) }
    }
}

#[repr(C)]
pub struct MutByteSlice<'a> {
    buffer: *mut u8,
    len: usize,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> MutByteSlice<'a> {
    pub fn new(slice: &'a mut [u8]) -> MutByteSlice<'a> {
        let len = slice.len();
        MutByteSlice {
            buffer: slice.as_mut_ptr(),
            len,
            _phantom: PhantomData,
        }
    }

    pub fn as_mut_slice(&mut self) -> &'a mut [u8] {
        unsafe { make_slice_mut(self.buffer, self.len) }
    }
}

// Define a struct to hold the array and its length
#[repr(C)]
pub struct HashSetWrapperU64 {
    pub data: *const c_ulonglong,
    pub len: usize,
}

// Function to convert a HashSet to a HashSetWrapper
pub fn convert_hashset_to_wrapper(hashset: &HashSet<u64>) -> HashSetWrapperU64 {
    let data: Vec<c_ulonglong> = hashset.iter().copied().collect();
    let len = data.len();
    let boxed_data = data.into_boxed_slice();
    let data_ptr = Box::into_raw(boxed_data) as *const c_ulonglong;
    HashSetWrapperU64 {
        data: data_ptr,
        len,
    }
}

pub fn free_hashset_wrapper(wrapper: HashSetWrapperU64) {
    unsafe {
        if !wrapper.data.is_null() {
            Box::from_raw(std::slice::from_raw_parts_mut(
                wrapper.data as *mut c_ulonglong,
                wrapper.len,
            ));
        }
    }
}
pub fn convert_wrapper_to_hashset(wrapper: HashSetWrapperU64) -> HashSet<u64> {
    let slice = unsafe { slice::from_raw_parts(wrapper.data, wrapper.len) };
    let hashset: HashSet<u64> = slice.iter().copied().collect();
    free_hashset_wrapper(wrapper);
    hashset
}

pub fn convert_ptr_to_option(ptr: *const c_ulonglong) -> Option<u64> {
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { *ptr })
    }
}

// ------------------------------------------------------------
// Enums - DatagramTypes
// ------------------------------------------------------------
#[no_mangle]
pub extern "C" fn DisplayDatagramType(enum_val: DatagramType) -> *const c_char {
    CString::new(display_datagram_type(enum_val))
        .unwrap()
        .into_raw()
}

#[no_mangle]
pub extern "C" fn DatagramTypeFromCode(code: u8) -> DatagramType {
    DatagramType::from(code)
}

#[no_mangle]
pub extern "C" fn DatagramTypeToCode(enum_val: DatagramType) -> u8 {
    u8::from(enum_val)
}

// ------------------------------------------------------------
// Enums - EndConnexionReason
// ------------------------------------------------------------
#[no_mangle]
pub extern "C" fn EndConnexionReasonFromCode(code: u8) -> EndConnexionReason {
    EndConnexionReason::from(code)
}

#[no_mangle]
pub extern "C" fn EndConnexionReasonToCode(enum_val: EndConnexionReason) -> u8 {
    u8::from(enum_val)
}

// ------------------------------------------------------------
// Enums - ObjectRequestAction
// ------------------------------------------------------------
#[no_mangle]
pub extern "C" fn ObjectRequestActionFromCode(code: u8) -> ObjectRequestAction {
    ObjectRequestAction::from(code)
}

#[no_mangle]
pub extern "C" fn ObjectRequestActionToCode(enum_val: ObjectRequestAction) -> u8 {
    u8::from(enum_val)
}

// ------------------------------------------------------------
// Enums - TopicAction
// ------------------------------------------------------------
#[no_mangle]
pub extern "C" fn TopicActionFromCode(code: u8) -> TopicAction {
    TopicAction::from(code)
}

#[no_mangle]
pub extern "C" fn TopicActionToCode(enum_val: TopicAction) -> u8 {
    u8::from(enum_val)
}

// ------------------------------------------------------------
// Enums - TopicResponse
// ------------------------------------------------------------
#[no_mangle]
pub extern "C" fn TopicResponseFromCode(code: u8) -> TopicResponse {
    TopicResponse::from(code)
}

#[no_mangle]
pub extern "C" fn TopicResponseToCode(enum_val: TopicResponse) -> u8 {
    u8::from(enum_val)
}

// ------------------------------------------------------------
// LIBS - utility methods
// ------------------------------------------------------------

#[no_mangle]
pub extern "C" fn GetBytesFromSlice(buffer: ByteSlice, from: usize, to: usize) -> VecU8 {
    VecU8::from_vec(get_bytes_from_slice(buffer.as_slice(), from, to))
}

#[no_mangle]
pub extern "C" fn GetU64AtPosition(buffer: ByteSlice, position: usize) -> u64 {
    match get_u64_at_pos(buffer.as_slice(), position) {
        Ok(val) => val,
        Err(_) => 0u64,
    }
}

#[no_mangle]
pub extern "C" fn GetU32AtPosition(buffer: ByteSlice, position: usize) -> u32 {
    match get_u32_at_pos(buffer.as_slice(), position) {
        Ok(val) => val,
        Err(_) => 0u32,
    }
}

#[no_mangle]
pub extern "C" fn GetU16AtPosition(buffer: ByteSlice, position: usize) -> u16 {
    match get_u16_at_pos(buffer.as_slice(), position) {
        Ok(val) => val,
        Err(_) => 0u16,
    }
}

// ------------------------------------------------------------
// Datagrams - connect requests
// ------------------------------------------------------------

#[repr(C)]
pub struct CDtgConnectNack {
    pub datagram_type: DatagramType,
    pub size: Size,
    pub payload: VecU8,
}

impl CDtgConnectNack {
    pub fn new(payload: VecU8) -> CDtgConnectNack {
        let message_size = payload.length as Size;

        CDtgConnectNack {
            datagram_type: DatagramType::ConnectNack,
            size: message_size,
            payload,
        }
    }
}

fn dtg_connect_nack_to_c_type(dtg: DtgConnectNack) -> CDtgConnectNack {
    CDtgConnectNack::new(VecU8::from_vec(dtg.payload))
}

fn dtg_connect_nack_to_rust_type(dtg: CDtgConnectNack) -> DtgConnectNack {
    match from_utf8(&dtg.payload.into_vec()) {
        Ok(str) => DtgConnectNack::new(str),
        Err(_) => DtgConnectNack::new(""),
    }
}

#[no_mangle]
pub extern "C" fn DtgConnectNew() -> DtgConnect {
    DtgConnect::new()
}

#[no_mangle]
pub extern "C" fn DtgConnectAsBytes(datagram: DtgConnect) -> VecU8 {
    VecU8::from_vec(datagram.as_bytes())
}

#[no_mangle]
pub extern "C" fn DtgConnectTryFromBuffer(buffer: ByteSlice) -> *const DtgConnect {
    match DtgConnect::try_from(buffer.as_slice()) {
        Ok(dtg) => &dtg,
        Err(_) => null(),
    }
}

#[no_mangle]
pub extern "C" fn DtgConnectAckNew(peer_id: ClientId, heartbeat_period: u16) -> DtgConnectAck {
    DtgConnectAck::new(peer_id, heartbeat_period)
}

#[no_mangle]
pub extern "C" fn DtgConnectAckAsBytes(datagram: DtgConnectAck) -> VecU8 {
    VecU8::from_vec(datagram.as_bytes())
}

#[no_mangle]
pub extern "C" fn DtgConnectAckTryFromBuffer(buffer: ByteSlice) -> *const DtgConnectAck {
    match DtgConnectAck::try_from(buffer.as_slice()) {
        Ok(dtg) => &dtg,
        Err(_) => null(),
    }
}

#[no_mangle]
pub extern "C" fn DtgConnectNackNew(msg: *const c_char) -> CDtgConnectNack {
    let str_msg = unsafe {
        if msg.is_null() {
            ""
        } else {
            let cstr = CStr::from_ptr(msg);
            match cstr.to_str().ok() {
                None => "",
                Some(val) => val,
            }
        }
    };

    dtg_connect_nack_to_c_type(DtgConnectNack::new(str_msg))
}

#[no_mangle]
pub extern "C" fn DtgConnectNackAsBytes(datagram: CDtgConnectNack) -> VecU8 {
    VecU8::from_vec(dtg_connect_nack_to_rust_type(datagram).as_bytes())
}

#[no_mangle]
pub extern "C" fn DtgConnectNackTryFromBuffer(buffer: ByteSlice) -> *const CDtgConnectNack {
    let dtg = match DtgConnectNack::try_from(buffer.as_slice()) {
        Ok(dtg) => dtg,
        Err(_) => {
            return null();
        }
    };

    &dtg_connect_nack_to_c_type(dtg)
}

// ------------------------------------------------------------
// Datagrams - data requests
// ------------------------------------------------------------

#[repr(C)]
pub struct CDtgData {
    pub datagram_type: DatagramType,
    // 1 byte
    pub size: Size,
    // 2 bytes (u16)
    pub sequence_number: u32,
    // 4 bytes (u32)
    pub topic_id: TopicId,
    // 8 bytes (u64)
    pub payload: VecU8, // size bytes
}

impl CDtgData {
    pub fn new(sequence_number: u32, topic_id: TopicId, payload: VecU8) -> CDtgData {
        CDtgData {
            datagram_type: DatagramType::Data,
            size: payload.length as Size,
            sequence_number,
            topic_id,
            payload,
        }
    }
}

fn dtg_data_to_c_type(dtg: DtgData) -> CDtgData {
    CDtgData::new(
        dtg.sequence_number,
        dtg.topic_id,
        VecU8::from_vec(dtg.payload),
    )
}

fn dtg_data_to_rust_type(dtg: CDtgData) -> DtgData {
    DtgData::new(dtg.sequence_number, dtg.topic_id, dtg.payload.into_vec())
}

#[no_mangle]
pub extern "C" fn DtgDataNew(sequence_number: u32, topic_id: TopicId, payload: VecU8) -> CDtgData {
    dtg_data_to_c_type(DtgData::new(sequence_number, topic_id, payload.into_vec()))
}

#[no_mangle]
pub extern "C" fn DtgDataAsBytes(datagram: CDtgData) -> VecU8 {
    VecU8::from_vec(dtg_data_to_rust_type(datagram).as_bytes())
}

#[no_mangle]
pub extern "C" fn DtgDataTryFromBuffer(buffer: ByteSlice) -> *const CDtgData {
    let dtg = match DtgData::try_from(buffer.as_slice()) {
        Ok(dtg) => dtg,
        Err(_) => {
            return null();
        }
    };
    &dtg_data_to_c_type(dtg)
}

// ------------------------------------------------------------
// Datagrams - heartbeat requests
// ------------------------------------------------------------

#[no_mangle]
pub extern "C" fn DtgHeartbeatNew() -> DtgHeartbeat {
    DtgHeartbeat::new()
}

#[no_mangle]
pub extern "C" fn DtgHeartbeatAsBytes(datagram: DtgHeartbeat) -> VecU8 {
    VecU8::from_vec(datagram.as_bytes())
}

#[no_mangle]
pub extern "C" fn DtgHeartbeatTryFromBuffer(buffer: ByteSlice) -> *const DtgHeartbeat {
    match DtgHeartbeat::try_from(buffer.as_slice()) {
        Ok(dtg) => &dtg,
        Err(_) => null(),
    }
}

#[no_mangle]
pub extern "C" fn DtgHeartbeatRequestNew() -> DtgHeartbeatRequest {
    DtgHeartbeatRequest::new()
}

#[no_mangle]
pub extern "C" fn DtgHeartbeatRequestAsBytes(datagram: DtgHeartbeatRequest) -> VecU8 {
    VecU8::from_vec(datagram.as_bytes())
}

#[no_mangle]
pub extern "C" fn DtgHeartbeatRequestTryFromBuffer(
    buffer: ByteSlice,
) -> *const DtgHeartbeatRequest {
    match DtgHeartbeatRequest::try_from(buffer.as_slice()) {
        Ok(dtg) => &dtg,
        Err(_) => null(),
    }
}

// ------------------------------------------------------------
// Datagrams - Ping requests
// ------------------------------------------------------------
#[no_mangle]
pub extern "C" fn DtgPingNew(ping_id: PingId) -> DtgPing {
    DtgPing::new(ping_id)
}

#[no_mangle]
pub extern "C" fn DtgPingAsBytes(datagram: DtgPing) -> VecU8 {
    VecU8::from_vec(datagram.as_bytes())
}

#[no_mangle]
pub extern "C" fn DtgPingTryFromBuffer(buffer: ByteSlice) -> *const DtgPing {
    match DtgPing::try_from(buffer.as_slice()) {
        Ok(dtg) => &dtg,
        Err(_) => null(),
    }
}

#[no_mangle]
pub extern "C" fn DtgPongNew(pong_id: PingId) -> DtgPong {
    DtgPong::new(pong_id)
}

#[no_mangle]
pub extern "C" fn DtgPongAsBytes(datagram: DtgPong) -> VecU8 {
    VecU8::from_vec(datagram.as_bytes())
}

#[no_mangle]
pub extern "C" fn DtgPongTryFromBuffer(buffer: ByteSlice) -> *const DtgPong {
    match DtgPong::try_from(buffer.as_slice()) {
        Ok(dtg) => &dtg,
        Err(_) => null(),
    }
}

// ------------------------------------------------------------
// Datagrams - misc requests
// ------------------------------------------------------------
#[no_mangle]
pub extern "C" fn DtgServerStatusNew() -> DtgServerStatus {
    DtgServerStatus::new()
}

#[no_mangle]
pub extern "C" fn DtgServerStatusAsBytes(datagram: DtgServerStatus) -> VecU8 {
    VecU8::from_vec(datagram.as_bytes())
}

#[no_mangle]
pub extern "C" fn DtgServerStatusTryFromBuffer(buffer: ByteSlice) -> *const DtgServerStatus {
    match DtgServerStatus::try_from(buffer.as_slice()) {
        Ok(dtg) => &dtg,
        Err(_) => null(),
    }
}

#[no_mangle]
pub extern "C" fn DtgServerStatusACKNew(nb_client: ClientId) -> DtgServerStatusACK {
    DtgServerStatusACK::new(nb_client)
}

#[no_mangle]
pub extern "C" fn DtgServerStatusACKAsBytes(datagram: DtgServerStatusACK) -> VecU8 {
    VecU8::from_vec(datagram.as_bytes())
}

#[no_mangle]
pub extern "C" fn DtgServerStatusACKTryFromBuffer(buffer: ByteSlice) -> *const DtgServerStatusACK {
    match DtgServerStatusACK::try_from(buffer.as_slice()) {
        Ok(dtg) => &dtg,
        Err(_) => null(),
    }
}

// ------------------------------------------------------------
// Datagrams - misc requests
// ------------------------------------------------------------
#[no_mangle]
pub extern "C" fn DtgShutdownNew(reason: EndConnexionReason) -> DtgShutdown {
    DtgShutdown::new(reason)
}

#[no_mangle]
pub extern "C" fn DtgShutdownAsBytes(datagram: DtgShutdown) -> VecU8 {
    VecU8::from_vec(datagram.as_bytes())
}

#[no_mangle]
pub extern "C" fn DtgShutdownTryFromBuffer(buffer: ByteSlice) -> *const DtgShutdown {
    match DtgShutdown::try_from(buffer.as_slice()) {
        Ok(dtg) => &dtg,
        Err(_) => null(),
    }
}

// ------------------------------------------------------------
// Datagrams - object requests
// ------------------------------------------------------------

#[repr(C)]
pub struct CDtgObjectRequest {
    pub datagram_type: DatagramType,
    pub flag: ObjectRequestAction,
    pub size: Size,
    pub object_id: ObjectId,
    pub payload: HashSetWrapperU64,
}

impl CDtgObjectRequest {
    pub fn new(
        flag: ObjectRequestAction,
        object_id: ObjectId,
        topics: HashSetWrapperU64,
    ) -> CDtgObjectRequest {
        let size: Size = (topics.len * size_of::<TopicId>()) as Size; // x = size_of(topics)
        CDtgObjectRequest {
            datagram_type: DatagramType::ObjectRequest,
            flag,
            size,
            object_id,
            payload: topics,
        }
    }
}

fn dtg_object_request_to_c_type(dtg: DtgObjectRequest) -> CDtgObjectRequest {
    CDtgObjectRequest::new(
        dtg.flag,
        dtg.object_id,
        convert_hashset_to_wrapper(&dtg.payload),
    )
}

fn dtg_object_request_to_rust_type(dtg: CDtgObjectRequest) -> DtgObjectRequest {
    DtgObjectRequest::new(
        dtg.flag,
        dtg.object_id,
        convert_wrapper_to_hashset(dtg.payload),
    )
}

#[no_mangle]
pub extern "C" fn DtgObjectRequestNew(
    flag: ObjectRequestAction,
    object_id: ObjectId,
    topics: HashSetWrapperU64,
) -> CDtgObjectRequest {
    CDtgObjectRequest::new(flag, object_id, topics)
}

#[no_mangle]
pub extern "C" fn DtgObjectRequestAsBytes(datagram: CDtgObjectRequest) -> VecU8 {
    VecU8::from_vec(dtg_object_request_to_rust_type(datagram).as_bytes())
}

#[no_mangle]
pub extern "C" fn DtgObjectRequestTryFromBuffer(buffer: ByteSlice) -> *const CDtgObjectRequest {
    let dtg = match DtgObjectRequest::try_from(buffer.as_slice()) {
        Ok(dtg) => dtg,
        Err(_) => {
            return null();
        }
    };

    &dtg_object_request_to_c_type(dtg)
}

#[repr(C)]
pub struct CDtgObjectRequestACK {
    pub datagram_type: DatagramType,
    pub flag: Flag, // Bit field XXXA UDMC (X: Unused, D: delete, M : modify, C: Create, A: subscribe, U: unsubscribe)
    pub object_id: ObjectId,
    pub final_object_id: *const c_ulonglong,
}

impl CDtgObjectRequestACK {
    pub fn new(
        flag: Flag,
        object_id: ObjectId,
        final_object_id: *const c_ulonglong,
    ) -> CDtgObjectRequestACK {
        CDtgObjectRequestACK {
            datagram_type: DatagramType::ObjectRequestAck,
            flag,
            object_id,
            final_object_id,
        }
    }
}

fn dtg_object_request_ack_to_c_type(dtg: DtgObjectRequestACK) -> CDtgObjectRequestACK {
    CDtgObjectRequestACK::new(
        dtg.flag,
        dtg.object_id,
        dtg.final_object_id as *const c_ulonglong,
    )
}

fn dtg_object_request_ack_to_rust_type(dtg: CDtgObjectRequestACK) -> DtgObjectRequestACK {
    DtgObjectRequestACK::new(dtg.flag, dtg.object_id, dtg.final_object_id as ObjectId)
}

#[no_mangle]
pub extern "C" fn DtgObjectRequestACKNew(
    flag: Flag,
    object_id: ObjectId,
    final_object_id: *const c_ulonglong,
) -> CDtgObjectRequestACK {
    CDtgObjectRequestACK::new(flag, object_id, final_object_id)
}

#[no_mangle]
pub extern "C" fn DtgObjectRequestACKAsBytes(datagram: CDtgObjectRequestACK) -> VecU8 {
    VecU8::from_vec(dtg_object_request_ack_to_rust_type(datagram).as_bytes())
}

#[no_mangle]
pub extern "C" fn DtgObjectRequestACKTryFromBuffer(
    buffer: ByteSlice,
) -> *const CDtgObjectRequestACK {
    let dtg = match DtgObjectRequestACK::try_from(buffer.as_slice()) {
        Ok(dtg) => dtg,
        Err(_) => {
            return null();
        }
    };
    &dtg_object_request_ack_to_c_type(dtg)
}

#[repr(C)]
pub struct CDtgObjectRequestNACK {
    pub datagram_type: DatagramType,
    pub flag: u8, // Bitfield XXXA UDMC (X: Unused, D: delete, M : modify, C: Create, A: subscribe, U: unsubscribe)
    pub size: Size,
    pub object_id: u64,
    pub payload: VecU8,
}

impl CDtgObjectRequestNACK {
    pub fn new(flag: u8, object_id: u64, reason: VecU8) -> CDtgObjectRequestNACK {
        CDtgObjectRequestNACK {
            datagram_type: DatagramType::ObjectRequestNack,
            flag,
            size: reason.length as Size,
            object_id,
            payload: reason,
        }
    }
}

fn dtg_object_request_nack_to_c_type(dtg: DtgObjectRequestNACK) -> CDtgObjectRequestNACK {
    CDtgObjectRequestNACK::new(dtg.flag, dtg.object_id, VecU8::from_vec(dtg.payload))
}

fn dtg_object_request_nack_to_rust_type(dtg: CDtgObjectRequestNACK) -> DtgObjectRequestNACK {
    let str_u8 = dtg.payload.into_vec();
    let reason = match from_utf8(&str_u8) {
        Ok(str) => str,
        Err(_) => "",
    };
    DtgObjectRequestNACK::new(dtg.flag, dtg.object_id, reason)
}
