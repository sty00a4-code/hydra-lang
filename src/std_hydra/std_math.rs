use crate::run::interpreter::Interpreter;
use crate::*;
use core::f64;
use rand::random;

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: "math" = make_map!{
        "nan" = f64::NAN,
        "inf" = f64::INFINITY,
        "pi" = f64::consts::PI,
        "tau" = f64::consts::TAU,
        "e" = f64::consts::E,
        "epsilon" = f64::EPSILON,
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
        "random" = native_fn!(_random),
        "random_int" = native_fn!(_random_int),
        "random_choice" = native_fn!(_random_choice),
    });
}
pub fn make_float(idx: usize, value: Value) -> Result<f64, Box<dyn Error>> {
    match value {
        Value::Int(value) => Ok(value as f64),
        Value::Float(value) => Ok(value),
        value => Err(format!(
            "expected {} for argument #{}, got {}",
            [
                Value::Int(Default::default()).typ(),
                Value::Float(Default::default()).typ()
            ]
            .join("/"),
            idx + 1,
            value.typ(),
        )
        .into()),
    }
}
define_native_fn!(_floor (_i args): value = typed!(args: Float) => {
    Ok(Some(value.floor().into()))
});
define_native_fn!(_ceil (_i args): value = typed!(args: Float) => {
    Ok(Some(value.ceil().into()))
});
define_native_fn!(_round (_i args): value = typed!(args: Float) => {
    Ok(Some(value.round().into()))
});
define_native_fn!(_round_ties_even (_i args): value = typed!(args: Float) => {
    Ok(Some(value.round_ties_even().into()))
});
define_native_fn!(_abs (_i args): value = typed!(args) => {
    Ok(Some(match value {
        Value::Int(v) => Value::Int(v.abs()),
        Value::Float(v) => Value::Float(v.abs()),
        value => return Err(format!(
            "expected {} for argument #1, got {}",
            [Value::Int(Default::default()).typ(), Value::Float(Default::default()).typ()].join("/"),
            value.typ()
        ).into())
    }))
});
define_native_fn!(_sqrt (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.sqrt().into()))
});
define_native_fn!(_cbrt (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.cbrt().into()))
});
define_native_fn!(_max (_i args): a = typed!(args), b = typed!(args) => {
    Ok(Some(match (a, b) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a.max(b)),
        (Value::Float(a), Value::Float(b)) => Value::Float(a.max(b)),
        (Value::Int(a), Value::Float(b)) => Value::Float((a as f64).max(b)),
        (Value::Float(a), Value::Int(b)) => Value::Float(a.max(b as f64)),
        (a, b) => return Err(format!(
            "expected {} for argument #1 and #2, got {} and {}",
            [Value::Int(Default::default()).typ(), Value::Float(Default::default()).typ()].join("/"),
            a.typ(),
            b.typ(),
        ).into())
    }))
});
define_native_fn!(_min (_i args): a = typed!(args), b = typed!(args) => {
    Ok(Some(match (a, b) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a.min(b)),
        (Value::Float(a), Value::Float(b)) => Value::Float(a.min(b)),
        (Value::Int(a), Value::Float(b)) => Value::Float((a as f64).min(b)),
        (Value::Float(a), Value::Int(b)) => Value::Float(a.min(b as f64)),
        (a, b) => return Err(format!(
            "expected {} for argument #1 and #2, got {} and {}",
            [Value::Int(Default::default()).typ(), Value::Float(Default::default()).typ()].join("/"),
            a.typ(),
            b.typ(),
        ).into())
    }))
});
define_native_fn!(_cos (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.cos().into()))
});
define_native_fn!(_sin (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.sin().into()))
});
define_native_fn!(_tan (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.tan().into()))
});
define_native_fn!(_cosh (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.cosh().into()))
});
define_native_fn!(_sinh (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.sinh().into()))
});
define_native_fn!(_tanh (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.tanh().into()))
});
define_native_fn!(_acos (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.acos().into()))
});
define_native_fn!(_asin (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.asin().into()))
});
define_native_fn!(_atan (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.atan().into()))
});
define_native_fn!(_acosh (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.acosh().into()))
});
define_native_fn!(_asinh (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.asinh().into()))
});
define_native_fn!(_atanh (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.atanh().into()))
});
define_native_fn!(_atan2 (_i args): a = typed!(args), b = typed!(args) => {
    let a = make_float(0, a)?;
    let b = make_float(1, b)?;
    Ok(Some(a.atan2(b).into()))
});
define_native_fn!(_fract (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.fract().into()))
});
define_native_fn!(_exp (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.exp().into()))
});
define_native_fn!(_exp2 (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.exp2().into()))
});
define_native_fn!(_exp_m1 (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.exp_m1().into()))
});
define_native_fn!(_recip (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.recip().into()))
});
define_native_fn!(_clamp (_i args): value = typed!(args), min = typed!(args), max = typed!(args) => {
    let value = make_float(0, value)?;
    let min = make_float(1, min)?;
    let max = make_float(2, max)?;
    Ok(Some(value.clamp(min, max).into()))
});
define_native_fn!(_ln (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.ln().into()))
});
define_native_fn!(_ln_1p (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.ln_1p().into()))
});
define_native_fn!(_log (_i args): value = typed!(args), base = typed!(args) => {
    let value = make_float(0, value)?;
    let base = make_float(1, base)?;
    Ok(Some(value.log(base).into()))
});
define_native_fn!(_log2 (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.log2().into()))
});
define_native_fn!(_log10 (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.log10().into()))
});
define_native_fn!(_radians (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.to_radians().into()))
});
define_native_fn!(_degrees (_i args): value = typed!(args) => {
    let value = make_float(0, value)?;
    Ok(Some(value.to_degrees().into()))
});
define_native_fn!(_random (_i args): => {
    Ok(Some(random::<f64>().into()))
});
define_native_fn!(_random_int (_i args): min = typed!(args: Int), max = typed!(args: Int?)  => {
    if let Some(max) = max {
        Ok(Some(((random::<f64>() * (max - min) as f64) as i64 + min).into()))
    } else {
        Ok(Some(((random::<f64>() * min as f64) as i64).into()))
    }
});
define_native_fn!(_random_choice (_i args): collection = typed!(args)  => {
    match collection {
        Value::Vector(values) => {
            let len = values.lock().unwrap().len();
            let index = (random::<f64>() * len as f64) as usize;
            Ok(values.lock().unwrap().get(index).cloned())
        }
        Value::Tuple(values) => {
            let len = values.lock().unwrap().len();
            let index = (random::<f64>() * len as f64) as usize;
            Ok(values.lock().unwrap().get(index).cloned())
        }
        Value::Map(values) => {
            let len = values.lock().unwrap().len();
            let index = (random::<f64>() * len as f64) as usize;
            Ok(Some(values.lock().unwrap().keys().cloned().collect::<Vec<String>>().remove(index).into()))
        }
        collection => Err(format!(
            "expected {} for argument #1, got {}",
            [
                Value::Vector(Default::default()).typ(),
                Value::Tuple(Default::default()).typ(),
                Value::Map(Default::default()).typ(),
            ].join("/"),
            collection.typ()
        ).into())
    }
});
