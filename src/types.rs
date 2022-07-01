pub mod any;
pub mod array;
pub mod boolean;
pub mod empty_array;
pub mod generic;
pub mod int16;
pub mod nullable;

pub use any::AnyType;
pub use array::ArrayType;
pub use boolean::BooleanType;
pub use empty_array::EmptyArrayType;
pub use generic::GenericType;
pub use int16::Int16Type;
pub use nullable::NullableType;

use std::{fmt::Debug, ops::Range};

use enum_as_inner::EnumAsInner;

use crate::{
    values::{Column, Scalar},
    values::{Value, ValueRef},
};

pub type GenericMap<'a> = [DataType];

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum DataType {
    Boolean,
    String,
    UInt8,
    UInt16,
    Int8,
    Int16,
    Null,
    Nullable(Box<DataType>),
    EmptyArray,
    Array(Box<DataType>),
    Generic(usize),
}

pub trait ValueType: Sized + 'static {
    type Scalar: Debug;
    type ScalarRef<'a>: Debug + Clone;
    type Column: Debug;
    type ColumnRef<'a>: Debug + Clone;

    fn to_owned_scalar<'a>(scalar: Self::ScalarRef<'a>) -> Self::Scalar;
    fn to_owned_column<'a>(col: Self::ColumnRef<'a>) -> Self::Column;
    fn to_scalar_ref<'a>(scalar: &'a Self::Scalar) -> Self::ScalarRef<'a>;
    fn to_column_ref<'a>(col: &'a Self::Column) -> Self::ColumnRef<'a>;
}

pub trait ArgType: ValueType {
    fn data_type() -> DataType;
    fn try_downcast_scalar<'a>(scalar: &'a Scalar) -> Option<Self::ScalarRef<'a>>;
    fn try_downcast_column<'a>(col: &'a Column) -> Option<Self::ColumnRef<'a>>;
    fn upcast_scalar(scalar: Self::Scalar) -> Scalar;
    fn upcast_column(col: Self::Column) -> Column;

    fn try_downcast_value<'a>(value: &'a ValueRef<'_, AnyType>) -> Option<ValueRef<'a, Self>> {
        Some(match value {
            ValueRef::Scalar(scalar) => ValueRef::Scalar(Self::try_downcast_scalar(scalar)?),
            ValueRef::Column(col) => ValueRef::Column(Self::try_downcast_column(col)?),
        })
    }

    fn upcast_value(value: Value<Self>) -> Value<AnyType> {
        match value {
            Value::Scalar(scalar) => Value::Scalar(Self::upcast_scalar(scalar)),
            Value::Column(col) => Value::Column(Self::upcast_column(col)),
        }
    }
}

pub trait ColumnViewer: ValueType {
    type ColumnIterator<'a>: Iterator<Item = Self::ScalarRef<'a>>;

    fn column_len<'a>(col: Self::ColumnRef<'a>) -> usize;
    fn index_column<'a>(col: Self::ColumnRef<'a>, index: usize) -> Self::ScalarRef<'a>;
    fn slice_column<'a>(col: Self::ColumnRef<'a>, range: Range<usize>) -> Self::ColumnRef<'a>;
    fn iter_column<'a>(col: Self::ColumnRef<'a>) -> Self::ColumnIterator<'a>;

}

pub trait ColumnBuilder: ValueType {
    fn create_column(capacity: usize, generics: &GenericMap) -> Self::Column;
    fn push_column(col: Self::Column, item: Self::Scalar) -> Self::Column;
    fn append_column(col: Self::Column, other: Self::Column) -> Self::Column;

    fn column_from_iter(
        iter: impl Iterator<Item = Self::Scalar>,
        generics: &GenericMap,
    ) -> Self::Column {
        let mut col = Self::create_column(iter.size_hint().0, generics);
        for item in iter {
            col = Self::push_column(col, item);
        }
        col
    }
}
