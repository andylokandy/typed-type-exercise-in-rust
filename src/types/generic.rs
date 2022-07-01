use std::ops::Range;

use crate::values::{Column, ColumnIterator, ColumnRef, Scalar, ScalarRef};

use super::{
    array::ArrayType, boolean::BooleanType, nullable::NullableType, ArgType, ColumnBuilder,
    ColumnViewer, DataType, GenericMap, Int16Type, ValueType,
};

pub struct GenericType<const INDEX: usize>;

impl<const INDEX: usize> ValueType for GenericType<INDEX> {
    type Scalar = Scalar;
    type ScalarRef<'a> = ScalarRef<'a>;
    type Column = Column;
    type ColumnRef<'a> = ColumnRef<'a>;

    fn to_owned_scalar<'a>(scalar: Self::ScalarRef<'a>) -> Self::Scalar {
        scalar.to_owned()
    }

    fn to_owned_column<'a>(col: Self::ColumnRef<'a>) -> Self::Column {
        col.to_owned()
    }

    fn to_scalar_ref<'a>(scalar: &'a Self::Scalar) -> Self::ScalarRef<'a> {
        scalar.as_ref()
    }

    fn to_column_ref<'a>(col: &'a Self::Column) -> Self::ColumnRef<'a> {
        col.slice_all()
    }
}

impl<const INDEX: usize> ArgType for GenericType<INDEX> {
    fn data_type() -> DataType {
        DataType::Generic(INDEX)
    }

    fn try_downcast_scalar<'a>(scalar: &'a Scalar) -> Option<Self::ScalarRef<'a>> {
        Some(scalar.as_ref())
    }

    fn try_downcast_column<'a>(col: &'a Column) -> Option<Self::ColumnRef<'a>> {
        Some(col.slice_all())
    }

    fn upcast_scalar(scalar: Self::Scalar) -> Scalar {
        scalar
    }

    fn upcast_column(col: Self::Column) -> Column {
        col
    }
}

impl<const INDEX: usize> ColumnViewer for GenericType<INDEX> {
    type ColumnIterator<'a> = ColumnIterator<'a>;

    fn column_len<'a>(col: Self::ColumnRef<'a>) -> usize {
        col.len()
    }

    fn index_column<'a>(col: Self::ColumnRef<'a>, index: usize) -> Self::ScalarRef<'a> {
        col.index(index)
    }

    fn slice_column<'a>(col: Self::ColumnRef<'a>, range: Range<usize>) -> Self::ColumnRef<'a> {
        col.slice(range)
    }

    fn iter_column<'a>(col: Self::ColumnRef<'a>) -> Self::ColumnIterator<'a> {
        col.iter()
    }

    fn column_covariance<'a: 'b, 'b>(col: &'b Self::ColumnRef<'a>) -> Self::ColumnRef<'b> {
        col.clone()
    }
}

impl<const INDEX: usize> ColumnBuilder for GenericType<INDEX> {
    fn create_column(capacity: usize, generics: &GenericMap) -> Self::Column {
        match &generics[INDEX] {
            DataType::Boolean => {
                BooleanType::upcast_column(BooleanType::create_column(capacity, generics))
            }
            DataType::Int16 => {
                Int16Type::upcast_column(Int16Type::create_column(capacity, generics))
            }
            DataType::Nullable(box ty) => NullableType::<GenericType<0>>::upcast_column(
                NullableType::<GenericType<0>>::create_column(capacity, &[ty.clone()]),
            ),
            DataType::Array(box ty) => {
                ArrayType::<GenericType<0>>::upcast_column(
                    ArrayType::<GenericType<0>>::create_column(capacity, &[ty.clone()]),
                )
            }
            ty => todo!("{ty}"),
        }
    }

    fn push_column(mut col: Self::Column, item: Self::Scalar) -> Self::Column {
        col.push(item);
        col
    }

    fn append_column(mut col: Self::Column, other: Self::Column) -> Self::Column {
        col.append(&other);
        col
    }
}
