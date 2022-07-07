use std::ops::Range;

use arrow2::bitmap::{Bitmap, MutableBitmap};

use crate::{
    util::bitmap_into_mut,
    values::{Column, Scalar},
};

use super::{ArgType, DataType, GenericMap, ValueType};

pub struct BooleanType;

impl ValueType for BooleanType {
    type Scalar = bool;
    type ScalarRef<'a> = bool;
    type Column = Bitmap;

    fn to_owned_scalar<'a>(scalar: Self::ScalarRef<'a>) -> Self::Scalar {
        scalar
    }

    fn to_scalar_ref<'a>(scalar: &'a Self::Scalar) -> Self::ScalarRef<'a> {
        *scalar
    }
}

impl ArgType for BooleanType {
    type ColumnIterator<'a> = arrow2::bitmap::utils::BitmapIter<'a>;
    type ColumnBuilder = MutableBitmap;

    fn data_type() -> DataType {
        DataType::Boolean
    }

    fn try_downcast_scalar<'a>(scalar: &'a Scalar) -> Option<Self::ScalarRef<'a>> {
        match scalar {
            Scalar::Boolean(scalar) => Some(*scalar),
            _ => None,
        }
    }

    fn try_downcast_column<'a>(col: &'a Column) -> Option<Self::Column> {
        match col {
            Column::Boolean(column) => Some(column.clone()),
            _ => None,
        }
    }

    fn upcast_scalar(scalar: Self::Scalar) -> Scalar {
        Scalar::Boolean(scalar)
    }

    fn upcast_column(col: Self::Column) -> Column {
        Column::Boolean(col)
    }

    fn column_len<'a>(col: &'a Self::Column) -> usize {
        col.len()
    }

    fn index_column<'a>(col: &'a Self::Column, index: usize) -> Self::ScalarRef<'a> {
        col.get(index).unwrap()
    }

    fn slice_column<'a>(col: &'a Self::Column, range: Range<usize>) -> Self::Column {
        col.clone().slice(range.start, range.end - range.start)
    }

    fn iter_column<'a>(col: &'a Self::Column) -> Self::ColumnIterator<'a> {
        col.iter()
    }

    fn column_from_iter(iter: impl Iterator<Item = Self::Scalar>, _: &GenericMap) -> Self::Column {
        iter.collect()
    }

    fn create_builer(capacity: usize, _: &GenericMap) -> Self::ColumnBuilder {
        MutableBitmap::with_capacity(capacity)
    }

    fn column_to_builder(col: Self::Column) -> Self::ColumnBuilder {
        bitmap_into_mut(col)
    }

    fn builder_len(builder: &Self::ColumnBuilder) -> usize {
        builder.len()
    }

    fn push_item(
        mut builder: Self::ColumnBuilder,
        item: Self::ScalarRef<'_>,
    ) -> Self::ColumnBuilder {
        builder.push(item);
        builder
    }

    fn push_default(mut builder: Self::ColumnBuilder) -> Self::ColumnBuilder {
        builder.push(false);
        builder
    }

    fn append_builder(
        mut builder: Self::ColumnBuilder,
        other_builder: Self::ColumnBuilder,
    ) -> Self::ColumnBuilder {
        builder.extend_from_slice(other_builder.as_slice(), 0, other_builder.len());
        builder
    }

    fn build_column(builder: Self::ColumnBuilder) -> Self::Column {
        builder.into()
    }
}
