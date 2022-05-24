use std::sync::Arc;

use crate::values::{Column, Scalar};

use super::{DataType, NullableType, Type};

pub struct Int16Type;

impl Type for Int16Type {
    type WrapNullable = NullableType<Self>;
    type Scalar = i16;
    type ScalarRef<'a> = &'a i16;
    type Column = Vec<i16>;
    type ColumnRef<'a> = &'a [i16];
    type ColumnIterator<'a> = std::slice::Iter<'a, i16>;

    fn data_type() -> DataType {
        DataType::Int16
    }

    fn try_downcast_scalar<'a>(scalar: &'a Scalar) -> Option<Self::ScalarRef<'a>> {
        match scalar {
            Scalar::Int16(scalar) => Some(scalar),
            _ => None,
        }
    }

    fn try_downcast_column<'a>(col: &'a Arc<Column>) -> Option<Self::ColumnRef<'a>> {
        match &**col {
            Column::Int16(col) => Some(col),
            _ => None,
        }
    }

    fn upcast_scalar(scalar: Self::Scalar) -> Scalar {
        Scalar::Int16(scalar)
    }

    fn upcast_column(col: Self::Column) -> Arc<Column> {
        Arc::new(Column::Int16(col))
    }

    fn to_owned_scalar<'a>(scalar: Self::ScalarRef<'a>) -> Self::Scalar {
        scalar.clone()
    }

    fn to_owned_column<'a>(col: Self::ColumnRef<'a>) -> Self::Column {
        col.to_vec()
    }

    fn to_scalar_ref<'a>(scalar: &'a Self::Scalar) -> Self::ScalarRef<'a> {
        scalar
    }

    fn to_column_ref<'a>(col: &'a Self::Column) -> Self::ColumnRef<'a> {
        col
    }

    fn index_column<'a>(col: Self::ColumnRef<'a>, index: usize) -> Self::ScalarRef<'a> {
        &col[index]
    }

    fn iter_column<'a>(col: Self::ColumnRef<'a>) -> Self::ColumnIterator<'a> {
        col.iter()
    }

    fn empty_column(capacity: usize) -> Self::Column {
        Vec::with_capacity(capacity)
    }

    fn push_column(mut col: Self::Column, item: Self::Scalar) -> Self::Column {
        col.push(item);
        col
    }

    fn column_from_iter(iter: impl Iterator<Item = Self::Scalar>) -> Self::Column {
        iter.collect()
    }
}
