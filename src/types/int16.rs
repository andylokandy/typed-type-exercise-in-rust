use std::ops::Range;

use crate::values::{Column, Scalar};

use super::{ArgType, DataType, GenericMap, ValueType};

pub struct Int16Type;

impl ValueType for Int16Type {
    type Scalar = i16;
    type ScalarRef<'a> = i16;
    type Column = Vec<i16>;
    type ColumnRef<'a> = &'a [i16];

    fn to_owned_scalar<'a>(scalar: Self::ScalarRef<'a>) -> Self::Scalar {
        scalar
    }

    fn to_owned_column<'a>(col: Self::ColumnRef<'a>) -> Self::Column {
        col.to_vec()
    }

    fn to_scalar_ref<'a>(scalar: &'a Self::Scalar) -> Self::ScalarRef<'a> {
        *scalar
    }

    fn to_column_ref<'a>(col: &'a Self::Column) -> Self::ColumnRef<'a> {
        col
    }
}

impl ArgType for Int16Type {
    type ColumnIterator<'a> = std::iter::Cloned<std::slice::Iter<'a, i16>>;

    fn data_type() -> DataType {
        DataType::Int16
    }

    fn try_downcast_scalar<'a>(scalar: &'a Scalar) -> Option<Self::ScalarRef<'a>> {
        match scalar {
            Scalar::Int16(scalar) => Some(*scalar),
            _ => None,
        }
    }

    fn try_downcast_column<'a>(col: &'a Column) -> Option<Self::ColumnRef<'a>> {
        match col {
            Column::Int16(column) => Some(column),
            _ => None,
        }
    }

    fn upcast_scalar(scalar: Self::Scalar) -> Scalar {
        Scalar::Int16(scalar)
    }

    fn upcast_column(col: Self::Column) -> Column {
        Column::Int16(col)
    }

    fn column_len<'a>(col: Self::ColumnRef<'a>) -> usize {
        col.len()
    }

    fn index_column<'a>(col: Self::ColumnRef<'a>, index: usize) -> Self::ScalarRef<'a> {
        col[index]
    }

    fn slice_column<'a>(col: Self::ColumnRef<'a>, range: Range<usize>) -> Self::ColumnRef<'a> {
        &col[range]
    }

    fn iter_column<'a>(col: Self::ColumnRef<'a>) -> Self::ColumnIterator<'a> {
        col.iter().cloned()
    }

    fn create_column(capacity: usize, _: &GenericMap) -> Self::Column {
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

    fn column_from_iter(iter: impl Iterator<Item = Self::Scalar>, _: &GenericMap) -> Self::Column {
        iter.collect()
    }
}
