use std::any::Any;
use std::iter;
use std::rc::Rc;

use crate::gc::handle::Handle;
use crate::gc::trace::Trace;
use crate::throw;
use crate::vm::local::LocalScope;

use super::boxed::Boolean as BoxedBoolean;
use super::boxed::Number as BoxedNumber;
use super::boxed::String as BoxedString;
use super::boxed::Symbol as BoxedSymbol;
use super::object::Object;
use super::object::PropertyKey;
use super::ops::abstractions::conversions::PreferredType;
use super::ops::abstractions::conversions::ValueConversion;
use super::Typeof;
use super::Value;

pub const MAX_SAFE_INTEGER: u64 = 9007199254740991u64;
pub const MAX_SAFE_INTEGERF: f64 = 9007199254740991f64;

unsafe impl Trace for f64 {
    fn trace(&self) {}
}

impl Object for f64 {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        sc.statics.number_prototype.clone().get_property(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: Value) -> Result<(), Value> {
        Ok(())
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        Ok(Value::undefined())
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        // TODO: Reflect.setPrototypeOf(this, value); should throw
        Ok(())
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        Ok(sc.statics.number_prototype.clone().into())
    }

    fn apply<'s>(
        &self,
        scope: &mut LocalScope,
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        throw!(scope, "number is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn type_of(&self) -> Typeof {
        Typeof::Number
    }

    fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
        Some(self)
    }
}

unsafe impl Trace for bool {
    fn trace(&self) {}
}

impl Object for bool {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        sc.statics.boolean_prototype.clone().get_property(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: Value) -> Result<(), Value> {
        Ok(())
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        Ok(Value::undefined())
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        Ok(())
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        Ok(sc.statics.boolean_prototype.clone().into())
    }

    fn apply<'s>(
        &self,
        scope: &mut LocalScope,
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        throw!(scope, "boolean is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn type_of(&self) -> Typeof {
        Typeof::Boolean
    }

    fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
        Some(self)
    }
}

unsafe impl Trace for Rc<str> {
    fn trace(&self) {}
}

// TODO: impl<T: Deref<Target=O>, O: Object> Object for T  possible?
impl Object for Rc<str> {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        if let Some(value) = str::get_property(self, sc, key.clone())?.into_option() {
            return Ok(value);
        }

        sc.statics.string_prototype.clone().get_property(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: Value) -> Result<(), Value> {
        Ok(())
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        Ok(Value::undefined())
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        Ok(())
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        Ok(sc.statics.string_prototype.clone().into())
    }

    fn apply<'s>(
        &self,
        scope: &mut LocalScope,
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        throw!(scope, "string is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        str::own_keys(self)
    }

    fn type_of(&self) -> Typeof {
        str::type_of(self)
    }

    fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
        Some(self)
    }
}

pub fn array_like_keys(len: usize) -> impl Iterator<Item = Value> {
    (0..len)
        .map(|i| i.to_string())
        .chain(iter::once_with(|| "length".to_string()))
        .map(|x| Value::String(x.as_str().into()))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Undefined;
unsafe impl Trace for Undefined {
    fn trace(&self) {}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Null;
unsafe impl Trace for Null {
    fn trace(&self) {}
}

impl Object for Undefined {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        throw!(sc, "Cannot read property {:?} of undefined", key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: Value) -> Result<(), Value> {
        throw!(sc, "Cannot set property {:?} of undefined", key)
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        Ok(Value::undefined())
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        throw!(sc, "Cannot set prototype of undefined")
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        throw!(sc, "Cannot get prototype of undefined")
    }

    fn apply<'s>(
        &self,
        sc: &mut LocalScope,
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        throw!(sc, "undefined is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn type_of(&self) -> Typeof {
        Typeof::Undefined
    }

    fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
        Some(self)
    }
}

impl Object for Null {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        throw!(sc, "Cannot read property {:?} of null", key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: Value) -> Result<(), Value> {
        throw!(sc, "Cannot set property {:?} of null", key)
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        Ok(Value::undefined())
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        throw!(sc, "Cannot set prototype of null")
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        throw!(sc, "Cannot get prototype of null")
    }

    fn apply<'s>(
        &self,
        sc: &mut LocalScope,
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        throw!(sc, "null is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
        Some(self)
    }
}

unsafe impl Trace for str {
    fn trace(&self) {}
}

impl Object for str {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        if let PropertyKey::String(st) = key {
            if st == "length" {
                return Ok(Value::Number(self.len() as f64));
            }

            if let Ok(index) = st.parse::<usize>() {
                let bytes = self.as_bytes();
                if let Some(&byte) = bytes.get(index) {
                    return Ok(Value::String((byte as char).to_string().into()));
                }
            }
        }

        Ok(Value::undefined())
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: Value) -> Result<(), Value> {
        Ok(())
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        Ok(Value::undefined())
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        Ok(())
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        Ok(sc.statics.string_prototype.clone().into())
    }

    fn apply<'s>(
        &self,
        scope: &mut LocalScope,
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        throw!(scope, "string is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        panic!("cannot convert string to any")
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(array_like_keys(self.len()).collect())
    }

    fn type_of(&self) -> Typeof {
        Typeof::String
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Symbol(Rc<str>);

impl Symbol {
    pub fn new(description: Rc<str>) -> Self {
        Symbol(description)
    }
}

unsafe impl Trace for Symbol {
    fn trace(&self) {}
}

impl Object for Symbol {
    fn get_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        sc.statics.symbol_prototype.clone().get_property(sc, key)
    }

    fn set_property(&self, sc: &mut LocalScope, key: PropertyKey<'static>, value: Value) -> Result<(), Value> {
        Ok(())
    }

    fn delete_property(&self, sc: &mut LocalScope, key: PropertyKey) -> Result<Value, Value> {
        Ok(Value::undefined())
    }

    fn set_prototype(&self, sc: &mut LocalScope, value: Value) -> Result<(), Value> {
        Ok(())
    }

    fn get_prototype(&self, sc: &mut LocalScope) -> Result<Value, Value> {
        Ok(sc.statics.symbol_prototype.clone().into())
    }

    fn apply<'s>(
        &self,
        scope: &mut LocalScope,
        callee: Handle<dyn Object>,
        this: Value,
        args: Vec<Value>,
    ) -> Result<Value, Value> {
        throw!(scope, "symbol is not a function")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn own_keys(&self) -> Result<Vec<Value>, Value> {
        Ok(Vec::new())
    }

    fn type_of(&self) -> Typeof {
        Typeof::Symbol
    }

    fn as_primitive_capable(&self) -> Option<&dyn PrimitiveCapabilities> {
        Some(self)
    }
}

pub trait PrimitiveCapabilities: ValueConversion {
    fn as_string(&self) -> Option<Rc<str>> {
        None
    }
    fn as_number(&self) -> Option<f64> {
        None
    }
    fn as_bool(&self) -> Option<bool> {
        None
    }
}

impl PrimitiveCapabilities for f64 {
    fn as_number(&self) -> Option<f64> {
        Some(*self)
    }
}

impl ValueConversion for f64 {
    fn to_primitive(&self, sc: &mut LocalScope, preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::Number(*self))
    }

    fn to_number(&self, sc: &mut LocalScope) -> Result<f64, Value> {
        Ok(*self)
    }

    fn to_boolean(&self) -> Result<bool, Value> {
        Ok(*self != 0.0 && !self.is_nan())
    }

    fn to_string(&self, sc: &mut LocalScope) -> Result<Rc<str>, Value> {
        // TODO: optimize
        Ok(ToString::to_string(self).into())
    }

    fn length_of_array_like(&self, sc: &mut LocalScope) -> Result<usize, Value> {
        todo!() // TODO
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle<dyn Object>, Value> {
        let num = BoxedNumber::new(sc, *self);
        Ok(sc.register(num))
    }
}

impl PrimitiveCapabilities for bool {
    fn as_bool(&self) -> Option<bool> {
        Some(*self)
    }
}

impl ValueConversion for bool {
    fn to_primitive(&self, sc: &mut LocalScope, preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::Boolean(*self))
    }

    fn to_number(&self, sc: &mut LocalScope) -> Result<f64, Value> {
        Ok(*self as u8 as f64)
    }

    fn to_boolean(&self) -> Result<bool, Value> {
        Ok(*self)
    }

    fn to_string(&self, sc: &mut LocalScope) -> Result<Rc<str>, Value> {
        Ok(if *self {
            sc.statics().get_true()
        } else {
            sc.statics().get_false()
        })
    }

    fn length_of_array_like(&self, sc: &mut LocalScope) -> Result<usize, Value> {
        todo!() // TODO
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle<dyn Object>, Value> {
        let bool = BoxedBoolean::new(sc, *self);
        Ok(sc.register(bool))
    }
}

impl PrimitiveCapabilities for Rc<str> {
    fn as_string(&self) -> Option<Rc<str>> {
        Some(self.clone())
    }
}

impl ValueConversion for Rc<str> {
    fn to_primitive(&self, sc: &mut LocalScope, preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::String(Rc::clone(self)))
    }

    fn to_number(&self, sc: &mut LocalScope) -> Result<f64, Value> {
        self.parse().or_else(|e| throw!(sc, "{}", e))
    }

    fn to_boolean(&self) -> Result<bool, Value> {
        Ok(!self.is_empty())
    }

    fn to_string(&self, sc: &mut LocalScope) -> Result<Rc<str>, Value> {
        Ok(Rc::clone(self))
    }

    fn length_of_array_like(&self, sc: &mut LocalScope) -> Result<usize, Value> {
        Ok(self.len())
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle<dyn Object>, Value> {
        let bool = BoxedString::new(sc, self.clone());
        Ok(sc.register(bool))
    }
}

impl PrimitiveCapabilities for Undefined {}

impl ValueConversion for Undefined {
    fn to_primitive(&self, sc: &mut LocalScope, preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::undefined())
    }

    fn to_number(&self, sc: &mut LocalScope) -> Result<f64, Value> {
        Ok(f64::NAN)
    }

    fn to_boolean(&self) -> Result<bool, Value> {
        Ok(false)
    }

    fn to_string(&self, sc: &mut LocalScope) -> Result<Rc<str>, Value> {
        Ok(sc.statics().undefined_str())
    }

    fn length_of_array_like(&self, sc: &mut LocalScope) -> Result<usize, Value> {
        todo!() // TODO: throw?
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle<dyn Object>, Value> {
        throw!(sc, "Cannot convert undefined to object")
    }
}

impl PrimitiveCapabilities for Null {}

impl ValueConversion for Null {
    fn to_primitive(&self, sc: &mut LocalScope, preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::null())
    }

    fn to_number(&self, sc: &mut LocalScope) -> Result<f64, Value> {
        Ok(0.0)
    }

    fn to_boolean(&self) -> Result<bool, Value> {
        Ok(false)
    }

    fn to_string(&self, sc: &mut LocalScope) -> Result<Rc<str>, Value> {
        Ok(sc.statics().null_str())
    }

    fn length_of_array_like(&self, sc: &mut LocalScope) -> Result<usize, Value> {
        todo!() // TODO: throw?
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle<dyn Object>, Value> {
        throw!(sc, "Cannot convert null to object");
    }
}

impl PrimitiveCapabilities for Symbol {}

impl ValueConversion for Symbol {
    fn to_primitive(&self, sc: &mut LocalScope, preferred_type: Option<PreferredType>) -> Result<Value, Value> {
        Ok(Value::Symbol(self.clone()))
    }

    fn to_number(&self, sc: &mut LocalScope) -> Result<f64, Value> {
        throw!(sc, "Cannot convert symbol to number");
    }

    fn to_boolean(&self) -> Result<bool, Value> {
        Ok(true)
    }

    fn to_string(&self, sc: &mut LocalScope) -> Result<Rc<str>, Value> {
        throw!(sc, "Cannot convert symbol to string");
    }

    fn length_of_array_like(&self, sc: &mut LocalScope) -> Result<usize, Value> {
        todo!() // TODO: throw?
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle<dyn Object>, Value> {
        let sym = BoxedSymbol::new(sc, self.clone());
        Ok(sc.register(sym))
    }
}
