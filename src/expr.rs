use std::sync::Arc;

use crate::{function::Function, types::DataType};

#[derive(Debug, Clone)]
pub enum AST {
    Literal(Literal<AST>),
    ColumnRef { name: String, data_type: DataType },
    FunctionCall { name: String, args: Vec<AST> },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal<Expr>),
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
    MapNullable(Vec<Cast>),
}

#[derive(Debug, Clone)]
pub enum Literal<T> {
    Null,
    Int8(i8),
    Int16(i16),
    UInt8(u8),
    UInt16(u16),
    Boolean(bool),
    Array(Vec<T>),
    String(String),
}
