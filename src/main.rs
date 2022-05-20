#![feature(generic_associated_types)]
#![feature(derive_default_enum)]

use std::collections::HashMap;
use std::sync::Arc;

use crate::expr::{Literal, AST};
use crate::function::{
    vectorize_binary, vectorize_binary_passthrough_nullable, Function, FunctionRegistry,
};
use crate::runtime::Runtime;
use crate::types::boolean::BooleanType;
use crate::types::int16::Int16Type;
use crate::types::nullable::NullableType;
use crate::types::DataType;
use crate::values::Column;

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
        },
        [(
            "a".to_string(),
            Arc::new(Column::Nullable(
                Arc::new(Column::UInt8(vec![10, 11, 12])),
                vec![false, true, false],
            )),
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
        },
        [
            (
                "a".to_string(),
                Arc::new(Column::Nullable(
                    Arc::new(Column::UInt8(vec![10, 11, 12])),
                    vec![false, true, false],
                )),
            ),
            (
                "b".to_string(),
                Arc::new(Column::Nullable(
                    Arc::new(Column::UInt8(vec![1, 2, 3])),
                    vec![false, true, true],
                )),
            ),
        ]
        .into_iter()
        .collect(),
    );
}

fn builtin_functions() -> FunctionRegistry {
    FunctionRegistry::with_builtins(vec![
        Function::new_2_arg::<BooleanType, BooleanType, BooleanType, _>("and", |lhs, rhs| {
            vectorize_binary(lhs, rhs, |lhs: &bool, rhs: &bool| *lhs && *rhs)
        }),
        Function::new_2_arg::<
            NullableType<BooleanType>,
            NullableType<BooleanType>,
            NullableType<BooleanType>,
            _,
        >("and", |lhs, rhs| {
            vectorize_binary_passthrough_nullable(lhs, rhs, |lhs: &bool, rhs: &bool| *lhs && *rhs)
        }),
        Function::new_2_arg::<
            NullableType<Int16Type>,
            NullableType<Int16Type>,
            NullableType<Int16Type>,
            _,
        >("plus", |lhs, rhs| {
            vectorize_binary_passthrough_nullable(lhs, rhs, |lhs: &i16, rhs: &i16| *lhs + *rhs)
        }),
    ])
}

pub fn run_ast(ast: &AST, columns: HashMap<String, Arc<Column>>) {
    let fn_registry = builtin_functions();
    let (expr, ty) = type_check::check(&ast, &fn_registry).unwrap();
    let runtime = Runtime { columns };
    let result = runtime.run(&expr);

    println!("ast: {ast}\nexpr: {expr}\ntype: {ty}\nresult: {result}\n");
}
