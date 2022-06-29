use std::{collections::HashMap, sync::Arc};

use crate::{
    expr::{Expr, Literal},
    types::{any::AnyType, DataType},
    values::{Column, Scalar},
    values::{Value, ValueRef},
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
                let cols = args.iter().map(|expr| self.run(expr)).collect::<Vec<_>>();
                let cols_ref = cols.iter().map(Value::as_ref).collect::<Vec<_>>();
                (function.eval)(cols_ref.as_slice(), generics)
            }
            Expr::Cast { expr, dest_type } => {
                let value = self.run(expr);
                self.run_cast(value.as_ref(), dest_type)
                    .expect(&format!("{value} can not be cast to {dest_type}"))
            }
        }
    }

    pub fn run_cast(
        &self,
        input: ValueRef<AnyType>,
        dest_type: &DataType,
    ) -> Option<Value<AnyType>> {
        match &input {
            ValueRef::Scalar(scalar) => match (scalar, dest_type) {
                (Scalar::Null, DataType::Nullable(_)) => Some(Value::Scalar(Scalar::Null)),
                (Scalar::EmptyArray, DataType::Array(dest_ty)) => {
                    let column = create_column(dest_ty, 0);
                    Some(Value::Scalar(Scalar::Array(column)))
                }
                (_, DataType::Nullable(dest_ty)) => self.run_cast(input, dest_ty),
                (Scalar::Array(array), DataType::Array(dest_ty)) => {
                    let array = self
                        .run_cast(ValueRef::Column(array), dest_ty)?
                        .into_column()
                        .ok()
                        .unwrap();
                    Some(Value::Scalar(Scalar::Array(array)))
                }
                (Scalar::UInt8(val), DataType::UInt16) => {
                    Some(Value::Scalar(Scalar::UInt16(*val as u16)))
                }
                (Scalar::Int8(val), DataType::Int16) => {
                    Some(Value::Scalar(Scalar::Int16(*val as i16)))
                }
                (Scalar::UInt8(val), DataType::Int16) => {
                    Some(Value::Scalar(Scalar::Int16(*val as i16)))
                }
                _ => None,
            },
            ValueRef::Column(col) => match (col, dest_type) {
                (Column::Null { len }, DataType::Nullable(dest_ty)) => {
                    Some(Value::Column(Column::Nullable {
                        column: Box::new(create_column(dest_ty, *len)),
                        nulls: vec![false; *len],
                    }))
                }
                (Column::EmptyArray { len }, DataType::Array(dest_ty)) => {
                    let array = Box::new(create_column(dest_ty, 0));
                    Some(Value::Column(Column::Array {
                        array,
                        offsets: vec![0..0; *len],
                    }))
                }
                (Column::Nullable { column, nulls }, DataType::Nullable(dest_ty)) => {
                    let column = self
                        .run_cast(ValueRef::Column(&**column), &*dest_ty)?
                        .into_column()
                        .ok()
                        .unwrap();
                    Some(Value::Column(Column::Nullable {
                        column: Box::new(column),
                        nulls: nulls.clone(),
                    }))
                }
                (_, DataType::Nullable(dest_ty)) => {
                    let column = self
                        .run_cast(ValueRef::Column(*col), &*dest_ty)?
                        .into_column()
                        .ok()
                        .unwrap();
                    Some(Value::Column(Column::Nullable {
                        nulls: vec![false; column.len()],
                        column: Box::new(column),
                    }))
                }
                (Column::Array { array, offsets }, DataType::Array(dest_ty)) => {
                    let array = self
                        .run_cast(ValueRef::Column(&**array), &*dest_ty)?
                        .into_column()
                        .ok()
                        .unwrap();
                    Some(Value::Column(Column::Array {
                        array: Box::new(array),
                        offsets: offsets.clone(),
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

fn create_column(dest_ty: &DataType, len: usize) -> Column {
    match dest_ty {
        DataType::Null => Column::Null { len },
        DataType::EmptyArray => Column::EmptyArray { len },
        DataType::Boolean => Column::Boolean(vec![false; len]),
        DataType::String => Column::String(vec![String::new(); len]),
        DataType::UInt8 => Column::UInt8(vec![0; len]),
        DataType::UInt16 => Column::UInt16(vec![0; len]),
        DataType::Int8 => Column::Int8(vec![0; len]),
        DataType::Int16 => Column::Int16(vec![0; len]),
        DataType::Nullable(dest_ty) => Column::Nullable {
            column: Box::new(create_column(dest_ty, len)),
            nulls: vec![false; len],
        },
        DataType::Array(dest_ty) => Column::Array {
            array: Box::new(create_column(dest_ty, len)),
            offsets: vec![0..0; len],
        },
        DataType::Generic(_) => unreachable!(),
    }
}
