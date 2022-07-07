use std::{marker::PhantomData, ops::Range};

use arrow2::{
    bitmap::{Bitmap, MutableBitmap},
    trusted_len::TrustedLen,
};

use crate::{
    util::bitmap_into_mut,
    values::{Column, Scalar},
};

use super::{ArgType, DataType, GenericMap, ValueType};

pub struct NullableType<T: ValueType>(PhantomData<T>);

impl<T: ValueType> ValueType for NullableType<T> {
    type Scalar = Option<T::Scalar>;
    type ScalarRef<'a> = Option<T::ScalarRef<'a>>;
    type Column = (T::Column, Bitmap);

    fn to_owned_scalar<'a>(scalar: Self::ScalarRef<'a>) -> Self::Scalar {
        scalar.map(T::to_owned_scalar)
    }

    fn to_scalar_ref<'a>(scalar: &'a Self::Scalar) -> Self::ScalarRef<'a> {
        scalar.as_ref().map(T::to_scalar_ref)
    }
}

impl<T: ArgType> ArgType for NullableType<T> {
    type ColumnIterator<'a> = NullableIterator<'a, T>;
    type ColumnBuilder = (T::ColumnBuilder, MutableBitmap);

    fn data_type() -> DataType {
        DataType::Nullable(Box::new(T::data_type()))
    }

    fn try_downcast_scalar<'a>(scalar: &'a Scalar) -> Option<Self::ScalarRef<'a>> {
        match scalar {
            Scalar::Null => Some(None),
            scalar => Some(Some(T::try_downcast_scalar(scalar)?)),
        }
    }

    fn try_downcast_column<'a>(col: &'a Column) -> Option<Self::Column> {
        match col {
            Column::Nullable { column, validity } => {
                Some((T::try_downcast_column(column)?, validity.clone()))
            }
            _ => None,
        }
    }

    fn upcast_scalar(scalar: Self::Scalar) -> Scalar {
        match scalar {
            Some(scalar) => T::upcast_scalar(scalar),
            None => Scalar::Null,
        }
    }

    fn upcast_column((col, validity): Self::Column) -> Column {
        Column::Nullable {
            column: Box::new(T::upcast_column(col)),
            validity,
        }
    }

    fn column_len<'a>((_, validity): &'a Self::Column) -> usize {
        validity.len()
    }

    fn index_column<'a>((col, validity): &'a Self::Column, index: usize) -> Self::ScalarRef<'a> {
        let scalar = T::index_column(col, index);
        if validity.get(index).unwrap() {
            Some(scalar)
        } else {
            None
        }
    }

    fn slice_column<'a>((col, validity): &'a Self::Column, range: Range<usize>) -> Self::Column {
        (T::slice_column(col, range), validity.clone())
    }

    fn iter_column<'a>((col, validity): &'a Self::Column) -> Self::ColumnIterator<'a> {
        NullableIterator {
            iter: T::iter_column(col),
            validity: validity.iter(),
        }
    }

    fn create_builder(capacity: usize, generics: &GenericMap) -> Self::ColumnBuilder {
        (
            T::create_builder(capacity, generics),
            MutableBitmap::with_capacity(capacity),
        )
    }

    fn column_to_builder((col, validity): Self::Column) -> Self::ColumnBuilder {
        (T::column_to_builder(col), bitmap_into_mut(validity))
    }

    fn builder_len((_, validity): &Self::ColumnBuilder) -> usize {
        validity.len()
    }

    fn push_item((col, validity): &mut Self::ColumnBuilder, item: Self::ScalarRef<'_>) {
        match item {
            Some(scalar) => {
                T::push_item(col, scalar);
                validity.push(true);
            }
            None => {
                T::push_default(col);
                validity.push(false);
            }
        }
    }

    fn push_default((col, validity): &mut Self::ColumnBuilder) {
        T::push_default(col);
        validity.push(false);
    }

    fn append_builder(
        (col, validity): &mut Self::ColumnBuilder,
        (other_col, other_nulls): &Self::ColumnBuilder,
    ) {
        T::append_builder(col, other_col);
        validity.extend_from_slice(other_nulls.as_slice(), 0, other_nulls.len());
    }

    fn build_column((col, validity): Self::ColumnBuilder) -> Self::Column {
        // TODO: check that they have same length
        (T::build_column(col), validity.into())
    }

    fn build_scalar((col, validity): Self::ColumnBuilder) -> Self::Scalar {
        assert_eq!(validity.len(), 1);
        if validity.get(0) {
            Some(T::build_scalar(col))
        } else {
            None
        }
    }
}

pub struct NullableIterator<'a, T: ArgType> {
    iter: T::ColumnIterator<'a>,
    validity: arrow2::bitmap::utils::BitmapIter<'a>,
}

impl<'a, T: ArgType> Iterator for NullableIterator<'a, T> {
    type Item = Option<T::ScalarRef<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().zip(self.validity.next()).map(
            |(scalar, is_null)| {
                if is_null {
                    None
                } else {
                    Some(scalar)
                }
            },
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        assert_eq!(self.iter.size_hint(), self.validity.size_hint());
        self.validity.size_hint()
    }
}

unsafe impl<'a, T: ArgType> TrustedLen for NullableIterator<'a, T> {}
