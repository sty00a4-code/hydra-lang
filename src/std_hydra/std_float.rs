use crate::*;
use crate::run::interpreter::{Interpreter, FLOAT_MODULE};
use super::std_math::*;

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: FLOAT_MODULE = make_map!{
        "floor" = native_fn!(_floor),
        "ceil" = native_fn!(_ceil),
        "round" = native_fn!(_round),
        "round_ties_even" = native_fn!(_round_ties_even),
        "abs" = native_fn!(_abs),
        "sqrt" = native_fn!(_sqrt),
        "cbrt" = native_fn!(_cbrt),
        "max" = native_fn!(_max),
        "min" = native_fn!(_min),
        "cos" = native_fn!(_cos),
        "sin" = native_fn!(_sin),
        "tan" = native_fn!(_tan),
        "cosh" = native_fn!(_cosh),
        "sinh" = native_fn!(_sinh),
        "tanh" = native_fn!(_tanh),
        "acos" = native_fn!(_acos),
        "asin" = native_fn!(_asin),
        "atan" = native_fn!(_atan),
        "acosh" = native_fn!(_acosh),
        "asinh" = native_fn!(_asinh),
        "atanh" = native_fn!(_atanh),
        "atan2" = native_fn!(_atan2),
        "fract" = native_fn!(_fract),
        "exp" = native_fn!(_exp),
        "exp2" = native_fn!(_exp2),
        "exp_m1" = native_fn!(_exp_m1),
        "recip" = native_fn!(_recip),
        "clamp" = native_fn!(_clamp),
        "ln" = native_fn!(_ln),
        "ln_1p" = native_fn!(_ln_1p),
        "log" = native_fn!(_log),
        "log10" = native_fn!(_log10),
        "log2" = native_fn!(_log2),
        "radians" = native_fn!(_radians),
        "degrees" = native_fn!(_degrees),
    });
}