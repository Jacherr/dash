use std::borrow::Cow;

use crate::{
    gc::Handle,
    js_std::error::{self, MaybeRc},
    json::parser::Parser,
    vm::value::{
        function::{CallContext, CallResult},
        Value, ValueKind,
    },
};

/// Implements JSON.parse
///
/// https://tc39.es/ecma262/multipage/structured-data.html#sec-json.parse
pub fn parse(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let source_cell = ctx
        .args
        .first()
        .cloned()
        .unwrap_or_else(|| Value::new(ValueKind::Undefined).into_handle(ctx.vm));

    let source = source_cell.borrow();
    let source_str = source.to_string();

    let parsed = Parser::new(source_str.as_bytes())
        .parse()
        .map_err(|e| error::create_error(MaybeRc::Owned(&e.to_string()), ctx.vm))?
        .into_js_value(ctx.vm)
        .map_err(|e| error::create_error(MaybeRc::Owned(&e.to_string()), ctx.vm))?;

    Ok(CallResult::Ready(parsed.into_handle(ctx.vm)))
}

/// Implements JSON.stringify
///
/// https://tc39.es/ecma262/multipage/structured-data.html#sec-json.stringify
pub fn stringify(ctx: CallContext) -> Result<CallResult, Handle<Value>> {
    let target = ctx.args.first().map(|c| c.borrow());

    let result = target
        .as_ref()
        .and_then(|c| c.to_json())
        .unwrap_or(Cow::Borrowed("undefined"));

    Ok(CallResult::Ready(
        ctx.vm
            .create_js_value(String::from(result))
            .into_handle(ctx.vm),
    ))
}
