pub mod any;
pub mod boolean;
pub mod int16;
pub mod nullable;

pub use any::AnyType;
pub use boolean::BooleanType;
pub use int16::Int16Type;
pub use nullable::NullableType;

use std::{fmt::Debug, sync::Arc};

use crate::values::{Column, Scalar};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    Any,
    Hole,
    Nullable(Box<DataType>),
    Array(Box<DataType>),
    Boolean,
    String,
    UInt8,
    UInt16,
    Int8,
    Int16,
}

pub trait Type: 'static {
    type WrapNullable: Type;
    type Scalar: Debug + Default;
    type ScalarRef<'a>: Debug + Clone;
    type Column: Debug;
    type ColumnRef<'a>: Debug + Clone;
    type ColumnIterator<'a>: Iterator<Item = Self::ScalarRef<'a>>;

    fn data_type() -> DataType;
    fn try_downcast_scalar<'a>(scalar: &'a Scalar) -> Option<Self::ScalarRef<'a>>;
    fn try_downcast_column<'a>(col: &'a Arc<Column>) -> Option<Self::ColumnRef<'a>>;
    fn upcast_scalar(scalar: Self::Scalar) -> Scalar;
    fn upcast_column(scalar: Self::Column) -> Arc<Column>;
    fn to_owned_scalar<'a>(scalar: Self::ScalarRef<'a>) -> Self::Scalar;
    fn to_owned_column<'a>(col: Self::ColumnRef<'a>) -> Self::Column;
    fn to_scalar_ref<'a>(scalar: &'a Self::Scalar) -> Self::ScalarRef<'a>;
    fn to_column_ref<'a>(col: &'a Self::Column) -> Self::ColumnRef<'a>;
    fn index_column<'a>(col: Self::ColumnRef<'a>, index: usize) -> Self::ScalarRef<'a>;
    fn iter_column<'a>(col: Self::ColumnRef<'a>) -> Self::ColumnIterator<'a>;
    fn empty_column(capacity: usize) -> Self::Column;
    fn push_column(col: Self::Column, item: Self::Scalar) -> Self::Column;

    fn column_from_iter(iter: impl Iterator<Item = Self::Scalar>) -> Self::Column {
        let mut col = Self::empty_column(iter.size_hint().0);
        for item in iter {
            col = Self::push_column(col, item);
        }
        col
    }
}
