use std::sync::Arc;

use crate::{
    function::{Function, FunctionID},
    property::ValueProperty,
    types::DataType,
};

#[derive(Debug, Clone)]
pub enum AST {
    Literal(Literal),
    ColumnRef {
        name: String,
        data_type: DataType,
        property: ValueProperty,
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
        generics: Vec<DataType>,
        args: Vec<(Expr, ValueProperty)>,
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
    String(Vec<u8>),
}
