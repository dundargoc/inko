use crate::mem::{tagged_int, Bool, ByteArray, Int, String as InkoString};
use crate::state::State;
use std::cmp::min;

#[no_mangle]
pub unsafe extern "system" fn inko_byte_array_new(
    state: *const State,
) -> *mut ByteArray {
    ByteArray::alloc((*state).byte_array_class, Vec::new())
}

#[no_mangle]
pub unsafe extern "system" fn inko_byte_array_push(
    bytes: *mut ByteArray,
    value: i64,
) {
    (*bytes).value.push(value as u8);
}

#[no_mangle]
pub unsafe extern "system" fn inko_byte_array_pop(
    bytes: *mut ByteArray,
) -> *const Int {
    if let Some(value) = (*bytes).value.pop() {
        tagged_int(value as i64)
    } else {
        tagged_int(-1)
    }
}

#[no_mangle]
pub unsafe extern "system" fn inko_byte_array_set(
    bytes: *mut ByteArray,
    index: i64,
    value: i64,
) -> *const Int {
    let bytes = &mut (*bytes).value;
    let index_ref = bytes.get_unchecked_mut(index as usize);
    let old_value = *index_ref;

    *index_ref = value as u8;
    tagged_int(old_value as i64)
}

#[no_mangle]
pub unsafe extern "system" fn inko_byte_array_get(
    bytes: *mut ByteArray,
    index: i64,
) -> *const Int {
    tagged_int(*(*bytes).value.get_unchecked(index as usize) as i64)
}

#[no_mangle]
pub unsafe extern "system" fn inko_byte_array_remove(
    bytes: *mut ByteArray,
    index: i64,
) -> *const Int {
    tagged_int((*bytes).value.remove(index as usize) as i64)
}

#[no_mangle]
pub unsafe extern "system" fn inko_byte_array_length(
    state: *const State,
    bytes: *const ByteArray,
) -> *const Int {
    Int::new((*state).int_class, (*bytes).value.len() as i64)
}

#[no_mangle]
pub unsafe extern "system" fn inko_byte_array_eq(
    state: *const State,
    lhs: *const ByteArray,
    rhs: *const ByteArray,
) -> *const Bool {
    let state = &*state;

    if (*lhs).value == (*rhs).value {
        state.true_singleton
    } else {
        state.false_singleton
    }
}

#[no_mangle]
pub unsafe extern "system" fn inko_byte_array_clear(bytes: *mut ByteArray) {
    (*bytes).value.clear();
}

#[no_mangle]
pub unsafe extern "system" fn inko_byte_array_clone(
    state: *const State,
    bytes: *const ByteArray,
) -> *mut ByteArray {
    ByteArray::alloc((*state).byte_array_class, (*bytes).value.clone())
}

#[no_mangle]
pub unsafe extern "system" fn inko_byte_array_drop(array: *mut ByteArray) {
    ByteArray::drop(array);
}

#[no_mangle]
pub unsafe extern "system" fn inko_byte_array_to_string(
    state: *const State,
    bytes: *const ByteArray,
) -> *const InkoString {
    InkoString::from_bytes((*state).string_class, (*bytes).value.clone())
}

#[no_mangle]
pub unsafe extern "system" fn inko_byte_array_drain_to_string(
    state: *const State,
    bytes: *mut ByteArray,
) -> *const InkoString {
    InkoString::from_bytes((*state).string_class, (*bytes).take_bytes())
}

#[no_mangle]
pub unsafe extern "system" fn inko_byte_array_slice(
    state: *const State,
    bytes: *const ByteArray,
    start: i64,
    length: i64,
) -> *mut ByteArray {
    let bytes = &*bytes;
    let end = min((start + length) as usize, bytes.value.len());

    ByteArray::alloc(
        (*state).byte_array_class,
        bytes.value[start as usize..end].to_vec(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn inko_byte_array_append(
    target: *mut ByteArray,
    source: *mut ByteArray,
) {
    (*target).value.append(&mut (*source).value);
}

#[no_mangle]
pub unsafe extern "system" fn inko_byte_array_copy_from(
    state: *const State,
    target: *mut ByteArray,
    source: *mut ByteArray,
    start: i64,
    length: i64,
) -> *const Int {
    let target = &mut *target;
    let source = &mut *source;
    let end = min((start + length) as usize, source.value.len());
    let slice = &source.value[start as usize..end];
    let amount = slice.len() as i64;

    target.value.extend_from_slice(slice);
    Int::new((*state).int_class, amount)
}

#[no_mangle]
pub unsafe extern "system" fn inko_byte_array_resize(
    bytes: *mut ByteArray,
    size: i64,
    filler: i64,
) {
    (*bytes).value.resize(size as usize, filler as u8);
}
