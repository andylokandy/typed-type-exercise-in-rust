use std::collections::HashMap;

use crate::{
    expr::{Expr, Literal},
    types::{any::AnyType, DataType},
    util::constant_bitmap,
    values::{Column, Value},
    values::{ColumnBuilder, Scalar},
};

pub struct Runtime {
    pub columns: HashMap<String, Column>,
}

impl Runtime {
    pub fn run(&self, expr: &Expr) -> Value<AnyType> {
        match expr {
            Expr::Literal(lit) => Value::Scalar(self.run_lit(lit)),
            Expr::ColumnRef { name } => Value::Column(self.columns[name].clone()),
            Expr::FunctionCall {
                function,
                args,
                generics,
                ..
            } => {
                let cols = args
                    .iter()
                    .map(|(expr, _)| self.run(expr))
                    .collect::<Vec<_>>();
                let cols_ref = cols.iter().map(Value::as_ref).collect::<Vec<_>>();
                (function.eval)(cols_ref.as_slice(), generics)
            }
            Expr::Cast { expr, dest_type } => {
                let value = self.run(expr);
                // TODO: remove me
                let desc_value = format!("{}", value);
                self.run_cast(value, dest_type)
                    .unwrap_or_else(|| panic!("{desc_value} can not be cast to {dest_type}"))
            }
        }
    }

    pub fn run_cast(&self, input: Value<AnyType>, dest_type: &DataType) -> Option<Value<AnyType>> {
        match input {
            Value::Scalar(scalar) => match (scalar, dest_type) {
                (Scalar::Null, DataType::Nullable(_)) => Some(Value::Scalar(Scalar::Null)),
                (Scalar::EmptyArray, DataType::Array(dest_ty)) => {
                    let column = ColumnBuilder::with_capacity(dest_ty, 0).build();
                    Some(Value::Scalar(Scalar::Array(column)))
                }
                (scalar, DataType::Nullable(dest_ty)) => {
                    self.run_cast(Value::Scalar(scalar), dest_ty)
                }
                (Scalar::Array(array), DataType::Array(dest_ty)) => {
                    let array = self
                        .run_cast(Value::Column(array), dest_ty)?
                        .into_column()
                        .ok()
                        .unwrap();
                    Some(Value::Scalar(Scalar::Array(array)))
                }
                (Scalar::UInt8(val), DataType::UInt16) => {
                    Some(Value::Scalar(Scalar::UInt16(val as u16)))
                }
                (Scalar::Int8(val), DataType::Int16) => {
                    Some(Value::Scalar(Scalar::Int16(val as i16)))
                }
                (Scalar::UInt8(val), DataType::Int16) => {
                    Some(Value::Scalar(Scalar::Int16(val as i16)))
                }
                (scalar @ Scalar::Boolean(_), DataType::Boolean)
                | (scalar @ Scalar::String(_), DataType::String)
                | (scalar @ Scalar::UInt8(_), DataType::UInt8)
                | (scalar @ Scalar::Int8(_), DataType::Int8)
                | (scalar @ Scalar::Int16(_), DataType::Int16)
                | (scalar @ Scalar::Null, DataType::Null)
                | (scalar @ Scalar::EmptyArray, DataType::EmptyArray) => {
                    Some(Value::Scalar(scalar))
                }
                _ => None,
            },
            Value::Column(col) => match (col, dest_type) {
                (Column::Null { len }, DataType::Nullable(dest_ty)) => {
                    Some(Value::Column(Column::Nullable {
                        column: Box::new(ColumnBuilder::with_capacity(dest_ty, len).build()),
                        validity: constant_bitmap(false, len).into(),
                    }))
                }
                (Column::EmptyArray { len }, DataType::Array(dest_ty)) => {
                    Some(Value::Column(Column::Array {
                        array: Box::new(ColumnBuilder::with_capacity(dest_ty, 0).build()),
                        offsets: vec![0; len + 1],
                    }))
                }
                (Column::Nullable { column, validity }, DataType::Nullable(dest_ty)) => {
                    let column = self
                        .run_cast(Value::Column(*column), &*dest_ty)?
                        .into_column()
                        .ok()
                        .unwrap();
                    Some(Value::Column(Column::Nullable {
                        column: Box::new(column),
                        validity,
                    }))
                }
                (col, DataType::Nullable(dest_ty)) => {
                    let column = self
                        .run_cast(Value::Column(col), &*dest_ty)?
                        .into_column()
                        .ok()
                        .unwrap();
                    Some(Value::Column(Column::Nullable {
                        validity: constant_bitmap(true, column.len()).into(),
                        column: Box::new(column),
                    }))
                }
                (Column::Array { array, offsets }, DataType::Array(dest_ty)) => {
                    let array = self
                        .run_cast(Value::Column(*array), &*dest_ty)?
                        .into_column()
                        .ok()
                        .unwrap();
                    Some(Value::Column(Column::Array {
                        array: Box::new(array),
                        offsets,
                    }))
                }
                (Column::UInt8(column), DataType::UInt16) => Some(Value::Column(Column::UInt16(
                    column.iter().map(|v| *v as u16).collect(),
                ))),
                (Column::Int8(column), DataType::Int16) => Some(Value::Column(Column::Int16(
                    column.iter().map(|v| *v as i16).collect(),
                ))),
                (Column::UInt8(column), DataType::Int16) => Some(Value::Column(Column::Int16(
                    column.iter().map(|v| *v as i16).collect(),
                ))),
                (col @ Column::Boolean(_), DataType::Boolean)
                | (col @ Column::String(_), DataType::String)
                | (col @ Column::UInt8(_), DataType::UInt8)
                | (col @ Column::Int8(_), DataType::Int8)
                | (col @ Column::Int16(_), DataType::Int16)
                | (col @ Column::Null { .. }, DataType::Null)
                | (col @ Column::EmptyArray { .. }, DataType::EmptyArray) => {
                    Some(Value::Column(col))
                }
                _ => None,
            },
        }
    }

    pub fn run_lit(&self, lit: &Literal) -> Scalar {
        match lit {
            Literal::Null => Scalar::Null,
            Literal::Int8(val) => Scalar::Int8(*val),
            Literal::Int16(val) => Scalar::Int16(*val),
            Literal::UInt8(val) => Scalar::UInt8(*val),
            Literal::UInt16(val) => Scalar::UInt16(*val),
            Literal::Boolean(val) => Scalar::Boolean(*val),
            Literal::String(val) => Scalar::String(val.clone()),
        }
    }
}
