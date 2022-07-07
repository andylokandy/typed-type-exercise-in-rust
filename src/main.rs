#![feature(generic_associated_types)]
#![feature(iterator_try_reduce)]
#![feature(box_patterns)]
#![feature(associated_type_defaults)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::needless_lifetimes)]

use std::collections::HashMap;
use std::io::Write;
use std::iter::once;
use std::sync::Arc;

use crate::expr::{Literal, AST};
use crate::function::FunctionRegistry;
use crate::function::{vectorize_2_arg, Function, FunctionSignature};
use crate::property::{FunctionProperty, ValueProperty};
use crate::runtime::Runtime;
use crate::types::DataType;
use crate::types::*;
use crate::types::{ArgType, ArrayType, Int16Type};
use crate::values::{Column, ColumnBuilder, ValueRef};
use crate::values::{Scalar, Value};

pub mod display;
pub mod expr;
pub mod function;
pub mod property;
pub mod runtime;
pub mod type_check;
pub mod types;
pub mod util;
pub mod values;

pub fn main() {
    run_cases(&mut std::io::stdout());
}

#[test]
pub fn test() {
    use goldenfile::Mint;

    let mut mint = Mint::new("tests");
    let mut file = mint.new_goldenfile("run-output").unwrap();
    run_cases(&mut file);
}

pub fn run_ast(output: &mut impl Write, ast: &AST, columns: HashMap<String, Column>) {
    writeln!(output, "ast: {ast}").unwrap();
    let fn_registry = builtin_functions();
    let (expr, ty, prop) = type_check::check(ast, &fn_registry).unwrap();
    writeln!(output, "expr: {expr}").unwrap();
    writeln!(output, "type: {ty}").unwrap();
    writeln!(output, "property: {prop}").unwrap();
    let runtime = Runtime { columns };
    let result = runtime.run(&expr);
    writeln!(output, "result: {result}\n").unwrap();
}

fn run_cases(output: &mut impl Write) {
    run_ast(
        output,
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
        output,
        &AST::FunctionCall {
            name: "and".to_string(),
            args: vec![
                AST::Literal(Literal::Null),
                AST::Literal(Literal::Boolean(false)),
            ],
            params: vec![],
        },
        HashMap::new(),
    );

    run_ast(
        output,
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
                column: Box::new(Column::UInt8(vec![10, 11, 12].into())),
                validity: vec![false, true, false].into(),
            },
        )]
        .into_iter()
        .collect(),
    );

    run_ast(
        output,
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
                    column: Box::new(Column::UInt8(vec![10, 11, 12].into())),
                    validity: vec![false, true, false].into(),
                },
            ),
            (
                "b".to_string(),
                Column::Nullable {
                    column: Box::new(Column::UInt8(vec![1, 2, 3].into())),
                    validity: vec![false, true, true].into(),
                },
            ),
        ]
        .into_iter()
        .collect(),
    );

    run_ast(
        output,
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
                column: Box::new(Column::Boolean(vec![true, false, true].into())),
                validity: vec![false, true, false].into(),
            },
        )]
        .into_iter()
        .collect(),
    );

    run_ast(
        output,
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
        output,
        &AST::FunctionCall {
            name: "create_tuple".to_string(),
            args: vec![
                AST::Literal(Literal::Null),
                AST::Literal(Literal::Boolean(true)),
            ],
            params: vec![],
        },
        [].into_iter().collect(),
    );

    run_ast(
        output,
        &AST::FunctionCall {
            name: "get_tuple".to_string(),
            args: vec![AST::FunctionCall {
                name: "create_tuple".to_string(),
                args: vec![
                    AST::ColumnRef {
                        name: "a".to_string(),
                        data_type: DataType::Int16,
                        property: ValueProperty::default().not_null(true),
                    },
                    AST::ColumnRef {
                        name: "b".to_string(),
                        data_type: DataType::Nullable(Box::new(DataType::String)),
                        property: ValueProperty::default().not_null(false),
                    },
                ],
                params: vec![],
            }],
            params: vec![1],
        },
        [
            ("a".to_string(), Column::Int16(vec![0, 1, 2, 3, 4].into())),
            (
                "b".to_string(),
                Column::Nullable {
                    column: Box::new(Column::String {
                        data: "abcde".as_bytes().to_vec().into(),
                        offsets: vec![0, 1, 2, 3, 4, 5],
                    }),
                    validity: vec![true, true, false, false, false].into(),
                },
            ),
        ]
        .into_iter()
        .collect(),
    );

    run_ast(
        output,
        &AST::FunctionCall {
            name: "get_tuple".to_string(),
            args: vec![AST::ColumnRef {
                name: "a".to_string(),
                data_type: DataType::Nullable(Box::new(DataType::Tuple(vec![
                    DataType::Boolean,
                    DataType::String,
                ]))),
                property: ValueProperty::default().not_null(true),
            }],
            params: vec![1],
        },
        [(
            "a".to_string(),
            Column::Nullable {
                column: Box::new(Column::Tuple {
                    fields: vec![
                        Column::Boolean(vec![false; 5].into()),
                        Column::String {
                            data: "abcde".as_bytes().to_vec().into(),
                            offsets: vec![0, 1, 2, 3, 4, 5],
                        },
                    ],
                    len: 5,
                }),
                validity: vec![true, true, false, false, false].into(),
            },
        )]
        .into_iter()
        .collect(),
    );

    run_ast(
        output,
        &AST::FunctionCall {
            name: "create_array".to_string(),
            args: vec![],
            params: vec![],
        },
        [].into_iter().collect(),
    );

    run_ast(
        output,
        &AST::FunctionCall {
            name: "create_array".to_string(),
            args: vec![
                AST::Literal(Literal::Null),
                AST::Literal(Literal::Boolean(true)),
            ],
            params: vec![],
        },
        [].into_iter().collect(),
    );

    run_ast(
        output,
        &AST::FunctionCall {
            name: "create_array".to_string(),
            args: vec![
                AST::ColumnRef {
                    name: "a".to_string(),
                    data_type: DataType::Int16,
                    property: ValueProperty::default().not_null(true),
                },
                AST::ColumnRef {
                    name: "b".to_string(),
                    data_type: DataType::Int16,
                    property: ValueProperty::default().not_null(true),
                },
            ],
            params: vec![],
        },
        [
            ("a".to_string(), Column::Int16(vec![0, 1, 2, 3, 4].into())),
            ("b".to_string(), Column::Int16(vec![5, 6, 7, 8, 9].into())),
        ]
        .into_iter()
        .collect(),
    );

    run_ast(
        output,
        &AST::FunctionCall {
            name: "create_array".to_string(),
            args: vec![
                AST::FunctionCall {
                    name: "create_array".to_string(),
                    args: vec![
                        AST::ColumnRef {
                            name: "a".to_string(),
                            data_type: DataType::Int16,
                            property: ValueProperty::default().not_null(true),
                        },
                        AST::ColumnRef {
                            name: "b".to_string(),
                            data_type: DataType::Int16,
                            property: ValueProperty::default().not_null(true),
                        },
                    ],
                    params: vec![],
                },
                AST::Literal(Literal::Null),
                AST::Literal(Literal::Null),
            ],
            params: vec![],
        },
        [
            ("a".to_string(), Column::Int16(vec![0, 1, 2, 3, 4].into())),
            ("b".to_string(), Column::Int16(vec![5, 6, 7, 8, 9].into())),
        ]
        .into_iter()
        .collect(),
    );

    run_ast(
        output,
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
                    offsets: vec![0, 20, 40, 60, 80, 100],
                },
            ),
            ("idx".to_string(), Column::UInt8(vec![0, 1, 2, 3, 4].into())),
        ]
        .into_iter()
        .collect(),
    );

    run_ast(
        output,
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
                            0, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55, 60, 65, 70, 75, 80, 85,
                            90, 100,
                        ],
                    }),
                    offsets: vec![0, 4, 8, 12, 16, 20],
                },
            ),
            ("idx".to_string(), Column::UInt8(vec![0, 1, 2].into())),
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

    registry.register_function_factory("least", |_, args_type| {
        Some(Arc::new(Function {
            signature: FunctionSignature {
                name: "least",
                args_type: vec![DataType::Int16; args_type.len()],
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

    registry.register_0_arg_core::<EmptyArrayType, _>(
        "create_array",
        FunctionProperty::default(),
        |_| Value::Scalar(()),
    );

    registry.register_function_factory("create_array", |_, args_type| {
        Some(Arc::new(Function {
            signature: FunctionSignature {
                name: "create_array",
                args_type: vec![DataType::Generic(0); args_type.len()],
                return_type: DataType::Array(Box::new(DataType::Generic(0))),
                property: FunctionProperty::default().preserve_not_null(true),
            },
            eval: Box::new(|args, generics| {
                let len = args.iter().find_map(|arg| match arg {
                    ValueRef::Column(col) => Some(col.len()),
                    _ => None,
                });
                if let Some(len) = len {
                    let mut array_builder = ColumnBuilder::with_capacity(&generics[0], 0);
                    for idx in 0..len {
                        for arg in args {
                            match arg {
                                ValueRef::Scalar(scalar) => {
                                    array_builder.push(scalar.as_ref());
                                }
                                ValueRef::Column(col) => {
                                    array_builder.push(col.index(idx));
                                }
                            }
                        }
                    }
                    let offsets = once(0)
                        .chain((0..len).map(|row| args.len() * (row + 1)))
                        .collect();
                    Value::Column(Column::Array {
                        array: Box::new(array_builder.build()),
                        offsets,
                    })
                } else {
                    // All args are scalars, so we return a scalar as result
                    let mut array = ColumnBuilder::with_capacity(&generics[0], 0);
                    for arg in args {
                        match arg {
                            ValueRef::Scalar(scalar) => {
                                array.push(scalar.as_ref());
                            }
                            ValueRef::Column(_) => unreachable!(),
                        }
                    }
                    Value::Scalar(Scalar::Array(array.build()))
                }
            }),
        }))
    });

    registry.register_with_writer_2_arg::<ArrayType<GenericType<0>>, Int16Type, GenericType<0>, _>(
        "get",
        FunctionProperty::default(),
        |array, idx, output| output.push(array.index(idx as usize)),
    );

    registry.register_function_factory("create_tuple", |_, args_type| {
        Some(Arc::new(Function {
            signature: FunctionSignature {
                name: "create_tuple",
                args_type: args_type.to_vec(),
                return_type: DataType::Tuple(args_type.to_vec()),
                property: FunctionProperty::default().preserve_not_null(true),
            },
            eval: Box::new(move |args, _generics| {
                let len = args.iter().find_map(|arg| match arg {
                    ValueRef::Column(col) => Some(col.len()),
                    _ => None,
                });
                if let Some(len) = len {
                    let fields = args
                        .iter()
                        .map(|arg| match arg {
                            ValueRef::Scalar(scalar) => scalar.as_ref().repeat(len).build(),
                            ValueRef::Column(col) => col.clone(),
                        })
                        .collect();
                    Value::Column(Column::Tuple { fields, len })
                } else {
                    // All args are scalars, so we return a scalar as result
                    let fields = args
                        .iter()
                        .map(|arg| match arg {
                            ValueRef::Scalar(scalar) => (*scalar).to_owned(),
                            ValueRef::Column(_) => unreachable!(),
                        })
                        .collect();
                    Value::Scalar(Scalar::Tuple(fields))
                }
            }),
        }))
    });

    registry.register_function_factory("get_tuple", |params, args_type| {
        let idx = *params.get(0)?;
        let tuple_tys = match args_type.get(0) {
            Some(DataType::Tuple(tys)) => tys,
            _ => return None,
        };
        if idx >= tuple_tys.len() {
            return None;
        }

        Some(Arc::new(Function {
            signature: FunctionSignature {
                name: "get_tuple",
                args_type: vec![DataType::Tuple(tuple_tys.to_vec())],
                return_type: tuple_tys[idx].clone(),
                property: FunctionProperty::default().preserve_not_null(true),
            },
            eval: Box::new(move |args, _| match &args[0] {
                ValueRef::Scalar(Scalar::Tuple(fields)) => Value::Scalar(fields[idx].to_owned()),
                ValueRef::Column(Column::Tuple { fields, .. }) => {
                    Value::Column(fields[idx].to_owned())
                }
                _ => unreachable!(),
            }),
        }))
    });

    registry.register_function_factory("get_tuple", |params, args_type| {
        let idx = *params.get(0)?;
        let tuple_tys = match args_type.get(0) {
            Some(DataType::Nullable(box DataType::Tuple(tys))) => tys,
            _ => return None,
        };
        if idx >= tuple_tys.len() {
            return None;
        }

        Some(Arc::new(Function {
            signature: FunctionSignature {
                name: "get_tuple",
                args_type: vec![DataType::Nullable(Box::new(DataType::Tuple(
                    tuple_tys.to_vec(),
                )))],
                return_type: DataType::Nullable(Box::new(tuple_tys[idx].clone())),
                property: FunctionProperty::default().preserve_not_null(true),
            },
            eval: Box::new(move |args, _| match &args[0] {
                ValueRef::Scalar(Scalar::Null) => Value::Scalar(Scalar::Null),
                ValueRef::Scalar(Scalar::Tuple(fields)) => Value::Scalar(fields[idx].to_owned()),
                ValueRef::Column(Column::Nullable {
                    column: box Column::Tuple { fields, .. },
                    validity,
                }) => Value::Column(Column::Nullable {
                    column: Box::new(fields[idx].to_owned()),
                    validity: validity.clone(),
                }),
                _ => unreachable!(),
            }),
        }))
    });

    registry
}
