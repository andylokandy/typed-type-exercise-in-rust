use std::sync::Arc;

use crate::values::{Column, ColumnIter, Scalar};

use super::{DataType, Type};

pub struct AnyType;

impl Type for AnyType {
    type Scalar = Scalar;
    type ScalarRef<'a> = Scalar;
    type Column = Arc<Column>;
    type ColumnRef<'a> = Arc<Column>;
    type ColumnIterator<'a> = ColumnIter;

    fn data_type() -> DataType {
        DataType::Any
    }

    fn try_downcast_scalar<'a>(scalar: &'a Scalar) -> Option<Self::ScalarRef<'a>> {
        Some(scalar.clone())
    }

    fn try_downcast_column<'a>(col: &'a Arc<Column>) -> Option<Self::ColumnRef<'a>> {
        Some(col.clone())
    }

    fn upcast_scalar(scalar: Self::Scalar) -> Scalar {
        scalar
    }

    fn upcast_column(col: Self::Column) -> Arc<Column> {
        col
    }

    fn to_owned_scalar<'a>(scalar: Self::ScalarRef<'a>) -> Self::Scalar {
        scalar.clone()
    }

    fn to_owned_column<'a>(col: Self::ColumnRef<'a>) -> Self::Column {
        col.clone()
    }

    fn to_scalar_ref<'a>(scalar: &'a Self::Scalar) -> Self::ScalarRef<'a> {
        scalar.clone()
    }

    fn to_column_ref<'a>(col: &'a Self::Column) -> Self::ColumnRef<'a> {
        col.clone()
    }

    fn index_column<'a>(col: Self::ColumnRef<'a>, index: usize) -> Self::ScalarRef<'a> {
        col.get(index)
    }

    fn iter_column<'a>(col: Self::ColumnRef<'a>) -> Self::ColumnIterator<'a> {
        col.iter()
    }

    fn empty_column(_capacity: usize) -> Self::Column {
        unimplemented!()
    }

    fn push_column(_col: Self::Column, _item: Self::Scalar) -> Self::Column {
        unimplemented!()
    }
}
