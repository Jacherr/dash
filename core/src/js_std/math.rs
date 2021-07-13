use super::error::{self, MaybeRc};
use crate::{
    gc::Handle,
    vm::value::{
        function::{CallContext, CallResult},
        Value,
    },
};

/// Implements Math.abs
///
/// https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-math.abs
pub fn abs(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let num = ctx
        .args
        .first()
        .map(|c| c.borrow().as_number())
        .unwrap_or(f64::NAN);

    Ok(CallResult::Ready(
        ctx.vm.create_js_value(num.abs()).into_handle(ctx.vm),
    ))
}

/// Implements Math.ceil
///
/// https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-ceil.abs
pub fn ceil(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let num = ctx
        .args
        .first()
        .map(|c| c.borrow().as_number())
        .unwrap_or(f64::NAN);

    Ok(CallResult::Ready(
        ctx.vm.create_js_value(num.ceil()).into_handle(ctx.vm),
    ))
}

/// Implements Math.floor
///
/// https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-floor.abs
pub fn floor(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let num = ctx
        .args
        .first()
        .map(|c| c.borrow().as_number())
        .unwrap_or(f64::NAN);

    Ok(CallResult::Ready(
        ctx.vm.create_js_value(num.floor()).into_handle(ctx.vm),
    ))
}

/// Implements Math.max
///
/// https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-math.max
pub fn max(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let mut arguments = ctx.arguments();
    let mut max = match arguments.next().cloned() {
        Some(value) => value,
        None => {
            return Ok(CallResult::Ready(
                ctx.vm.create_js_value(-f64::INFINITY).into_handle(ctx.vm),
            ))
        }
    };
    let mut max_num = max.borrow().as_number();

    for arg_cell in arguments {
        let arg = arg_cell.borrow().as_number();
        if arg > max_num {
            max_num = arg;
            max = Handle::clone(&arg_cell);
        }
    }
    Ok(CallResult::Ready(max))
}

/// Implements Math.min
///
/// https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-math.min
pub fn min(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let mut arguments = ctx.arguments();
    let mut max = match arguments.next().cloned() {
        Some(value) => value,
        None => {
            return Ok(CallResult::Ready(
                ctx.vm.create_js_value(f64::INFINITY).into_handle(ctx.vm),
            ))
        }
    };
    let mut max_num = max.borrow().as_number();

    for arg_cell in arguments {
        let arg = arg_cell.borrow().as_number();
        if arg < max_num {
            max_num = arg;
            max = Handle::clone(&arg_cell);
        }
    }
    Ok(CallResult::Ready(max))
}

/// Implements Math.pow
///
/// https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-math.pow
pub fn pow(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let mut args = ctx.arguments();

    let lhs = args
        .next()
        .map(|n| n.borrow().as_number())
        .unwrap_or(f64::NAN);

    let rhs = args
        .next()
        .map(|n| n.borrow().as_number())
        .unwrap_or(f64::NAN);

    Ok(CallResult::Ready(
        ctx.vm.create_js_value(lhs.powf(rhs)).into_handle(ctx.vm),
    ))
}

/// Implements Math.random
///
/// https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-math.random
pub fn random(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let maybe_random = ctx.vm.agent.random();

    match maybe_random {
        Some(rand) => Ok(CallResult::Ready(
            ctx.vm.create_js_value(rand).into_handle(ctx.vm),
        )),
        None => Err(error::create_error(
            MaybeRc::Owned("Random number generation failed"),
            ctx.vm,
        )),
    }
}
