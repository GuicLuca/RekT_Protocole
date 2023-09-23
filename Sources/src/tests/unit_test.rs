use std::collections::HashSet;

use crate::libs::types::TopicId;
use crate::libs::utils::{diff_hashsets, get_bytes_from_slice, get_u16_at_pos, get_u32_at_pos, get_u64_at_pos, u8_to_vec_be, vec_to_u8};

// ------------------------------------------------
//    Workflow test
// ------------------------------------------------
#[test]
fn test_main_end_successfully() {
    assert!(crate::main().is_ok());
}


// ------------------------------------------------
//    Lib test
// ------------------------------------------------

#[test]
fn test_get_bytes_from_slice() {
    let buffer: Vec<u8> = vec!(1,2,3,4,5,6);

    let good_slice: Vec<u8>  = vec!(3,4,5);
    let wrong_slice: Vec<u8>  = vec!(3,4);

    assert_eq!(good_slice, get_bytes_from_slice(&buffer, 2,4));
    assert_ne!(wrong_slice, get_bytes_from_slice(&buffer, 2, 4));
}

#[test]
fn test_get_bytes_at_position() {
    let mut buffer: Vec<u8> = Vec::with_capacity(8);
    let number: u64 = u64::MAX;
    buffer.extend(number.to_le_bytes().into_iter());


    assert_eq!(Ok(u64::MAX), get_u64_at_pos(&buffer, 0));
    assert_eq!(Ok(u32::MAX), get_u32_at_pos(&buffer, 4));
    assert_eq!(Ok(u16::MAX), get_u16_at_pos(&buffer, 6));
    assert_eq!(Ok(u16::MAX), get_u16_at_pos(&buffer, 0));
}

#[test]
fn test_diff_hashset() {
    let original_set: HashSet<TopicId> = HashSet::from([1, 2, 3, 4, 5]);
    let new_set: HashSet<TopicId> = HashSet::from([2, 3, 4, 5, 6]);
    assert_eq!((vec!(6 as TopicId),vec!(1 as TopicId)),diff_hashsets(&new_set, &original_set));
}

#[test]
fn test_vec_to_u8() {
    let vector: Vec<u8> = vec!(1,1,1,1,1,1,1,1);
    let vector2: Vec<u8> = vec!(1,0,1,1,1,1,1,1);
    let vector3: Vec<u8> = vec!(0,0,0,0,0,1,0,1);

    assert_eq!(u8::MAX, vec_to_u8(vector));
    assert_eq!(191u8, vec_to_u8(vector2));
    assert_eq!(5u8, vec_to_u8(vector3));
}

#[test]
fn test_u8_to_vec() {
    assert_eq!(vec![1,0,0,0,0,0,0,0], u8_to_vec_be(128));
    assert_eq!(vec![1,1,1,1,1,1,1,1], u8_to_vec_be(u8::MAX));
    assert_eq!(vec![0,0,0,0,0,0,0,1], u8_to_vec_be(1));
}