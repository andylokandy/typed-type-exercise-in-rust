use std::ops::Range;

use crate::values::{Column, Scalar};

use super::{ArgType, DataType, GenericMap, ValueType};

pub struct NullType;

impl ValueType for NullType {
    type Scalar = ();
    type ScalarRef<'a> = ();
    type Column = usize;

    fn to_owned_scalar<'a>(scalar: Self::ScalarRef<'a>) -> Self::Scalar {
        scalar
    }

    fn to_scalar_ref<'a>(scalar: &'a Self::Scalar) -> Self::ScalarRef<'a> {
        *scalar
    }
}

impl ArgType for NullType {
    type ColumnIterator<'a> = std::iter::Take<std::iter::Repeat<()>>;
    type ColumnBuilder = usize;

    fn data_type() -> DataType {
        DataType::Null
    }

    fn try_downcast_scalar<'a>(scalar: &'a Scalar) -> Option<Self::ScalarRef<'a>> {
        match scalar {
            Scalar::Null => Some(()),
            _ => None,
        }
    }

    fn try_downcast_column<'a>(col: &'a Column) -> Option<Self::Column> {
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

    fn column_len<'a>(len: &'a Self::Column) -> usize {
        *len
    }

    fn index_column<'a>(len: &'a Self::Column, index: usize) -> Self::ScalarRef<'a> {
        if index >= *len {
            panic!("index {index} out of 0..{len}");
        }
    }

    fn slice_column<'a>(len: &'a Self::Column, range: Range<usize>) -> Self::Column {
        if range.end <= *len {
            range.end - range.start
        } else {
            panic!("range {range:?} out of 0..{len}");
        }
    }

    fn iter_column<'a>(len: &'a Self::Column) -> Self::ColumnIterator<'a> {
        std::iter::repeat(()).take(*len)
    }

    fn column_from_iter(iter: impl Iterator<Item = Self::Scalar>, _: &GenericMap) -> Self::Column {
        iter.count()
    }

    fn create_builder(_capacity: usize, _generics: &GenericMap) -> Self::ColumnBuilder {
        0
    }

    fn column_to_builder(len: Self::Column) -> Self::ColumnBuilder {
        len
    }

    fn builder_len(len: &Self::ColumnBuilder) -> usize {
        *len
    }

    fn push_item(len: &mut Self::ColumnBuilder, _item: Self::Scalar) {
        *len += 1
    }

    fn push_default(len: &mut Self::ColumnBuilder) {
        *len += 1
    }

    fn append_builder(len: &mut Self::ColumnBuilder, other_len: &Self::ColumnBuilder) {
        *len += other_len
    }

    fn build_column(len: Self::ColumnBuilder) -> Self::Column {
        len
    }

    fn build_scalar(len: Self::ColumnBuilder) -> Self::Scalar {
        assert_eq!(len, 1);
    }
}
