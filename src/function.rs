use std::{collections::HashMap, sync::Arc};

use educe::Educe;

use crate::{
    types::*,
    values::{Value, ValueRef},
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
            eval: Box::new(erase_function_generic_1_arg(func)),
        }
    }
}

#[derive(Default)]
pub struct FunctionRegistry {
    pub funcs: Vec<Arc<Function>>,
    /// A function to build function depending on the const parameters and the type of arguments (before coersion).
    ///
    /// The first argument is the const parameters and the second argument is the type of arguments.
    pub factories: HashMap<
        &'static str,
        Box<dyn Fn(&[usize], &[&DataType]) -> Option<Arc<Function>> + 'static>,
    >,
}

impl FunctionRegistry {
    pub fn register_1_arg<I1: Type, O: Type, F>(&mut self, name: &'static str, func: F)
    where
        F: for<'a> Fn(I1::ScalarRef<'a>) -> O::Scalar + 'static + Clone + Copy,
    {
        let has_nullable = &[I1::data_type(), O::data_type()].iter().any(|t| match t {
            DataType::Nullable(_) => true,
            _ => false,
        });

        assert!(
            !has_nullable,
            "Function {} has nullable argument or output, please use register_1_arg_core instead",
            name
        );

        self.register_1_arg_core::<I1, O, _>(name, move |val| vectorize_1_arg(val, func));

        self.register_1_arg_core::<NullableType<I1>, NullableType<O>, _>(name, move |val| {
            vectorize_passthrough_nullable_1_arg(val, func)
        });
    }

    pub fn register_1_arg_core<I1: Type, O: Type, F>(&mut self, name: &'static str, func: F)
    where
        F: Fn(ValueRef<I1>) -> Value<O> + 'static + Clone + Copy,
    {
        self.funcs.push(Arc::new(Function {
            signature: FunctionSignature {
                name,
                args_type: vec![I1::data_type()],
                return_type: O::data_type(),
            },
            eval: Box::new(erase_function_generic_1_arg(func)),
        }));
    }

    pub fn register_2_arg<I1: Type, I2: Type, O: Type, F>(&mut self, name: &'static str, func: F)
    where
        F: for<'a> Fn(I1::ScalarRef<'a>, I2::ScalarRef<'a>) -> O::Scalar
            + Sized
            + 'static
            + Clone
            + Copy,
    {
        let has_nullable = &[I1::data_type(), I2::data_type(), O::data_type()]
            .iter()
            .any(|t| match t {
                DataType::Nullable(_) => true,
                _ => false,
            });

        assert!(
            !has_nullable,
            "Function {} has nullable argument or output, please use register_2_arg_core instead",
            name
        );

        self.register_2_arg_core::<I1, I2, O, _>(name, move |lhs, rhs| {
            vectorize_2_arg(lhs, rhs, func.clone())
        });

        self.register_2_arg_core::<NullableType<I1>, NullableType<I2>, NullableType<O>, _>(
            name,
            move |lhs, rhs| vectorize_passthrough_nullable_2_arg(lhs, rhs, func),
        );
    }

    pub fn register_2_arg_core<I1: Type, I2: Type, O: Type, F>(
        &mut self,
        name: &'static str,
        func: F,
    ) where
        F: for<'a> Fn(ValueRef<'a, I1>, ValueRef<'a, I2>) -> Value<O>
            + Sized
            + 'static
            + Clone
            + Copy,
    {
        self.funcs.push(Arc::new(Function {
            signature: FunctionSignature {
                name,
                args_type: vec![I1::data_type(), I2::data_type()],
                return_type: O::data_type(),
            },
            eval: Box::new(erase_function_generic_2_arg(func)),
        }));
    }

    pub fn register_function_factory(
        &mut self,
        name: &'static str,
        factory: impl Fn(&[usize], &[&DataType]) -> Option<Arc<Function>> + 'static,
    ) {
        self.factories.insert(name, Box::new(factory));
    }
}

fn erase_function_generic_1_arg<I1: Type, O: Type>(
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

fn erase_function_generic_2_arg<I1: Type, I2: Type, O: Type>(
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

pub fn vectorize_1_arg<'a, I1: Type, O: Type>(
    val: ValueRef<'a, I1>,
    func: impl Fn(I1::ScalarRef<'a>) -> O::Scalar,
) -> Value<O> {
    match val {
        ValueRef::Scalar(val) => Value::Scalar(func(val)),
        ValueRef::Column(col) => {
            let iter = I1::iter_column(col).map(|val| func(val));
            let col = O::column_from_iter(iter);
            Value::Column(col)
        }
    }
}

pub fn vectorize_2_arg<'a, 'b, I1: Type, I2: Type, O: Type>(
    lhs: ValueRef<'a, I1>,
    rhs: ValueRef<'b, I2>,
    func: impl Fn(I1::ScalarRef<'a>, I2::ScalarRef<'b>) -> O::Scalar,
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

pub fn vectorize_passthrough_nullable_1_arg<'a, I1: Type, O: Type>(
    val: ValueRef<'a, NullableType<I1>>,
    func: impl Fn(I1::ScalarRef<'a>) -> O::Scalar,
) -> Value<NullableType<O>> {
    match val {
        ValueRef::Scalar(None) => Value::Scalar(None),
        ValueRef::Scalar(Some(val)) => Value::Scalar(Some(func(val))),
        ValueRef::Column((col, nulls)) => {
            let iter = I1::iter_column(col).map(|val| func(val));
            let col = O::column_from_iter(iter);
            Value::Column((col, nulls.to_vec()))
        }
    }
}

pub fn vectorize_passthrough_nullable_2_arg<'a, 'b, I1: Type, I2: Type, O: Type>(
    lhs: ValueRef<'a, NullableType<I1>>,
    rhs: ValueRef<'b, NullableType<I2>>,
    func: impl Fn(I1::ScalarRef<'a>, I2::ScalarRef<'b>) -> O::Scalar,
) -> Value<NullableType<O>> {
    match (lhs, rhs) {
        (ValueRef::Scalar(None), _) | (_, ValueRef::Scalar(None)) => Value::Scalar(None),
        (ValueRef::Scalar(Some(lhs)), ValueRef::Scalar(Some(rhs))) => {
            Value::Scalar(Some(func(lhs, rhs)))
        }
        (ValueRef::Scalar(Some(lhs)), ValueRef::Column((rhs, rhs_nulls))) => {
            let iter = I2::iter_column(rhs).map(|rhs| func(lhs.clone(), rhs));
            let col = O::column_from_iter(iter);
            Value::Column((col, rhs_nulls.to_vec()))
        }
        (ValueRef::Column((lhs, lhs_nulls)), ValueRef::Scalar(Some(rhs))) => {
            let iter = I1::iter_column(lhs).map(|lhs| func(lhs, rhs.clone()));
            let col = O::column_from_iter(iter);
            Value::Column((col, lhs_nulls.to_vec()))
        }
        (ValueRef::Column((lhs, lhs_nulls)), ValueRef::Column((rhs, rhs_nulls))) => {
            let iter = I1::iter_column(lhs)
                .zip(I2::iter_column(rhs))
                .map(|(lhs, rhs)| func(lhs, rhs));
            let col = O::column_from_iter(iter);

            let nulls = lhs_nulls
                .iter()
                .zip(rhs_nulls)
                .map(|(lhs, rhs)| *lhs || *rhs)
                .collect();
            Value::Column((col, nulls))
        }
    }
}
