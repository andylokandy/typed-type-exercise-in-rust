use std::ops::Range;

use arrow2::buffer::Buffer;

use crate::values::{Column, Scalar};

use super::{ArgType, DataType, GenericMap, ValueType};

pub struct Int16Type;

impl ValueType for Int16Type {
    type Scalar = i16;
    type ScalarRef<'a> = i16;
    type Column = Buffer<i16>;

    fn to_owned_scalar<'a>(scalar: Self::ScalarRef<'a>) -> Self::Scalar {
        scalar
    }

    fn to_scalar_ref<'a>(scalar: &'a Self::Scalar) -> Self::ScalarRef<'a> {
        *scalar
    }
}

impl ArgType for Int16Type {
    type ColumnIterator<'a> = std::iter::Cloned<std::slice::Iter<'a, i16>>;
    type ColumnBuilder = Vec<i16>;

    fn data_type() -> DataType {
        DataType::Int16
    }

    fn try_downcast_scalar<'a>(scalar: &'a Scalar) -> Option<Self::ScalarRef<'a>> {
        match scalar {
            Scalar::Int16(scalar) => Some(*scalar),
            _ => None,
        }
    }

    fn try_downcast_column<'a>(col: &'a Column) -> Option<Self::Column> {
        match col {
            Column::Int16(column) => Some(column.clone()),
            _ => None,
        }
    }

    fn upcast_scalar(scalar: Self::Scalar) -> Scalar {
        Scalar::Int16(scalar)
    }

    fn upcast_column(col: Self::Column) -> Column {
        Column::Int16(col)
    }

    fn column_len<'a>(col: &'a Self::Column) -> usize {
        col.len()
    }

    fn index_column<'a>(col: &'a Self::Column, index: usize) -> Self::ScalarRef<'a> {
        col[index]
    }

    fn slice_column<'a>(col: &'a Self::Column, range: Range<usize>) -> Self::Column {
        col.clone().slice(range.start, range.end - range.start)
    }

    fn iter_column<'a>(col: &'a Self::Column) -> Self::ColumnIterator<'a> {
        col.iter().cloned()
    }

    fn column_from_iter(iter: impl Iterator<Item = Self::Scalar>, _: &GenericMap) -> Self::Column {
        iter.collect()
    }

    fn create_builer(capacity: usize, _generics: &GenericMap) -> Self::ColumnBuilder {
        Vec::with_capacity(capacity)
    }

    fn column_to_builder(col: Self::Column) -> Self::ColumnBuilder {
        col.to_vec()
    }

    fn builder_len(builder: &Self::ColumnBuilder) -> usize {
        builder.len()
    }

    fn push_item(mut builder: Self::ColumnBuilder, item: Self::Scalar) -> Self::ColumnBuilder {
        builder.push(item);
        builder
    }

    fn push_default(mut builder: Self::ColumnBuilder) -> Self::ColumnBuilder {
        builder.push(0);
        builder
    }

    fn append_builder(
        mut builder: Self::ColumnBuilder,
        mut other_builder: Self::ColumnBuilder,
    ) -> Self::ColumnBuilder {
        builder.append(&mut other_builder);
        builder
    }

    fn build_column(builder: Self::ColumnBuilder) -> Self::Column {
        builder.into()
    }
}
