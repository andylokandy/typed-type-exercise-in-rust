use std::sync::Arc;

use educe::Educe;

use crate::{
    runtime::{Value, ValueRef},
    types::*,
};
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: &'static str,
    pub args_type: Vec<DataType>,
    pub return_type: DataType,
}

#[derive(Educe)]
#[educe(Debug)]
pub struct Function {
    pub signature: FunctionSignature,
    #[educe(Debug(ignore))]
    pub eval: Box<dyn Fn(&[ValueRef<AnyType>]) -> Value<AnyType>>,
}

impl Function {
    pub fn new_1_arg<I1: Type, O: Type, F>(name: &'static str, func: F) -> Function
    where
        F: Fn(ValueRef<I1>) -> Value<O> + 'static,
    {
        Function {
            signature: FunctionSignature {
                name,
                args_type: vec![I1::data_type()],
                return_type: O::data_type(),
            },
            eval: Box::new(erase_function_generic_1(func)),
        }
    }

    pub fn new_2_arg<I1: Type, I2: Type, O: Type, F>(name: &'static str, func: F) -> Function
    where
        F: for<'a> Fn(ValueRef<'a, I1>, ValueRef<'a, I2>) -> Value<O> + Sized + 'static,
    {
        Function {
            signature: FunctionSignature {
                name,
                args_type: vec![I1::data_type(), I2::data_type()],
                return_type: O::data_type(),
            },
            eval: Box::new(erase_function_generic_2(func)),
        }
    }
}

pub struct FunctionRegistry(Vec<Arc<Function>>);

impl FunctionRegistry {
    pub fn with_builtins(func: Vec<Function>) -> FunctionRegistry {
        let fns = func.into_iter().map(Arc::new).collect();
        FunctionRegistry(fns)
    }

    pub fn search(&self, name: &str, args_len: usize) -> Vec<Arc<Function>> {
        self.0
            .iter()
            .filter(|func| {
                func.signature.name == name && func.signature.args_type.len() == args_len
            })
            .map(Arc::clone)
            .collect()
    }
}

fn erase_function_generic_1<I1: Type, O: Type>(
    func: impl for<'a> Fn(ValueRef<'a, I1>) -> Value<O>,
) -> impl Fn(&[ValueRef<AnyType>]) -> Value<AnyType> {
    move |args| {
        let arg1 = match &args[0] {
            ValueRef::Scalar(scalar) => ValueRef::Scalar(I1::try_downcast_scalar(scalar).unwrap()),
            ValueRef::Column(col) => ValueRef::Column(I1::try_downcast_column(col).unwrap()),
        };

        let result = func(arg1);

        match result {
            Value::Scalar(scalar) => Value::Scalar(O::upcast_scalar(scalar)),
            Value::Column(col) => Value::Column(O::upcast_column(col)),
        }
    }
}

fn erase_function_generic_2<I1: Type, I2: Type, O: Type>(
    func: impl for<'a> Fn(ValueRef<'a, I1>, ValueRef<'a, I2>) -> Value<O>,
) -> impl Fn(&[ValueRef<AnyType>]) -> Value<AnyType> {
    move |args| {
        let arg1 = match &args[0] {
            ValueRef::Scalar(scalar) => ValueRef::Scalar(I1::try_downcast_scalar(scalar).unwrap()),
            ValueRef::Column(col) => ValueRef::Column(I1::try_downcast_column(&col).unwrap()),
        };
        let arg2 = match &args[1] {
            ValueRef::Scalar(scalar) => ValueRef::Scalar(I2::try_downcast_scalar(scalar).unwrap()),
            ValueRef::Column(col) => ValueRef::Column(I2::try_downcast_column(&col).unwrap()),
        };

        let result = func(arg1, arg2);

        match result {
            Value::Scalar(scalar) => Value::Scalar(O::upcast_scalar(scalar)),
            Value::Column(col) => Value::Column(O::upcast_column(col)),
        }
    }
}

pub fn vectorize_binary<'a, I1: Type, I2: Type, O: Type>(
    lhs: ValueRef<'a, I1>,
    rhs: ValueRef<'a, I2>,
    func: impl Fn(I1::ScalarRef<'a>, I2::ScalarRef<'a>) -> O::Scalar,
) -> Value<O> {
    match (lhs, rhs) {
        (ValueRef::Scalar(lhs), ValueRef::Scalar(rhs)) => Value::Scalar(func(lhs, rhs)),
        (ValueRef::Scalar(lhs), ValueRef::Column(rhs)) => {
            let mut col = O::empty_column(0);
            for rhs in I2::iter_column(rhs) {
                col = O::push_column(col, func(lhs.clone(), rhs));
            }
            Value::Column(col)
        }
        (ValueRef::Column(lhs), ValueRef::Scalar(rhs)) => {
            let mut col = O::empty_column(0);
            for lhs in I1::iter_column(lhs) {
                col = O::push_column(col, func(lhs, rhs.clone()));
            }
            Value::Column(col)
        }
        (ValueRef::Column(lhs), ValueRef::Column(rhs)) => {
            let mut col = O::empty_column(0);
            for (lhs, rhs) in I1::iter_column(lhs).zip(I2::iter_column(rhs)) {
                col = O::push_column(col, func(lhs, rhs));
            }
            Value::Column(col)
        }
    }
}

pub fn vectorize_binary_passthrough_nullable<'a, I1: Type, I2: Type, O: Type>(
    lhs: ValueRef<'a, NullableType<I1>>,
    rhs: ValueRef<'a, NullableType<I2>>,
    func: impl Fn(I1::ScalarRef<'a>, I2::ScalarRef<'a>) -> O::Scalar,
) -> Value<NullableType<O>> {
    match (lhs, rhs) {
        (ValueRef::Scalar(None), _) | (_, ValueRef::Scalar(None)) => Value::Scalar(None),
        (ValueRef::Scalar(Some(lhs)), ValueRef::Scalar(Some(rhs))) => {
            Value::Scalar(Some(func(lhs, rhs)))
        }
        (ValueRef::Scalar(Some(lhs)), ValueRef::Column((rhs, rhs_nulls))) => {
            let mut col = O::empty_column(0);
            for rhs in I2::iter_column(rhs) {
                col = O::push_column(col, func(lhs.clone(), rhs));
            }
            Value::Column((col, rhs_nulls.to_vec()))
        }
        (ValueRef::Column((lhs, lhs_nulls)), ValueRef::Scalar(Some(rhs))) => {
            let mut col = O::empty_column(0);
            for lhs in I1::iter_column(lhs) {
                col = O::push_column(col, func(lhs, rhs.clone()));
            }
            Value::Column((col, lhs_nulls.to_vec()))
        }
        (ValueRef::Column((lhs, lhs_nulls)), ValueRef::Column((rhs, rhs_nulls))) => {
            let mut col = O::empty_column(0);
            for (lhs, rhs) in I1::iter_column(lhs).zip(I2::iter_column(rhs)) {
                col = O::push_column(col, func(lhs, rhs));
            }
            let nulls = lhs_nulls
                .iter()
                .zip(rhs_nulls)
                .map(|(lhs, rhs)| *lhs || *rhs)
                .collect();
            Value::Column((col, nulls))
        }
    }
}
