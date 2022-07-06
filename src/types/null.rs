use std::ops::Range;

use crate::values::{Column, Scalar};

use super::{ArgType, DataType, GenericMap, ValueType};

pub struct NullType;

impl ValueType for NullType {
    type Scalar = ();
    type ScalarRef<'a> = ();
    type Column = usize;
    type ColumnRef<'a> = usize;

    fn to_owned_scalar<'a>(scalar: Self::ScalarRef<'a>) -> Self::Scalar {
        scalar
    }

    fn to_owned_column<'a>(len: Self::ColumnRef<'a>) -> Self::Column {
        len
    }

    fn to_scalar_ref<'a>(scalar: &'a Self::Scalar) -> Self::ScalarRef<'a> {
        *scalar
    }

    fn to_column_ref<'a>(len: &'a Self::Column) -> Self::ColumnRef<'a> {
        *len
    }
}

impl ArgType for NullType {
    type ColumnIterator<'a> = std::iter::Take<std::iter::Repeat<()>>;

    fn data_type() -> DataType {
        DataType::Null
    }

    fn try_downcast_scalar<'a>(scalar: &'a Scalar) -> Option<Self::ScalarRef<'a>> {
        match scalar {
            Scalar::Null => Some(()),
            _ => None,
        }
    }

    fn try_downcast_column<'a>(col: &'a Column) -> Option<Self::ColumnRef<'a>> {
        match col {
            Column::Null { len } => Some(*len),
            _ => None,
        }
    }

    fn upcast_scalar(_: Self::Scalar) -> Scalar {
        Scalar::Null
    }

    fn upcast_column(len: Self::Column) -> Column {
        Column::Null { len }
    }

    fn column_len<'a>(len: Self::ColumnRef<'a>) -> usize {
        len
    }

    fn index_column<'a>(len: Self::ColumnRef<'a>, index: usize) -> Self::ScalarRef<'a> {
        if index >= len {
            panic!("index {index} out of 0..{len}");
        }
    }

    fn slice_column<'a>(len: Self::ColumnRef<'a>, range: Range<usize>) -> Self::ColumnRef<'a> {
        if range.start < len && range.end <= len {
            range.end - range.start
        } else {
            panic!("range {range:?} out of 0..{len}");
        }
    }

    fn iter_column<'a>(len: Self::ColumnRef<'a>) -> Self::ColumnIterator<'a> {
        std::iter::repeat(()).take(len)
    }

    fn create_column(_capacity: usize, _: &GenericMap) -> Self::Column {
        0
    }

    fn push_column(len: Self::Column, _: Self::Scalar) -> Self::Column {
        len + 1
    }

    fn append_column(len: Self::Column, other_len: Self::Column) -> Self::Column {
        len + other_len
    }

    fn column_from_iter(iter: impl Iterator<Item = Self::Scalar>, _: &GenericMap) -> Self::Column {
        iter.count()
    }
}
