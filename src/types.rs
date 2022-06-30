pub mod any;
pub mod array;
pub mod boolean;
pub mod generic;
pub mod int16;
pub mod nullable;

pub use any::AnyType;
pub use array::ArrayType;
pub use boolean::BooleanType;
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

// pub enum GenericMap {
//     Subsitution(Subsitution),
//     // TODO: use SmallVec
//     Static(Vec<DataType>),
// }

// impl Index<usize> for GenericMap {
//     type Output = DataType;

//     fn index(&self, index: usize) -> &Self::Output {
//         match self {
//             GenericMap::Subsitution(Subsitution(map)) => &map[&index],
//             GenericMap::Static(types) => &types[index],
//         }
//     }
// }

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
    type ScalarBorrow<'a> = Self::ScalarRef<'a>;
    type ColumnBorrow<'a> = Self::ColumnRef<'a>;
    type ColumnIterator<'a>: Iterator<Item = Self::ScalarBorrow<'a>>;

    fn column_len<'a>(col: Self::ColumnRef<'a>) -> usize;
    fn index_column<'a>(col: Self::ColumnRef<'a>, index: usize) -> Self::ScalarBorrow<'a>;
    fn slice_column<'a>(col: Self::ColumnRef<'a>, range: Range<usize>) -> Self::ColumnBorrow<'a>;
    fn iter_column<'a>(col: Self::ColumnRef<'a>) -> Self::ColumnIterator<'a>;

    fn scalar_borrow_to_ref<'a: 'b, 'b>(scalar: &'b Self::ScalarBorrow<'a>) -> Self::ScalarRef<'b>;
    fn column_borrow_to_ref<'a: 'b, 'b>(col: &'b Self::ColumnBorrow<'a>) -> Self::ColumnRef<'b>;
    fn column_covariance<'a: 'b, 'b>(col: &'b Self::ColumnRef<'a>) -> Self::ColumnRef<'b>;
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

// #[macro_export]
// macro_rules! dispatch_data_type {
//     ($ty:expr, $expr:expr) => {{
//         match_template::match_template! {
//             TYPE = [
//                 Boolean => $crate::types::BooleanType,
//             ],
//             match $api_version {
//                 $crate::types::DataType::TYPE => $e,
//             }
//         }
//     }};
// }
