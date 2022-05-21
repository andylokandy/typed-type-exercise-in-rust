use std::sync::Arc;

use crate::{function::Function, types::DataType, values::Scalar};

#[derive(Debug, Clone)]
pub enum AST {
    Literal(Literal),
    ColumnRef { name: String, data_type: DataType },
    FunctionCall { name: String, args: Vec<AST> },
}

#[derive(Debug, Clone)]
pub enum Literal {
    Null,
    Int8(i8),
    Int16(i16),
    UInt8(u8),
    UInt16(u16),
    Boolean(bool),
    String(String),
    Array(Vec<AST>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Scalar(Scalar),
    ColumnRef {
        name: String,
    },
    Cast {
        expr: Box<Expr>,
        casts: Vec<Cast>,
    },
    FunctionCall {
        function: Arc<Function>,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum Cast {
    Int8ToInt16,
    UInt8ToUInt16,
    UInt8ToInt16,
    ToNullable,
    EmptyArrayToAnyArray,
    EmptyArrayToUniformArray { item_type: DataType },
    MapNullable(Vec<Cast>),
}
