use crate::throw;
use crate::value::function::native::CallContext;
use crate::value::object::PropertyKey;
use crate::value::ops::abstractions::conversions::ValueConversion;
use crate::value::set::Set;
use crate::value::Value;
use crate::value::ValueContext;

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let set = Set::new(cx.scope);
    if let Some(iter) = cx.args.first() {
        let len = iter.length_of_array_like(cx.scope)?;

        for i in 0..len {
            let i = i.to_string();
            let item = iter.get_property(cx.scope, PropertyKey::String(i.into()))?;
            set.add(item);
        }
    }

    Ok(Value::Object(cx.scope.register(set)))
}

pub fn add(cx: CallContext) -> Result<Value, Value> {
    let this = match &cx.this {
        Value::Object(o) | Value::External(o) => o.as_any().downcast_ref::<Set>(),
        _ => throw!(cx.scope, "Incompatible receiver"),
    };
    let this = match &this {
        Some(set) => set,
        _ => throw!(cx.scope, "Incompatible receiver"),
    };

    let item = cx.args.first().unwrap_or_undefined();
    this.add(item);

    Ok(cx.this)
}

pub fn has(cx: CallContext) -> Result<Value, Value> {
    let this = match &cx.this {
        Value::Object(o) | Value::External(o) => o.as_any().downcast_ref::<Set>(),
        _ => throw!(cx.scope, "Incompatible receiver"),
    };
    let this = match &this {
        Some(set) => set,
        _ => throw!(cx.scope, "Incompatible receiver"),
    };

    let item = cx.args.first().unwrap_or_undefined();
    Ok(Value::Boolean(this.has(&item)))
}

pub fn delete(cx: CallContext) -> Result<Value, Value> {
    let this = match &cx.this {
        Value::Object(o) | Value::External(o) => o.as_any().downcast_ref::<Set>(),
        _ => throw!(cx.scope, "Incompatible receiver"),
    };
    let this = match &this {
        Some(set) => set,
        _ => throw!(cx.scope, "Incompatible receiver"),
    };

    let item = cx.args.first().unwrap_or_undefined();
    this.delete(&item);

    Ok(cx.this)
}

pub fn clear(cx: CallContext) -> Result<Value, Value> {
    let this = match &cx.this {
        Value::Object(o) | Value::External(o) => o.as_any().downcast_ref::<Set>(),
        _ => throw!(cx.scope, "Incompatible receiver"),
    };
    let this = match &this {
        Some(set) => set,
        _ => throw!(cx.scope, "Incompatible receiver"),
    };

    this.clear();

    Ok(cx.this)
}

pub fn size(cx: CallContext) -> Result<Value, Value> {
    let this = match &cx.this {
        Value::Object(o) | Value::External(o) => o.as_any().downcast_ref::<Set>(),
        _ => throw!(cx.scope, "Incompatible receiver"),
    };
    let this = match &this {
        Some(set) => set,
        _ => throw!(cx.scope, "Incompatible receiver"),
    };

    Ok(Value::number(this.size() as f64))
}