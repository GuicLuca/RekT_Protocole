use std::collections::HashSet;
use std::mem::size_of;

use crate::libs::types::TopicId;

/**===================================*
*                                     *
*     Array/vec/set manipulators      *
*                                     *
*=====================================*/

/**
 * This functions return a bytes slice according to
 * the given bounds. FROM and TO are include in the returned slice.
 *
 * @param buffer: &[u8], the original array,
 * @param from: usize, first bound,
 * @param to: usize, last bound,
 *
 * @return Vec<u8>, the slice requested
 */
pub fn get_bytes_from_slice(
    buffer: &[u8],
    from: usize,
    to: usize,
) -> Vec<u8> {
    // 1 - check bound validity
    match () {
        _ if to < from => panic!("from is greater than to"),
        _ if to >= buffer.len() => panic!("to is greater than the last index"),
        _ if to == from => return Vec::new(),
        _ => (),
    }

    // 2 - return the correct slice
    buffer[from..to + 1].into()
}


/**
 * This method is an helper to find an u64 at position
 * in a buffer of u8
 *
 * @param buffer: &[u8], the source of the u64
 * @param position: usize, the position of the first byte of the u64
 *
 * @return u64
 */
pub fn get_u64_at_pos(buffer: &[u8], position: usize) -> Result<u64, &str>
{
    let slice = get_bytes_from_slice(buffer, position, position + size_of::<u64>() - 1);
    if slice.len() != 8 {
        return Err("Slice len is invalid to convert it into an u64.");
    }
    Ok(u64::from_le_bytes(slice.try_into().unwrap()))
}

/**
 * This method is an helper to find an u32 at position
 * in a buffer of u8
 *
 * @param buffer: &[u8], the source of the u32
 * @param position: usize, the position of the first byte of the u32
 *
 * @return u32
 */
pub fn get_u32_at_pos(buffer: &[u8], position: usize) -> Result<u32, &str>
{
    let slice = get_bytes_from_slice(buffer, position, position + size_of::<u32>() - 1);
    if slice.len() != 4 {
        return Err("Slice len is invalid to convert it into an u32.");
    }
    Ok(u32::from_le_bytes(slice.try_into().unwrap()))
}

/**
 * This method is an helper to find an u16 at position
 * in a buffer of u8
 *
 * @param buffer: &[u8], the source of the u16
 * @param position: usize, the position of the first byte of the u16
 *
 * @return u16
 */
pub fn get_u16_at_pos(buffer: &[u8], position: usize) -> Result<u16, &str>
{
    let slice = get_bytes_from_slice(buffer, position, position + size_of::<u16>() - 1);
    if slice.len() != 2 {
        return Err("Slice len is invalid to convert it into an u16.");
    }
    Ok(u16::from_le_bytes(slice.try_into().unwrap()))
}


/**
 * This method return a tuple off two vec containing
 * added and removed values.
 *
 * @param new_set: &HashSet<TopicId>, The new set containing incoming values
 * @param current_set: &HashSet<TopicId>, the current set containing actual values
 *
 * @return added_values, removed_values: (Vec<TopicId>, Vec<TopicId>): two vectors containing differences from the original set
 */
pub fn diff_hashsets(new_set: &HashSet<TopicId>, current_set: &HashSet<TopicId>) -> (Vec<TopicId>, Vec<TopicId>) {
    let added_values = new_set.difference(current_set).cloned().collect();
    let removed_values = current_set.difference(new_set).cloned().collect();
    (added_values, removed_values)
}

/**
 * This method return the u8 image of the
 * given bitfields. The bitfield must be in little endian
 *
 * @param bitfields: Vec<u8>
 *
 * @return u8
 */
pub fn vec_to_u8(bitfield: Vec<u8>) -> u8 {
    if bitfield.len() != 8 {
        return panic!("Bitfield length is invalid ! It must be exactly 8.");
    }
    (bitfield[0] << 7) | (bitfield[1] << 6) | (bitfield[2] << 5) | (bitfield[3] << 4) | (bitfield[4] << 3) | (bitfield[5] << 2) | (bitfield[6] << 1) | (bitfield[7] << 0)
}

/**
 * This method return the bitfield image of the
 * given u8.
 *
 * @param number: u8
 *
 * @return Vec<u8>, bitfield in big endian
 */
pub fn u8_to_vec_be(number: u8) -> Vec<u8> {
    let mut bits = Vec::with_capacity(8);
    for i in 0..8 {
        bits.push(if (number & (1 << i)) != 0 { 1 } else { 0 });
    }
    bits.reverse();
    bits
}