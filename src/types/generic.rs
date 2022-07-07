use std::ops::Range;

use crate::values::{Column, ColumnBuilder, ColumnIterator, Scalar, ScalarRef};

use super::{ArgType, DataType, GenericMap, ValueType};

pub struct GenericType<const INDEX: usize>;

impl<const INDEX: usize> ValueType for GenericType<INDEX> {
    type Scalar = Scalar;
    type ScalarRef<'a> = ScalarRef<'a>;
    type Column = Column;

    fn to_owned_scalar<'a>(scalar: Self::ScalarRef<'a>) -> Self::Scalar {
        scalar.to_owned()
    }

    fn to_scalar_ref<'a>(scalar: &'a Self::Scalar) -> Self::ScalarRef<'a> {
        scalar.as_ref()
    }
}

impl<const INDEX: usize> ArgType for GenericType<INDEX> {
    type ColumnIterator<'a> = ColumnIterator<'a>;
    type ColumnBuilder = ColumnBuilder;

    fn data_type() -> DataType {
        DataType::Generic(INDEX)
    }

    fn try_downcast_scalar<'a>(scalar: &'a Scalar) -> Option<Self::ScalarRef<'a>> {
        Some(scalar.as_ref())
    }

    fn try_downcast_column<'a>(col: &'a Column) -> Option<Self::Column> {
        Some(col.clone())
    }

    fn upcast_scalar(scalar: Self::Scalar) -> Scalar {
        scalar
    }

    fn upcast_column(col: Self::Column) -> Column {
        col
    }

    fn column_len<'a>(col: &'a Self::Column) -> usize {
        col.len()
    }

    fn index_column<'a>(col: &'a Self::Column, index: usize) -> Self::ScalarRef<'a> {
        col.index(index)
    }

    fn slice_column<'a>(col: &'a Self::Column, range: Range<usize>) -> Self::Column {
        col.slice(range)
    }

    fn iter_column<'a>(col: &'a Self::Column) -> Self::ColumnIterator<'a> {
        col.iter()
    }

    fn create_builder(capacity: usize, generics: &GenericMap) -> Self::ColumnBuilder {
        ColumnBuilder::with_capacity(&generics[INDEX], capacity)
    }

    fn column_to_builder(col: Self::Column) -> Self::ColumnBuilder {
        ColumnBuilder::from_column(col)
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
        builder.push_default();
        builder
    }

    fn append_builder(
        mut builder: Self::ColumnBuilder,
        other: Self::ColumnBuilder,
    ) -> Self::ColumnBuilder {
        builder.append(&other);
        builder
    }

    fn build_column(builder: Self::ColumnBuilder) -> Self::Column {
        builder.build()
    }
}
