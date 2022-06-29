#![feature(generic_associated_types)]
#![feature(iterator_try_reduce)]
#![feature(box_patterns)]
#![feature(associated_type_defaults)]

use std::collections::HashMap;
use std::sync::Arc;

use crate::expr::{Literal, AST};
use crate::function::FunctionRegistry;
use crate::function::{vectorize_2_arg, Function, FunctionSignature};
use crate::runtime::Runtime;
use crate::types::DataType;
use crate::types::*;
use crate::types::{ArgType, ArrayType, Int16Type};
use crate::values::Column;
use crate::values::{Scalar, Value, ValueRef};

pub mod display;
pub mod expr;
pub mod function;
pub mod runtime;
pub mod type_check;
pub mod types;
pub mod values;

fn main() {
    run_ast(
        &AST::FunctionCall {
            name: "and".to_string(),
            args: vec![
                AST::Literal(Literal::Boolean(true)),
                AST::Literal(Literal::Boolean(false)),
            ],
            params: vec![],
        },
        HashMap::new(),
    );

    run_ast(
        &AST::FunctionCall {
            name: "plus".to_string(),
            args: vec![
                AST::ColumnRef {
                    name: "a".to_string(),
                    data_type: DataType::Nullable(Box::new(DataType::UInt8)),
                },
                AST::Literal(Literal::Int8(-10)),
            ],
            params: vec![],
        },
        [(
            "a".to_string(),
            Arc::new(Column::Nullable {
                column: Arc::new(Column::UInt8(vec![10, 11, 12])),
                nulls: vec![false, true, false],
            }),
        )]
        .into_iter()
        .collect(),
    );

    run_ast(
        &AST::FunctionCall {
            name: "plus".to_string(),
            args: vec![
                AST::ColumnRef {
                    name: "a".to_string(),
                    data_type: DataType::Nullable(Box::new(DataType::UInt8)),
                },
                AST::ColumnRef {
                    name: "b".to_string(),
                    data_type: DataType::Nullable(Box::new(DataType::UInt8)),
                },
            ],
            params: vec![],
        },
        [
            (
                "a".to_string(),
                Arc::new(Column::Nullable {
                    column: Arc::new(Column::UInt8(vec![10, 11, 12])),
                    nulls: vec![false, true, false],
                }),
            ),
            (
                "b".to_string(),
                Arc::new(Column::Nullable {
                    column: Arc::new(Column::UInt8(vec![1, 2, 3])),
                    nulls: vec![false, true, true],
                }),
            ),
        ]
        .into_iter()
        .collect(),
    );

    run_ast(
        &AST::FunctionCall {
            name: "not".to_string(),
            args: vec![AST::ColumnRef {
                name: "a".to_string(),
                data_type: DataType::Nullable(Box::new(DataType::Boolean)),
            }],
            params: vec![],
        },
        [(
            "a".to_string(),
            Arc::new(Column::Nullable {
                column: Arc::new(Column::Boolean(vec![true, false, true])),
                nulls: vec![false, true, false],
            }),
        )]
        .into_iter()
        .collect(),
    );

    run_ast(
        &AST::FunctionCall {
            name: "least".to_string(),
            args: vec![
                AST::Literal(Literal::UInt8(10)),
                AST::Literal(Literal::UInt8(20)),
                AST::Literal(Literal::UInt8(30)),
                AST::Literal(Literal::UInt8(40)),
            ],
            params: vec![],
        },
        HashMap::new(),
    );
}

fn builtin_functions() -> FunctionRegistry {
    let mut registry = FunctionRegistry::default();

    registry.register_2_arg::<BooleanType, BooleanType, BooleanType, _>("and", |lhs, rhs, _| {
        *lhs && *rhs
    });
    registry
        .register_2_arg::<Int16Type, Int16Type, Int16Type, _>("plus", |lhs, rhs, _| *lhs + *rhs);
    registry.register_1_arg::<BooleanType, BooleanType, _>("not", |lhs, _| !*lhs);
    registry.register_function_factory("least", |_, args_len| {
        Some(Arc::new(Function {
            signature: FunctionSignature {
                name: "least",
                args_type: vec![DataType::Int16; args_len],
                return_type: DataType::Int16,
            },
            eval: Box::new(|args, generics| {
                if args.len() == 0 {
                    Value::Scalar(Scalar::Int16(0))
                } else if args.len() == 1 {
                    args[0].clone().to_owned()
                } else {
                    let mut min: Value<Int16Type> = vectorize_2_arg(
                        Int16Type::try_downcast_value(&args[0]).unwrap(),
                        Int16Type::try_downcast_value(&args[1]).unwrap(),
                        generics,
                        |lhs, rhs, _| *lhs.min(rhs),
                    );
                    for arg in &args[2..] {
                        min = vectorize_2_arg(
                            min.as_ref(),
                            Int16Type::try_downcast_value(arg).unwrap(),
                            generics,
                            |lhs, rhs, _| *lhs.min(rhs),
                        );
                    }
                    Int16Type::upcast_value(min)
                }
            }),
        }))
    });
    registry.register_function_factory("array", |_, args_len| {
        Some(Arc::new(Function {
            signature: FunctionSignature {
                name: "array",
                args_type: vec![DataType::Generic(0); args_len],
                return_type: DataType::Generic(0),
            },
            eval: Box::new(|args, generics| match generics[&0] {
                DataType::Boolean => create_array_scalar::<BooleanType>(args),
                DataType::Int16 => create_array_scalar::<Int16Type>(args),
                _ => unimplemented!(),
            }),
        }))
    });
    registry.register_2_arg::<ArrayType<GenericType<0>>, Int16Type, GenericType<0>, _>(
        "get",
        |array, idx, generics| array.index(*idx as usize),
    );

    registry
}

fn create_array_scalar<T: ArgType + ColumnBuilder>(args: &[ValueRef<AnyType>]) -> Value<AnyType> {
    Value::Scalar(Scalar::Array(T::upcast_column(T::column_from_iter(
        args.iter().cloned().map(|arg| match arg {
            ValueRef::Scalar(scalar) => T::to_owned_scalar(T::try_downcast_scalar(scalar).unwrap()),
            ValueRef::Column(_) => unimplemented!(),
            _ => unreachable!(),
        }),
    ))))
}

pub fn run_ast(ast: &AST, columns: HashMap<String, Arc<Column>>) {
    println!("ast: {ast}");
    let fn_registry = builtin_functions();
    let (expr, ty) = type_check::check(&ast, &fn_registry).unwrap();
    println!("expr: {expr}");
    println!("ty: {ty}");
    let runtime = Runtime { columns };
    let result = runtime.run(&expr);
    println!("result: {result}\n");
}
