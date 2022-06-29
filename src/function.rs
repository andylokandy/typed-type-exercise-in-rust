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

/// `FunctionID` is a unique identifier for a function. It's used to construct
/// the exactly same function from the remote execution nodes.
#[derive(Debug, Clone)]
pub enum FunctionID {
    Builtin(usize),
    Factory {
        name: String,
        params: Vec<usize>,
        args_len: usize,
    },
}

#[derive(Educe)]
#[educe(Debug)]
pub struct Function {
    pub signature: FunctionSignature,
    #[educe(Debug(ignore))]
    pub eval: Box<dyn Fn(&[ValueRef<AnyType>], &GenericMap) -> Value<AnyType>>,
}

impl Function {
    pub fn new_1_arg<I1: ArgType, O: ArgType, F>(name: &'static str, func: F) -> Function
    where
        F: Fn(ValueRef<I1>, &GenericMap) -> Value<O> + 'static,
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
    /// The first argument is the const parameters and the second argument is the number of arguments.
    pub factories:
        HashMap<&'static str, Box<dyn Fn(&[usize], usize) -> Option<Arc<Function>> + 'static>>,
}

impl FunctionRegistry {
    pub fn search_candidates(
        &self,
        name: &str,
        params: &[usize],
        args_len: usize,
    ) -> Vec<(FunctionID, Arc<Function>)> {
        if params.is_empty() {
            let builtin_funcs = self
                .funcs
                .iter()
                .enumerate()
                .filter_map(|(i, func)| {
                    if func.signature.name == name && func.signature.args_type.len() == args_len {
                        Some((FunctionID::Builtin(i), func.clone()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            if !builtin_funcs.is_empty() {
                return builtin_funcs;
            }
        }

        self.factories
            .get(name)
            .and_then(|factory| factory(params, args_len))
            .map(|func| {
                vec![(
                    FunctionID::Factory {
                        name: name.to_string(),
                        params: params.to_vec(),
                        args_len,
                    },
                    func,
                )]
            })
            .unwrap_or(Vec::new())
    }

    pub fn register_1_arg<I1: ArgType + ColumnViewer, O: ArgType + ColumnBuilder, F>(
        &mut self,
        name: &'static str,
        func: F,
    ) where
        I1::Scalar: Default,
        O::Scalar: Default,
        F: for<'a> Fn(I1::ScalarRef<'a>, &GenericMap) -> O::Scalar + 'static + Clone + Copy,
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

        self.register_1_arg_core::<I1, O, _>(name, move |val, generics| {
            vectorize_1_arg(val, generics, func)
        });

        self.register_1_arg_core::<NullableType<I1>, NullableType<O>, _>(
            name,
            move |val, generics| vectorize_passthrough_nullable_1_arg(val, generics, func),
        );
    }

    pub fn register_1_arg_core<I1: ArgType, O: ArgType, F>(&mut self, name: &'static str, func: F)
    where
        F: Fn(ValueRef<I1>, &GenericMap) -> Value<O> + 'static + Clone + Copy,
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

    pub fn register_2_arg<
        I1: ArgType + ColumnViewer,
        I2: ArgType + ColumnViewer,
        O: ArgType + ColumnBuilder,
        F,
    >(
        &mut self,
        name: &'static str,
        func: F,
    ) where
        I1::Scalar: Default,
        I2::Scalar: Default,
        O::Scalar: Default,
        F: for<'a> Fn(I1::ScalarRef<'a>, I2::ScalarRef<'a>, &GenericMap) -> O::Scalar
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

        self.register_2_arg_core::<I1, I2, O, _>(name, move |lhs, rhs, generics| {
            vectorize_2_arg(lhs, rhs, generics, func.clone())
        });

        self.register_2_arg_core::<NullableType<I1>, NullableType<I2>, NullableType<O>, _>(
            name,
            move |lhs, rhs, generics| {
                vectorize_passthrough_nullable_2_arg(lhs, rhs, generics, func)
            },
        );
    }

    pub fn register_2_arg_core<I1: ArgType, I2: ArgType, O: ArgType, F>(
        &mut self,
        name: &'static str,
        func: F,
    ) where
        F: for<'a> Fn(ValueRef<'a, I1>, ValueRef<'a, I2>, &GenericMap) -> Value<O>
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
        factory: impl Fn(&[usize], usize) -> Option<Arc<Function>> + 'static,
    ) {
        self.factories.insert(name, Box::new(factory));
    }
}

fn erase_function_generic_1_arg<I1: ArgType, O: ArgType>(
    func: impl for<'a> Fn(ValueRef<'a, I1>, &GenericMap) -> Value<O>,
) -> impl Fn(&[ValueRef<AnyType>], &GenericMap) -> Value<AnyType> {
    move |args, generics| {
        let arg1 = match &args[0] {
            ValueRef::Scalar(scalar) => ValueRef::Scalar(I1::try_downcast_scalar(scalar).unwrap()),
            ValueRef::Column(col) => ValueRef::Column(I1::try_downcast_column(&col).unwrap()),
        };

        let result = func(arg1, generics);

        match result {
            Value::Scalar(scalar) => Value::Scalar(O::upcast_scalar(scalar)),
            Value::Column(col) => Value::Column(O::upcast_column(col)),
        }
    }
}

fn erase_function_generic_2_arg<I1: ArgType, I2: ArgType, O: ArgType>(
    func: impl for<'a> Fn(ValueRef<'a, I1>, ValueRef<'a, I2>, &GenericMap) -> Value<O>,
) -> impl Fn(&[ValueRef<AnyType>], &GenericMap) -> Value<AnyType> {
    move |args, generics| {
        let arg1 = match &args[0] {
            ValueRef::Scalar(scalar) => ValueRef::Scalar(I1::try_downcast_scalar(scalar).unwrap()),
            ValueRef::Column(col) => ValueRef::Column(I1::try_downcast_column(&col).unwrap()),
        };
        let arg2 = match &args[1] {
            ValueRef::Scalar(scalar) => ValueRef::Scalar(I2::try_downcast_scalar(scalar).unwrap()),
            ValueRef::Column(col) => ValueRef::Column(I2::try_downcast_column(&col).unwrap()),
        };

        let result = func(arg1, arg2, generics);

        match result {
            Value::Scalar(scalar) => Value::Scalar(O::upcast_scalar(scalar)),
            Value::Column(col) => Value::Column(O::upcast_column(col)),
        }
    }
}

pub fn vectorize_1_arg<'a, I1: ColumnViewer, O: ColumnBuilder>(
    val: ValueRef<'a, I1>,
    generics: &GenericMap,
    func: impl Fn(I1::ScalarRef<'a>, &GenericMap) -> O::Scalar,
) -> Value<O> {
    match val {
        ValueRef::Scalar(val) => Value::Scalar(func(val, generics)),
        ValueRef::Column(col) => {
            let iter =
                I1::iter_column(col).map(|val| func(I1::scalar_borrow_to_ref(&val), generics));
            let col = O::column_from_iter(iter);
            Value::Column(col)
        }
    }
}

pub fn vectorize_2_arg<'a, 'b, I1: ColumnViewer, I2: ColumnViewer, O: ColumnBuilder>(
    lhs: ValueRef<'a, I1>,
    rhs: ValueRef<'b, I2>,
    generics: &GenericMap,
    func: impl Fn(I1::ScalarRef<'a>, I2::ScalarRef<'b>, &GenericMap) -> O::Scalar,
) -> Value<O> {
    match (lhs, rhs) {
        (ValueRef::Scalar(lhs), ValueRef::Scalar(rhs)) => Value::Scalar(func(lhs, rhs, generics)),
        (ValueRef::Scalar(lhs), ValueRef::Column(rhs)) => {
            let iter = I2::iter_column(rhs)
                .map(|rhs| func(lhs.clone(), I2::scalar_borrow_to_ref(&rhs), generics));
            let col = O::column_from_iter(iter);
            Value::Column(col)
        }
        (ValueRef::Column(lhs), ValueRef::Scalar(rhs)) => {
            let iter = I1::iter_column(lhs)
                .map(|lhs| func(I1::scalar_borrow_to_ref(&lhs), rhs.clone(), generics));
            let col = O::column_from_iter(iter);
            Value::Column(col)
        }
        (ValueRef::Column(lhs), ValueRef::Column(rhs)) => {
            let iter = I1::iter_column(lhs)
                .zip(I2::iter_column(rhs))
                .map(|(lhs, rhs)| {
                    func(
                        I1::scalar_borrow_to_ref(&lhs),
                        I2::scalar_borrow_to_ref(&rhs),
                        generics,
                    )
                });
            let col = O::column_from_iter(iter);
            Value::Column(col)
        }
    }
}

pub fn vectorize_passthrough_nullable_1_arg<'a, I1: ColumnViewer, O: ColumnBuilder>(
    val: ValueRef<'a, NullableType<I1>>,
    generics: &GenericMap,
    func: impl Fn(I1::ScalarRef<'a>, &GenericMap) -> O::Scalar,
) -> Value<NullableType<O>>
where
    I1::Scalar: Default,
    O::Scalar: Default,
{
    match val {
        ValueRef::Scalar(None) => Value::Scalar(None),
        ValueRef::Scalar(Some(val)) => Value::Scalar(Some(func(val, generics))),
        ValueRef::Column((col, nulls)) => {
            let iter =
                I1::iter_column(col).map(|val| func(I1::scalar_borrow_to_ref(&val), generics));
            let col = O::column_from_iter(iter);
            Value::Column((col, nulls.to_vec()))
        }
    }
}

pub fn vectorize_passthrough_nullable_2_arg<
    'a,
    'b,
    I1: ColumnViewer,
    I2: ColumnViewer,
    O: ColumnBuilder,
>(
    lhs: ValueRef<'a, NullableType<I1>>,
    rhs: ValueRef<'b, NullableType<I2>>,
    generics: &GenericMap,
    func: impl Fn(I1::ScalarRef<'a>, I2::ScalarRef<'b>, &GenericMap) -> O::Scalar,
) -> Value<NullableType<O>>
where
    I1::Scalar: Default,
    I2::Scalar: Default,
    O::Scalar: Default,
{
    match (lhs, rhs) {
        (ValueRef::Scalar(None), _) | (_, ValueRef::Scalar(None)) => Value::Scalar(None),
        (ValueRef::Scalar(Some(lhs)), ValueRef::Scalar(Some(rhs))) => {
            Value::Scalar(Some(func(lhs, rhs, generics)))
        }
        (ValueRef::Scalar(Some(lhs)), ValueRef::Column((rhs, rhs_nulls))) => {
            let iter = I2::iter_column(rhs)
                .map(|rhs| func(lhs.clone(), I2::scalar_borrow_to_ref(&rhs), generics));
            let col = O::column_from_iter(iter);
            Value::Column((col, rhs_nulls.to_vec()))
        }
        (ValueRef::Column((lhs, lhs_nulls)), ValueRef::Scalar(Some(rhs))) => {
            let iter = I1::iter_column(lhs)
                .map(|lhs| func(I1::scalar_borrow_to_ref(&lhs), rhs.clone(), generics));
            let col = O::column_from_iter(iter);
            Value::Column((col, lhs_nulls.to_vec()))
        }
        (ValueRef::Column((lhs, lhs_nulls)), ValueRef::Column((rhs, rhs_nulls))) => {
            let iter = I1::iter_column(lhs)
                .zip(I2::iter_column(rhs))
                .map(|(lhs, rhs)| {
                    func(
                        I1::scalar_borrow_to_ref(&lhs),
                        I2::scalar_borrow_to_ref(&rhs),
                        generics,
                    )
                });
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
