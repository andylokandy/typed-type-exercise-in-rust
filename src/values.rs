use std::sync::Arc;

use enum_as_inner::EnumAsInner;

use crate::types::Type;

#[derive(EnumAsInner)]
pub enum Value<T: Type> {
    Scalar(T::Scalar),
    Column(T::Column),
}

#[derive(EnumAsInner)]
pub enum ValueRef<'a, T: Type> {
    Scalar(T::ScalarRef<'a>),
    Column(T::ColumnRef<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default, EnumAsInner)]
pub enum Scalar {
    #[default]
    Null,
    Int8(i8),
    Int16(i16),
    UInt8(u8),
    UInt16(u16),
    Boolean(bool),
    String(String),
    Array(Array),
}

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum Column {
    Int8(Vec<i8>),
    Int16(Vec<i16>),
    UInt8(Vec<u8>),
    UInt16(Vec<u16>),
    Boolean(Vec<bool>),
    String(Vec<String>),
    Array(Vec<Array>),
    Nullable(Arc<Column>, Vec<bool>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default, EnumAsInner)]
pub enum Array {
    #[default]
    Empty,
    Any(Vec<Scalar>),
    Uniform(Arc<Column>),
}

impl<'a, T: Type> ValueRef<'a, T> {
    pub fn to_owned(self) -> Value<T> {
        match self {
            ValueRef::Scalar(scalar) => Value::Scalar(T::to_owned_scalar(scalar)),
            ValueRef::Column(col) => Value::Column(T::to_owned_column(col)),
        }
    }
}

impl<'a, T: Type> Value<T> {
    pub fn as_ref(&'a self) -> ValueRef<'a, T> {
        match self {
            Value::Scalar(scalar) => ValueRef::Scalar(T::to_scalar_ref(scalar)),
            Value::Column(col) => ValueRef::Column(T::to_column_ref(col)),
        }
    }
}

impl<'a, T: Type> Clone for ValueRef<'a, T> {
    fn clone(&self) -> Self {
        match self {
            ValueRef::Scalar(scalar) => ValueRef::Scalar(scalar.clone()),
            ValueRef::Column(col) => ValueRef::Column(col.clone()),
        }
    }
}

impl Column {
    pub fn get(&self, index: usize) -> Scalar {
        match self {
            Column::Int8(values) => Scalar::Int8(values[index]),
            Column::Int16(values) => Scalar::Int16(values[index]),
            Column::UInt8(values) => Scalar::UInt8(values[index]),
            Column::UInt16(values) => Scalar::UInt16(values[index]),
            Column::Boolean(values) => Scalar::Boolean(values[index]),
            Column::String(values) => Scalar::String(values[index].clone()),
            Column::Array(_) => todo!(),
            Column::Nullable(column, nulls) => {
                if nulls.get(index).cloned().unwrap_or(false) {
                    Scalar::Null
                } else {
                    column.get(index)
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Column::Int8(col) => col.len(),
            Column::Int16(col) => col.len(),
            Column::UInt8(col) => col.len(),
            Column::UInt16(col) => col.len(),
            Column::Boolean(col) => col.len(),
            Column::String(col) => col.len(),
            Column::Array(_) => todo!(),
            Column::Nullable(col, _) => col.len(),
        }
    }

    pub fn iter(self: Arc<Self>) -> ColumnIter {
        ColumnIter {
            len: self.len(),
            column: self,
            index: 0,
        }
    }
}

pub struct ColumnIter {
    column: Arc<Column>,
    len: usize,
    index: usize,
}

impl Iterator for ColumnIter {
    type Item = Scalar;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let item = self.column.get(self.index);
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

pub fn combine_nulls(nulls: &[&[bool]]) -> Vec<bool> {
    let nulls: Vec<&[bool]> = nulls.iter().filter(|n| !n.is_empty()).cloned().collect();
    if nulls.is_empty() {
        return vec![];
    }

    let mut res = nulls[0].to_vec();
    for nulls in &nulls[1..] {
        for (r, n) in res.iter_mut().zip(nulls.iter()) {
            *r |= *n;
        }
    }

    res
}
