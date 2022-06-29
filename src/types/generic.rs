use std::{marker::PhantomData, ops::Range, sync::Arc};

use crate::values::{Column, ColumnIterator, Scalar};

use super::{ArgType, ColumnViewer, DataType, ValueType};

pub struct GenericType<const INDEX: usize>;

impl<const INDEX: usize> ValueType for GenericType<INDEX> {
    type Scalar = Scalar;
    type ScalarRef<'a> = &'a Scalar;
    type Column = Column;
    type ColumnRef<'a> = &'a Column;

    fn to_owned_scalar<'a>(scalar: Self::ScalarRef<'a>) -> Self::Scalar {
        scalar.clone()
    }

    fn to_owned_column<'a>(col: Self::ColumnRef<'a>) -> Self::Column {
        col.clone()
    }

    fn to_scalar_ref<'a>(scalar: &'a Self::Scalar) -> Self::ScalarRef<'a> {
        &scalar
    }

    fn to_column_ref<'a>(col: &'a Self::Column) -> Self::ColumnRef<'a> {
        &col
    }
}

impl<const INDEX: usize> ArgType for GenericType<INDEX> {
    fn data_type() -> DataType {
        DataType::Generic(INDEX)
    }

    fn try_downcast_scalar<'a>(scalar: &'a Scalar) -> Option<Self::ScalarRef<'a>> {
        Some(scalar)
    }

    fn try_downcast_column<'a>(col: &'a Column) -> Option<Self::ColumnRef<'a>> {
        Some(col)
    }

    fn upcast_scalar(scalar: Self::Scalar) -> Scalar {
        scalar
    }

    fn upcast_column(col: Self::Column) -> Column {
        col
    }
}

impl<const INDEX: usize> ColumnViewer for GenericType<INDEX> {
    type ScalarBorrow<'a> = Self::Scalar;
    type ColumnIterator<'a> = ColumnIterator<'a>;

    fn scalar_borrow_to_ref<'a>(scalar: &'a Self::ScalarBorrow<'a>) -> Self::ScalarRef<'a> {
        scalar
    }

    fn column_len<'a>(col: Self::ColumnRef<'a>) -> usize {
        col.len()
    }

    fn index_column<'a>(col: Self::ColumnRef<'a>, index: usize) -> Self::ScalarBorrow<'a> {
        col.index(index)
    }

    fn slice_column<'a>(col: Self::ColumnRef<'a>, range: Range<usize>) -> Self::ColumnRef<'a> {
        &col.slice(range)
    }

    fn iter_column<'a>(col: Self::ColumnRef<'a>) -> Self::ColumnIterator<'a> {
        col.iter()
    }
}

impl<const INDEX: usize> ColumnBuilder for GenericType<INDEX> {
    fn empty_column(capacity: usize) -> Self::Column {
        Vec::with_capacity(capacity)
    }

    fn push_column(mut col: Self::Column, item: Self::Scalar) -> Self::Column {
        col.push(item);
        col
    }

    fn append_column(mut col: Self::Column, mut other: Self::Column) -> Self::Column {
        col.append(&mut other);
        col
    }

    fn column_from_iter(iter: impl Iterator<Item = Self::Scalar>) -> Self::Column {
        iter.collect()
    }
}
