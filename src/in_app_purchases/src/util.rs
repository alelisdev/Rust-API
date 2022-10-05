// Used to skip serializing fields if they are none
pub fn is_none<T>(opt: &Option<T>) -> bool {
    opt.is_none()
}

//#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn is_false(value: &bool) -> bool {
    return !value;
}

// This is only used for serialize.
//#[allow(clippy::trivially_copy_pass_by_ref)]
#[allow(dead_code)]
pub fn is_empty<T>(value: &Vec<T>) -> bool {
    value.is_empty()
}
