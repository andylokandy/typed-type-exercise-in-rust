use std::{marker::PhantomData, ops::Range};

use arrow2::trusted_len::TrustedLen;

use crate::values::{Column, Scalar};

use super::{ArgType, DataType, GenericMap, ValueType};

pub struct ArrayType<T: ArgType>(PhantomData<T>);

impl<T: ArgType> ValueType for ArrayType<T> {
    type Scalar = T::Column;
    type ScalarRef<'a> = T::Column;
    type Column = (T::Column, Vec<usize>);

    fn to_owned_scalar<'a>(scalar: Self::ScalarRef<'a>) -> Self::Scalar {
        scalar
    }

    fn to_scalar_ref<'a>(scalar: &'a Self::Scalar) -> Self::ScalarRef<'a> {
        scalar.clone()
    }
}

impl<T: ArgType> ArgType for ArrayType<T> {
    type ColumnIterator<'a> = ArrayIterator<'a, T>;
    type ColumnBuilder = (T::ColumnBuilder, Vec<usize>);

    fn data_type() -> DataType {
        DataType::Array(Box::new(T::data_type()))
    }

    fn try_downcast_scalar<'a>(scalar: &'a Scalar) -> Option<Self::ScalarRef<'a>> {
        match scalar {
            Scalar::Array(array) => T::try_downcast_column(array),
            _ => None,
        }
    }

    fn try_downcast_column<'a>(col: &'a Column) -> Option<Self::Column> {
        match col {
            Column::Array { array, offsets } => {
                Some((T::try_downcast_column(array)?, offsets.clone()))
            }
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

    fn column_len<'a>((_, offsets): &'a Self::Column) -> usize {
        offsets.len()
    }

    fn index_column<'a>((col, offsets): &'a Self::Column, index: usize) -> Self::ScalarRef<'a> {
        T::slice_column(col, offsets[index]..offsets[index + 1])
    }

    fn slice_column<'a>((col, offsets): &'a Self::Column, range: Range<usize>) -> Self::Column {
        (col.clone(), offsets[range].to_vec())
    }

    fn iter_column<'a>((col, offsets): &'a Self::Column) -> Self::ColumnIterator<'a> {
        ArrayIterator {
            col,
            offsets: offsets.windows(2),
        }
    }

    fn create_builder(_capacity: usize, generics: &GenericMap) -> Self::ColumnBuilder {
        (T::create_builder(0, generics), vec![0])
    }

    fn column_to_builder((col, offsets): Self::Column) -> Self::ColumnBuilder {
        (T::column_to_builder(col), offsets)
    }

    fn builder_len((_, offsets): &Self::ColumnBuilder) -> usize {
        offsets.len() - 1
    }

    fn push_item((builder, offsets): &mut Self::ColumnBuilder, item: Self::ScalarRef<'_>) {
        let other_col = T::column_to_builder(item);
        T::append_builder(builder, &other_col);
        let len = T::builder_len(builder);
        offsets.push(len);
    }

    fn push_default((builder, offsets): &mut Self::ColumnBuilder) {
        let len = T::builder_len(builder);
        offsets.push(len);
    }

    fn append_builder(
        (builder, offsets): &mut Self::ColumnBuilder,
        (other_builder, other_offsets): &Self::ColumnBuilder,
    ) {
        let end = offsets.last().cloned().unwrap();
        offsets.extend(other_offsets.iter().skip(1).map(|offset| offset + end));
        T::append_builder(builder, other_builder);
    }

    fn build_column((builder, offsets): Self::ColumnBuilder) -> Self::Column {
        // TODO: check that they have same length
        (T::build_column(builder), offsets)
    }

    fn build_scalar((builder, offsets): Self::ColumnBuilder) -> Self::Scalar {
        assert_eq!(offsets.len(), 2);
        T::slice_column(&T::build_column(builder), offsets[0]..offsets[1])
    }
}

pub struct ArrayIterator<'a, T: ArgType> {
    col: &'a T::Column,
    offsets: std::slice::Windows<'a, usize>,
}

impl<'a, T: ArgType> Iterator for ArrayIterator<'a, T> {
    type Item = T::Column;

    fn next(&mut self) -> Option<Self::Item> {
        self.offsets
            .next()
            .map(|range| T::slice_column(self.col, range[0]..range[1]))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.offsets.size_hint()
    }
}

unsafe impl<'a, T: ArgType> TrustedLen for ArrayIterator<'a, T> {}
