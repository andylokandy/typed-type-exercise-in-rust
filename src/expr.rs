use std::sync::Arc;

use crate::{
    function::{Function, FunctionID},
    types::{DataType, GenericMap},
};

#[derive(Debug, Clone)]
pub enum AST {
    Literal(Literal),
    ColumnRef {
        name: String,
        data_type: DataType,
    },
    FunctionCall {
        name: String,
        params: Vec<usize>,
        args: Vec<AST>,
    },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    ColumnRef {
        name: String,
    },
    Cast {
        expr: Box<Expr>,
        dest_type: DataType,
    },
    FunctionCall {
        id: FunctionID,
        function: Arc<Function>,
        generics: GenericMap,
        args: Vec<Expr>,
    },
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
}
