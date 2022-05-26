#![feature(generic_associated_types)]
#![feature(derive_default_enum)]

use std::collections::HashMap;
use std::sync::Arc;

use function::{vectorize_2_arg, Function, FunctionSignature};
use types::{Int16Type, Type};
use values::{Scalar, Value};

use crate::expr::{Literal, AST};
use crate::function::FunctionRegistry;
use crate::runtime::Runtime;
use crate::types::boolean::BooleanType;
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
            params: vec![],
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
            Arc::new(Column::Nullable(
                Arc::new(Column::Boolean(vec![true, false, true])),
                vec![false, true, false],
            )),
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

    registry
        .register_2_arg::<BooleanType, BooleanType, BooleanType, _>("and", |lhs, rhs| *lhs && *rhs);
    registry.register_2_arg::<Int16Type, Int16Type, Int16Type, _>("plus", |lhs, rhs| *lhs + *rhs);
    registry.register_1_arg::<BooleanType, BooleanType, _>("not", |lhs| !*lhs);
    registry.register_function_factory("least", |_, args_ty| {
        Some(Arc::new(Function {
            signature: FunctionSignature {
                name: "least",
                args_type: vec![DataType::Int16; args_ty.len()],
                return_type: DataType::Int16,
            },
            eval: Box::new(|args| {
                if args.len() == 0 {
                    Value::Scalar(Scalar::Int16(0))
                } else if args.len() == 1 {
                    args[0].clone().to_owned()
                } else {
                    let mut min: Value<Int16Type> = vectorize_2_arg(
                        Int16Type::try_downcast_value(&args[0]).unwrap(),
                        Int16Type::try_downcast_value(&args[1]).unwrap(),
                        |lhs, rhs| *lhs.min(rhs),
                    );
                    for arg in &args[2..] {
                        min = vectorize_2_arg(
                            min.as_ref(),
                            Int16Type::try_downcast_value(arg).unwrap(),
                            |lhs, rhs| *lhs.min(rhs),
                        );
                    }
                    Int16Type::upcast_value(min)
                }
            }),
        }))
    });

    registry
}

pub fn run_ast(ast: &AST, columns: HashMap<String, Arc<Column>>) {
    let fn_registry = builtin_functions();
    let (expr, ty) = type_check::check(&ast, &fn_registry).unwrap();
    let runtime = Runtime { columns };
    let result = runtime.run(&expr);

    println!("ast: {ast}\nexpr: {expr}\ntype: {ty}\nresult: {result}\n");
}
