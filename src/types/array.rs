use std::{marker::PhantomData, ops::Range};

use crate::values::{Column, Scalar};

use super::{ArgType, ColumnBuilder, ColumnViewer, DataType, GenericMap, ValueType};

pub struct ArrayType<T: ArgType>(PhantomData<T>);

impl<T: ArgType> ValueType for ArrayType<T> {
    type Scalar = T::Column;
    type ScalarRef<'a> = T::ColumnRef<'a>;
    type Column = (T::Column, Vec<Range<usize>>);
    type ColumnRef<'a> = (T::ColumnRef<'a>, &'a [Range<usize>]);

    fn to_owned_scalar<'a>(scalar: Self::ScalarRef<'a>) -> Self::Scalar {
        T::to_owned_column(scalar)
    }

    fn to_owned_column<'a>((col, offsets): Self::ColumnRef<'a>) -> Self::Column {
        (T::to_owned_column(col), offsets.to_vec())
    }

    fn to_scalar_ref<'a>(scalar: &'a Self::Scalar) -> Self::ScalarRef<'a> {
        T::to_column_ref(scalar)
    }

    fn to_column_ref<'a>((col, offsets): &'a Self::Column) -> Self::ColumnRef<'a> {
        (T::to_column_ref(col), offsets)
    }
}

impl<T: ArgType> ArgType for ArrayType<T> {
    fn data_type() -> DataType {
        DataType::Array(Box::new(T::data_type()))
    }

    fn try_downcast_scalar<'a>(scalar: &'a Scalar) -> Option<Self::ScalarRef<'a>> {
        match scalar {
            Scalar::Array(array) => T::try_downcast_column(array),
            _ => None,
        }
    }

    fn try_downcast_column<'a>(col: &'a Column) -> Option<Self::ColumnRef<'a>> {
        match col {
            Column::Array { array, offsets } => Some((T::try_downcast_column(array)?, offsets)),
            _ => None,
        }
    }

    fn upcast_scalar(scalar: Self::Scalar) -> Scalar {
        Scalar::Array(T::upcast_column(scalar))
    }

    fn upcast_column((col, offsets): Self::Column) -> Column {
        Column::Array {
            array: Box::new(T::upcast_column(col)),
            offsets,
        }
    }
}

impl<T: ArgType + ColumnViewer> ColumnViewer for ArrayType<T> {
    type ColumnIterator<'a> = ArrayIterator<'a, T>;

    fn column_len<'a>((_, offsets): Self::ColumnRef<'a>) -> usize {
        offsets.len()
    }

    fn index_column<'a>((col, offsets): Self::ColumnRef<'a>, index: usize) -> Self::ScalarRef<'a> {
        T::slice_column(col, offsets[index].clone())
    }

    fn slice_column<'a>(
        (col, offsets): Self::ColumnRef<'a>,
        range: Range<usize>,
    ) -> Self::ColumnRef<'a> {
        (col, &offsets[range])
    }

    fn iter_column<'a>((col, offsets): Self::ColumnRef<'a>) -> Self::ColumnIterator<'a> {
        ArrayIterator {
            col,
            offsets: offsets.iter(),
        }
    }

    fn column_covariance<'a: 'b, 'b>(
        (col, offsets): &'b Self::ColumnRef<'a>,
    ) -> Self::ColumnRef<'b> {
        (T::column_covariance(col), offsets)
    }
}

pub struct ArrayIterator<'a, T: ColumnViewer> {
    col: T::ColumnRef<'a>,
    offsets: std::slice::Iter<'a, Range<usize>>,
}

impl<'a, T: ColumnViewer> Iterator for ArrayIterator<'a, T> {
    type Item = T::ColumnRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.offsets
            .next()
            .map(|range| T::slice_column(self.col.clone(), range.clone()))
    }
}

impl<T: ArgType + ColumnViewer + ColumnBuilder> ColumnBuilder for ArrayType<T> {
    fn create_column(capacity: usize, generics: &GenericMap) -> Self::Column {
        (
            T::create_column(capacity, generics),
            Vec::with_capacity(capacity),
        )
    }

    fn push_column((col, mut offsets): Self::Column, item: Self::Scalar) -> Self::Column {
        let begin = T::column_len(T::to_column_ref(&col));
        let end = begin + T::column_len(T::to_column_ref(&item));
        offsets.push(begin..end);
        let col = T::append_column(col, item);
        (col, offsets)
    }

    fn append_column(
        (col, mut offsets): Self::Column,
        (other_col, other_offsets): Self::Column,
    ) -> Self::Column {
        let end = offsets.iter().map(|range| range.end).max().unwrap_or(0);
        offsets.extend(
            other_offsets
                .iter()
                .map(|range| range.start + end..range.end + end),
        );
        let col = T::append_column(col, other_col);
        (col, offsets)
    }
}
