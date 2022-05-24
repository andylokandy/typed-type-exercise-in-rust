use std::sync::Arc;

use educe::Educe;

use crate::{
    function_nullable::{erase_function_generic_1_nullable, erase_function_generic_2_nullable},
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
}

#[derive(Default)]
pub struct FunctionRegistry(Vec<Arc<Function>>);

impl FunctionRegistry {
    pub fn with_builtins(func: Vec<Function>) -> FunctionRegistry {
        let fns = func.into_iter().map(Arc::new).collect();
        FunctionRegistry(fns)
    }

    pub fn register_1_arg<I1: Type, O: Type, F>(&mut self, name: &'static str, func: F)
    where
        F: Fn(ValueRef<I1>) -> Value<O> + 'static + Clone,
    {
        self.0.push(Arc::new(Function {
            signature: FunctionSignature {
                name,
                args_type: vec![I1::data_type()],
                return_type: O::data_type(),
            },
            eval: Box::new(erase_function_generic_1(func.clone())),
        }));

        let has_nullable = &[I1::data_type(), O::data_type()].iter().any(|t| match t {
            DataType::Nullable(_) => true,
            _ => false,
        });

        if !has_nullable {
            self.0.push(Arc::new(Function {
                signature: FunctionSignature {
                    name,
                    args_type: vec![<NullableType<I1>>::data_type()],
                    return_type: <NullableType<O>>::data_type(),
                },
                eval: Box::new(erase_function_generic_1_nullable(func)),
            }));
        }
    }

    pub fn register_2_arg<I1: Type, I2: Type, O: Type, F>(&mut self, name: &'static str, func: F)
    where
        F: for<'a> Fn(ValueRef<'a, I1>, ValueRef<'a, I2>) -> Value<O> + Sized + 'static + Clone,
    {
        self.0.push(Arc::new(Function {
            signature: FunctionSignature {
                name,
                args_type: vec![I1::data_type(), I2::data_type()],
                return_type: O::data_type(),
            },
            eval: Box::new(erase_function_generic_2(func.clone())),
        }));

        let has_nullable = &[I1::data_type(), I2::data_type(), O::data_type()]
            .iter()
            .any(|t| match t {
                DataType::Nullable(_) => true,
                _ => false,
            });

        if !has_nullable {
            self.0.push(Arc::new(Function {
                signature: FunctionSignature {
                    name,
                    args_type: vec![
                        <NullableType<I1>>::data_type(),
                        <NullableType<I2>>::data_type(),
                    ],
                    return_type: <NullableType<O>>::data_type(),
                },
                eval: Box::new(erase_function_generic_2_nullable(func)),
            }));
        }
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

pub fn vectorize_unary<'a, I1: Type, O: Type>(
    lhs: ValueRef<'a, I1>,
    func: impl Fn(I1::ScalarRef<'a>) -> O::Scalar,
) -> Value<O> {
    match lhs {
        ValueRef::Scalar(lhs) => Value::Scalar(func(lhs)),

        ValueRef::Column(lhs) => {
            let iter = I1::iter_column(lhs).map(|lhs| func(lhs));
            let col = O::column_from_iter(iter);
            Value::Column(col)
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
            let iter = I2::iter_column(rhs).map(|rhs| func(lhs.clone(), rhs));
            let col = O::column_from_iter(iter);
            Value::Column(col)
        }
        (ValueRef::Column(lhs), ValueRef::Scalar(rhs)) => {
            let iter = I1::iter_column(lhs).map(|lhs| func(lhs, rhs.clone()));
            let col = O::column_from_iter(iter);
            Value::Column(col)
        }
        (ValueRef::Column(lhs), ValueRef::Column(rhs)) => {
            let iter = I1::iter_column(lhs)
                .zip(I2::iter_column(rhs))
                .map(|(lhs, rhs)| func(lhs, rhs));
            let col = O::column_from_iter(iter);
            Value::Column(col)
        }
    }
}
