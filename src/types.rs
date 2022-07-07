pub mod any;
pub mod array;
pub mod boolean;
pub mod empty_array;
pub mod generic;
pub mod int16;
pub mod null;
pub mod nullable;

pub use any::AnyType;
pub use array::ArrayType;
use arrow2::trusted_len::TrustedLen;
pub use boolean::BooleanType;
pub use empty_array::EmptyArrayType;
pub use generic::GenericType;
pub use int16::Int16Type;
pub use null::NullType;
pub use nullable::NullableType;

use std::{fmt::Debug, ops::Range};

use enum_as_inner::EnumAsInner;

use crate::{
    values::Scalar,
    values::{Column, Value, ValueRef},
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
    Tuple(Vec<DataType>),
    Generic(usize),
}

pub trait ValueType: Sized + 'static {
    type Scalar: Debug + Clone;
    type ScalarRef<'a>: Debug + Clone;
    type Column: Debug + Clone;

    fn to_owned_scalar<'a>(scalar: Self::ScalarRef<'a>) -> Self::Scalar;
    fn to_scalar_ref<'a>(scalar: &'a Self::Scalar) -> Self::ScalarRef<'a>;
}

pub trait ArgType: ValueType {
    type ColumnIterator<'a>: Iterator<Item = Self::ScalarRef<'a>> + TrustedLen;
    type ColumnBuilder;

    fn data_type() -> DataType;
    fn try_downcast_scalar<'a>(scalar: &'a Scalar) -> Option<Self::ScalarRef<'a>>;
    fn try_downcast_column<'a>(col: &'a Column) -> Option<Self::Column>;
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

    fn column_len<'a>(col: &'a Self::Column) -> usize;
    fn index_column<'a>(col: &'a Self::Column, index: usize) -> Self::ScalarRef<'a>;
    fn slice_column<'a>(col: &'a Self::Column, range: Range<usize>) -> Self::Column;
    fn iter_column<'a>(col: &'a Self::Column) -> Self::ColumnIterator<'a>;
    fn column_from_iter(
        iter: impl Iterator<Item = Self::Scalar>,
        generics: &GenericMap,
    ) -> Self::Column {
        let mut col = Self::create_builder(iter.size_hint().0, generics);
        for item in iter {
            col = Self::push_item(col, Self::to_scalar_ref(&item));
        }
        Self::build_column(col)
    }

    fn create_builder(capacity: usize, generics: &GenericMap) -> Self::ColumnBuilder;
    fn column_to_builder(col: Self::Column) -> Self::ColumnBuilder;
    fn builder_len(builder: &Self::ColumnBuilder) -> usize;
    fn push_item(builder: Self::ColumnBuilder, item: Self::ScalarRef<'_>) -> Self::ColumnBuilder;
    fn push_default(builder: Self::ColumnBuilder) -> Self::ColumnBuilder;
    fn append_builder(
        builder: Self::ColumnBuilder,
        other_builder: Self::ColumnBuilder,
    ) -> Self::ColumnBuilder;
    fn build_column(builder: Self::ColumnBuilder) -> Self::Column;
}
