#![feature(generic_associated_types)]
#![feature(iterator_try_reduce)]
#![feature(box_patterns)]
#![feature(associated_type_defaults)]

#![allow(clippy::len_without_is_empty)]
#![allow(clippy::needless_lifetimes)]

use std::collections::HashMap;
use std::sync::Arc;

use property::{FunctionProperty, ValueProperty};

use crate::expr::{Literal, AST};
use crate::function::FunctionRegistry;
use crate::function::{vectorize_2_arg, Function, FunctionSignature};
use crate::runtime::Runtime;
use crate::types::DataType;
use crate::types::*;
use crate::types::{ArgType, ArrayType, Int16Type};
use crate::values::Column;
use crate::values::{Scalar, Value};

pub mod display;
pub mod expr;
pub mod function;
pub mod property;
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
                    property: ValueProperty::default().not_null(false),
                },
                AST::Literal(Literal::Int8(-10)),
            ],
            params: vec![],
        },
        [(
            "a".to_string(),
            Column::Nullable {
                column: Box::new(Column::UInt8(vec![10, 11, 12])),
                nulls: vec![false, true, false],
            },
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
                    property: ValueProperty::default().not_null(false),
                },
                AST::ColumnRef {
                    name: "b".to_string(),
                    data_type: DataType::Nullable(Box::new(DataType::UInt8)),
                    property: ValueProperty::default().not_null(false),
                },
            ],
            params: vec![],
        },
        [
            (
                "a".to_string(),
                Column::Nullable {
                    column: Box::new(Column::UInt8(vec![10, 11, 12])),
                    nulls: vec![false, true, false],
                },
            ),
            (
                "b".to_string(),
                Column::Nullable {
                    column: Box::new(Column::UInt8(vec![1, 2, 3])),
                    nulls: vec![false, true, true],
                },
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
                property: ValueProperty::default().not_null(false),
            }],
            params: vec![],
        },
        [(
            "a".to_string(),
            Column::Nullable {
                column: Box::new(Column::Boolean(vec![true, false, true])),
                nulls: vec![false, true, false],
            },
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

    run_ast(
        &AST::FunctionCall {
            name: "get".to_string(),
            args: vec![
                AST::ColumnRef {
                    name: "array".to_string(),
                    data_type: DataType::Array(Box::new(DataType::Int16)),
                    property: ValueProperty::default().not_null(true),
                },
                AST::ColumnRef {
                    name: "idx".to_string(),
                    data_type: DataType::UInt8,
                    property: ValueProperty::default().not_null(true),
                },
            ],
            params: vec![],
        },
        [
            (
                "array".to_string(),
                Column::Array {
                    array: Box::new(Column::Int16((0..100).collect())),
                    offsets: vec![0..20, 20..40, 40..60, 60..80, 80..100],
                },
            ),
            ("idx".to_string(), Column::UInt8(vec![0, 1, 2, 3, 4])),
        ]
        .into_iter()
        .collect(),
    );

    run_ast(
        &AST::FunctionCall {
            name: "get".to_string(),
            args: vec![
                AST::ColumnRef {
                    name: "array".to_string(),
                    data_type: DataType::Array(Box::new(DataType::Array(Box::new(
                        DataType::Int16,
                    )))),
                    property: ValueProperty::default().not_null(true),
                },
                AST::ColumnRef {
                    name: "idx".to_string(),
                    data_type: DataType::UInt8,
                    property: ValueProperty::default().not_null(true),
                },
            ],
            params: vec![],
        },
        [
            (
                "array".to_string(),
                Column::Array {
                    array: Box::new(Column::Array {
                        array: Box::new(Column::Int16((0..100).collect())),
                        offsets: vec![
                            0..5,
                            5..10,
                            10..15,
                            15..20,
                            20..25,
                            25..30,
                            30..35,
                            35..40,
                            40..45,
                            45..50,
                            50..55,
                            55..60,
                            60..65,
                            65..70,
                            70..75,
                            75..80,
                            80..85,
                            85..90,
                            90..95,
                            95..100,
                        ],
                    }),
                    offsets: vec![0..4, 4..8, 8..12, 12..16, 16..20],
                },
            ),
            ("idx".to_string(), Column::UInt8(vec![0, 1, 2])),
        ]
        .into_iter()
        .collect(),
    );
}

fn builtin_functions() -> FunctionRegistry {
    let mut registry = FunctionRegistry::default();

    registry.register_2_arg::<BooleanType, BooleanType, BooleanType, _>(
        "and",
        FunctionProperty::default(),
        |lhs, rhs| lhs && rhs,
    );

    registry.register_2_arg::<Int16Type, Int16Type, Int16Type, _>(
        "plus",
        FunctionProperty::default(),
        |lhs, rhs| lhs + rhs,
    );

    registry.register_1_arg::<BooleanType, BooleanType, _>(
        "not",
        FunctionProperty::default(),
        |val| !val,
    );

    registry.register_function_factory("least", |_, args_len| {
        Some(Arc::new(Function {
            signature: FunctionSignature {
                name: "least",
                args_type: vec![DataType::Int16; args_len],
                return_type: DataType::Int16,
                property: FunctionProperty::default().preserve_not_null(true),
            },
            eval: Box::new(|args, generics| {
                if args.is_empty() {
                    Value::Scalar(Scalar::Int16(0))
                } else if args.len() == 1 {
                    args[0].clone().to_owned()
                } else {
                    let mut min: Value<Int16Type> = vectorize_2_arg(
                        Int16Type::try_downcast_value(&args[0]).unwrap(),
                        Int16Type::try_downcast_value(&args[1]).unwrap(),
                        generics,
                        |lhs, rhs| lhs.min(rhs),
                    );
                    for arg in &args[2..] {
                        min = vectorize_2_arg(
                            min.as_ref(),
                            Int16Type::try_downcast_value(arg).unwrap(),
                            generics,
                            |lhs, rhs| lhs.min(rhs),
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
                property: FunctionProperty::default().preserve_not_null(true),
            },
            eval: Box::new(|_args, _generics| todo!()),
        }))
    });
    registry.register_2_arg::<ArrayType<GenericType<0>>, Int16Type, GenericType<0>, _>(
        "get",
        FunctionProperty::default(),
        |array, idx| array.index(idx as usize).to_owned(),
    );

    registry
}

pub fn run_ast(ast: &AST, columns: HashMap<String, Column>) {
    println!("ast: {ast}");
    let fn_registry = builtin_functions();
    let (expr, ty, prop) = type_check::check(ast, &fn_registry).unwrap();
    println!("expr: {expr}");
    println!("type: {ty}");
    println!("property: {prop}");
    let runtime = Runtime { columns };
    let result = runtime.run(&expr);
    println!("result: {result}\n");
}
