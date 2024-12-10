use crate::*;
use crate::run::interpreter::{Interpreter, CHAR_MODULE};

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: CHAR_MODULE = make_map!{
        "lower" = native_fn!(_lower),
        "upper" = native_fn!(_upper),
        "to_digit" = native_fn!(_to_digit),
        "to_hex" = native_fn!(_to_hex),
        "to_bin" = native_fn!(_to_bin),
        "to_radix" = native_fn!(_to_radix),
        "is_lower" = native_fn!(_is_lower),
        "is_upper" = native_fn!(_is_upper),
        "is_alphabetic" = native_fn!(_is_alphabetic),
        "is_numeric" = native_fn!(_is_numeric),
        "is_digit" = native_fn!(_is_digit),
        "is_hex" = native_fn!(_is_hex),
        "is_control" = native_fn!(_is_control),
        "is_graphic" = native_fn!(_is_graphic),
        "is_punct" = native_fn!(_is_punct),
        "is_space" = native_fn!(_is_space),
    });
}

define_native_fn!(_lower (_i args): value = typed!(args: Char) => {
    Ok(Some(value.to_ascii_lowercase().into()))
});
define_native_fn!(_upper (_i args): value = typed!(args: Char) => {
    Ok(Some(value.to_ascii_uppercase().into()))
});
define_native_fn!(_to_digit (_i args): value = typed!(args: Char) => {
    Ok(value.to_digit(10).map(|v| Value::Int(v as i64)))
});
define_native_fn!(_to_hex (_i args): value = typed!(args: Char) => {
    Ok(value.to_digit(16).map(|v| Value::Int(v as i64)))
});
define_native_fn!(_to_bin (_i args): value = typed!(args: Char) => {
    Ok(value.to_digit(2).map(|v| Value::Int(v as i64)))
});
define_native_fn!(_to_radix (_i args): value = typed!(args: Char), radix = typed!(args: Int) => {
    Ok(value.to_digit(radix as u32).map(|v| Value::Int(v as i64)))
});
define_native_fn!(_is_alphabetic (_i args): value = typed!(args: Char) => {
    Ok(Some(value.is_ascii_alphabetic().into()))
});
define_native_fn!(_is_numeric (_i args): value = typed!(args: Char) => {
    Ok(Some(value.is_ascii_alphanumeric().into()))
});
define_native_fn!(_is_control (_i args): value = typed!(args: Char) => {
    Ok(Some(value.is_ascii_control().into()))
});
define_native_fn!(_is_digit (_i args): value = typed!(args: Char) => {
    Ok(Some(value.is_ascii_digit().into()))
});
define_native_fn!(_is_graphic (_i args): value = typed!(args: Char) => {
    Ok(Some(value.is_ascii_graphic().into()))
});
define_native_fn!(_is_hex (_i args): value = typed!(args: Char) => {
    Ok(Some(value.is_ascii_hexdigit().into()))
});
define_native_fn!(_is_lower (_i args): value = typed!(args: Char) => {
    Ok(Some(value.is_ascii_lowercase().into()))
});
define_native_fn!(_is_upper (_i args): value = typed!(args: Char) => {
    Ok(Some(value.is_ascii_uppercase().into()))
});
define_native_fn!(_is_punct (_i args): value = typed!(args: Char) => {
    Ok(Some(value.is_ascii_punctuation().into()))
});
define_native_fn!(_is_space (_i args): value = typed!(args: Char) => {
    Ok(Some(value.is_ascii_whitespace().into()))
});