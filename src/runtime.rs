use std::{collections::HashMap, sync::Arc};

use enum_as_inner::EnumAsInner;

use crate::{
    expr::{Cast, Expr, Literal},
    types::{any::AnyType, Type},
    values::{Column, Scalar},
};

#[derive(EnumAsInner)]
pub enum Value<T: Type> {
    Scalar(T::Scalar),
    Column(T::Column),
}

pub enum ValueRef<'a, T: Type> {
    Scalar(T::ScalarRef<'a>),
    Column(T::ColumnRef<'a>),
}

pub struct Runtime {
    pub columns: HashMap<String, Arc<Column>>,
}

impl Runtime {
    pub fn run(&self, expr: &Expr) -> Value<AnyType> {
        match expr {
            Expr::Literal(lit) => Value::Scalar(self.run_lit(lit)),
            Expr::ColumnRef { name } => Value::Column(self.columns[name].clone()),
            Expr::FunctionCall { function, args } => {
                let cols = args.iter().map(|expr| self.run(expr)).collect::<Vec<_>>();
                let cols_ref = cols.iter().map(Into::into).collect::<Vec<_>>();
                (function.eval)(cols_ref.as_slice())
            }
            Expr::Cast { expr, casts } => self.run_cast((&self.run(expr)).into(), casts),
        }
    }

    pub fn run_cast(&self, input: ValueRef<AnyType>, casts: &[Cast]) -> Value<AnyType> {
        casts.iter().fold(input.into(), |val, cast| match cast {
            Cast::Int8ToInt16 => match val {
                Value::Scalar(Scalar::Int8(val)) => Value::Scalar(Scalar::Int16(val as i16)),
                Value::Column(col) => {
                    let col = col.as_int8().unwrap();
                    Value::Column(Arc::new(Column::Int16(
                        col.iter().map(|val| *val as i16).collect(),
                    )))
                }
                _ => unreachable!(),
            },
            Cast::UInt8ToUInt16 => match val {
                Value::Scalar(Scalar::UInt8(val)) => Value::Scalar(Scalar::UInt16(val as u16)),
                Value::Column(col) => {
                    let col = col.as_u_int8().unwrap();
                    Value::Column(Arc::new(Column::UInt16(
                        col.iter().map(|val| *val as u16).collect(),
                    )))
                }
                _ => unreachable!(),
            },
            Cast::UInt8ToInt16 => match val {
                Value::Scalar(Scalar::UInt8(val)) => Value::Scalar(Scalar::Int16(val as i16)),
                Value::Column(col) => {
                    let col = col.as_u_int8().unwrap();
                    Value::Column(Arc::new(Column::Int16(
                        col.iter().map(|val| *val as i16).collect(),
                    )))
                }
                _ => unreachable!(),
            },
            Cast::ToNullable => match val {
                Value::Scalar(scalar) => Value::Scalar(scalar),
                Value::Column(col) => Value::Column(Arc::new(Column::Nullable(col, Vec::new()))),
            },
            Cast::MapNullable(casts) => match val {
                Value::Scalar(Scalar::Null) => Value::Scalar(Scalar::Null),
                val @ Value::Scalar(_) => self.run_cast((&val).into(), casts),
                Value::Column(col) => {
                    let (col, nulls) = col.as_nullable().unwrap();
                    let col = self.run_cast((&Value::Column(col.clone())).into(), casts);
                    Value::Column(Arc::new(Column::Nullable(
                        col.into_column().ok().unwrap(),
                        nulls.clone(),
                    )))
                }
            },
        })
    }

    pub fn run_lit(&self, lit: &Literal<Expr>) -> Scalar {
        match lit {
            Literal::Null => Scalar::Null,
            Literal::Int8(val) => Scalar::Int8(*val),
            Literal::Int16(val) => Scalar::Int16(*val),
            Literal::UInt8(val) => Scalar::UInt8(*val),
            Literal::UInt16(val) => Scalar::UInt16(*val),
            Literal::Boolean(val) => Scalar::Boolean(*val),
            Literal::String(val) => Scalar::String(val.clone()),
            Literal::Array(_items) => todo!(),
        }
    }
}

impl<'a, T: Type> From<&'a Value<T>> for ValueRef<'a, T> {
    fn from(value: &'a Value<T>) -> Self {
        match value {
            Value::Scalar(scalar) => ValueRef::Scalar(T::to_scalar_ref(scalar)),
            Value::Column(col) => ValueRef::Column(T::to_column_ref(col)),
        }
    }
}

impl<'a, T: Type> From<ValueRef<'a, T>> for Value<T> {
    fn from(value: ValueRef<'a, T>) -> Self {
        match value {
            ValueRef::Scalar(scalar) => Value::Scalar(T::to_owned_scalar(scalar)),
            ValueRef::Column(col) => Value::Column(T::to_owned_column(col)),
        }
    }
}
