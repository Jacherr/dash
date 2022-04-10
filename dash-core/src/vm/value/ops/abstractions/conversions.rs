use std::rc::Rc;

use crate::gc::handle::Handle;
use crate::throw;
use crate::vm::local::LocalScope;
use crate::vm::value::boxed::Boolean;
use crate::vm::value::boxed::Number;
use crate::vm::value::boxed::String as BoxedString;
use crate::vm::value::boxed::Symbol as BoxedSymbol;
use crate::vm::value::object::Object;
use crate::vm::value::primitive::MAX_SAFE_INTEGERF;
use crate::vm::value::Value;

pub trait ValueConversion {
    fn to_primitive(
        &self,
        sc: &mut LocalScope,
        preferred_type: Option<PreferredType>,
    ) -> Result<Value, Value>;
    fn to_number(&self) -> Result<f64, Value>;
    fn to_length(&self) -> Result<f64, Value>;
    fn to_length_u(&self) -> Result<usize, Value>;
    fn to_integer_or_infinity(&self) -> Result<f64, Value>;
    fn to_boolean(&self) -> Result<bool, Value>;
    fn to_string(&self, sc: &mut LocalScope) -> Result<Rc<str>, Value>;
    fn length_of_array_like(&self, sc: &mut LocalScope) -> Result<usize, Value>;
    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle<dyn Object>, Value>;
}

impl ValueConversion for Value {
    fn to_number(&self) -> Result<f64, Value> {
        match self {
            Value::Number(n) => Ok(*n),
            _ => todo!(), // TODO: implement other cases
        }
    }

    fn to_boolean(&self) -> Result<bool, Value> {
        match self {
            Value::Boolean(b) => Ok(*b),
            _ => todo!(), // TODO: implement other cases
        }
    }

    fn to_string(&self, sc: &mut LocalScope) -> Result<Rc<str>, Value> {
        match self {
            Value::String(s) => Ok(s.clone()),
            Value::Boolean(b) => Ok(b
                .then(|| sc.statics.get_true())
                .unwrap_or_else(|| sc.statics.get_false())),
            Value::Null(_) => Ok(sc.statics.null_str()),
            Value::Undefined(_) => Ok(sc.statics.undefined_str()),
            Value::Number(n) => Ok(n.to_string().into()), // TODO: we can do better
            Value::External(e) => todo!(),                // ??
            Value::Object(_) => {
                let prim_value = self.to_primitive(sc, Some(PreferredType::String))?;
                prim_value.to_string(sc)
            }
            Value::Symbol(s) => throw!(sc, "Cannot convert symbol to a string"),
        }
    }

    fn to_primitive(
        &self,
        sc: &mut LocalScope,
        preferred_type: Option<PreferredType>,
    ) -> Result<Value, Value> {
        if let Value::Object(obj) = self {
            // TODO: Call @@toPrimitive instead of toString once we have symbols
            let exotic_to_prim = self.get_property(sc, "toString".into())?;

            if let Value::Undefined(_) = exotic_to_prim {
                // TODO: d. Return ? OrdinaryToPrimitive(input, preferredType).
                throw!(sc, "Failed to convert to primitive");
            }

            let result = exotic_to_prim.apply(sc, self.clone(), Vec::new())?;

            if let Value::Object(_) = result {
                throw!(sc, "Failed to convert to primitive");
            }

            Ok(result)
        } else {
            Ok(self.clone())
        }
    }

    fn to_length(&self) -> Result<f64, Value> {
        // Let len be ? ToIntegerOrInfinity(argument).
        let len = self.to_integer_or_infinity()?;
        // 2. If len ≤ 0, return +0𝔽.
        if len <= 0.0 {
            return Ok(0.0);
        }

        // Return 𝔽(min(len, 253 - 1)).
        Ok(len.min(MAX_SAFE_INTEGERF))
    }

    fn to_integer_or_infinity(&self) -> Result<f64, Value> {
        // Let number be ? ToNumber(argument).
        let number = self.to_number()?;
        // 2. If number is NaN, +0𝔽, or -0𝔽, return 0.
        if number.is_nan() || number == 0.0 {
            return Ok(0.0);
        }

        // 3. If number is +∞𝔽, return +∞.
        // 4. If number is -∞𝔽, return -∞.
        if number == f64::INFINITY || number == f64::NEG_INFINITY {
            return Ok(number);
        }

        // 5. Let integer be floor(abs(ℝ(number))).
        let integer = number.abs().floor();

        // 6. If number < -0𝔽, set integer to -integer.
        if number < 0.0 {
            Ok(-integer)
        } else {
            Ok(integer)
        }
    }

    fn to_length_u(&self) -> Result<usize, Value> {
        self.to_length().map(|x| x as usize)
    }

    fn length_of_array_like(&self, sc: &mut LocalScope) -> Result<usize, Value> {
        self.get_property(sc, "length".into())?.to_length_u()
    }

    fn to_object(&self, sc: &mut LocalScope) -> Result<Handle<dyn Object>, Value> {
        fn register_dyn<O: Object + 'static, F: Fn(&mut LocalScope) -> O>(
            sc: &mut LocalScope,
            fun: F,
        ) -> Result<Handle<dyn Object>, Value> {
            let obj = fun(sc);
            Ok(sc.register(obj))
        }

        match self {
            Value::Object(o) => Ok(o.clone()),
            Value::Undefined(_) => throw!(sc, "Cannot convert undefined to object"),
            Value::Null(_) => throw!(sc, "Cannot convert null to object"),
            Value::Boolean(b) => register_dyn(sc, |sc| Boolean::new(sc, *b)),
            Value::Symbol(s) => register_dyn(sc, |sc| BoxedSymbol::new(sc, s.clone())),
            Value::Number(n) => register_dyn(sc, |sc| Number::new(sc, *n)),
            Value::String(s) => register_dyn(sc, |sc| BoxedString::new(sc, s.clone())),
            Value::External(e) => Ok(e.clone()), // TODO: is this correct?
        }
    }
}

pub enum PreferredType {
    String,
    Number,
}
