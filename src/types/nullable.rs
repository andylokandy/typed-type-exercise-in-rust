use std::{marker::PhantomData, ops::Range, sync::Arc};

use crate::values::{Column, Scalar};

use super::{ArgType, ColumnBuilder, ColumnViewer, DataType, ValueType};

pub struct NullableType<T: ValueType>(PhantomData<T>);

impl<T: ValueType> ValueType for NullableType<T>
where
    T::Scalar: Default,
{
    type Scalar = Option<T::Scalar>;
    type ScalarRef<'a> = Option<T::ScalarRef<'a>>;
    type Column = (T::Column, Vec<bool>);
    type ColumnRef<'a> = (T::ColumnRef<'a>, &'a [bool]);

    fn to_owned_scalar<'a>(scalar: Self::ScalarRef<'a>) -> Self::Scalar {
        scalar.map(T::to_owned_scalar)
    }

    fn to_owned_column<'a>(col: Self::ColumnRef<'a>) -> Self::Column {
        let (col, nulls) = col;
        (T::to_owned_column(col), nulls.to_vec())
    }

    fn to_scalar_ref<'a>(scalar: &'a Self::Scalar) -> Self::ScalarRef<'a> {
        scalar.as_ref().map(T::to_scalar_ref)
    }

    fn to_column_ref<'a>(col: &'a Self::Column) -> Self::ColumnRef<'a> {
        let (col, nulls) = col;
        (T::to_column_ref(col), &nulls)
    }
}

impl<T: ArgType> ArgType for NullableType<T>
where
    T::Scalar: Default,
{
    fn data_type() -> DataType {
        DataType::Nullable(Box::new(T::data_type()))
    }

    fn try_downcast_scalar<'a>(scalar: &'a Scalar) -> Option<Self::ScalarRef<'a>> {
        match scalar {
            Scalar::Null => Some(None),
            scalar => Some(Some(T::try_downcast_scalar(scalar)?)),
        }
    }

    fn try_downcast_column<'a>(col: &'a Column) -> Option<Self::ColumnRef<'a>> {
        match col {
            Column::Nullable { column, nulls } => Some((T::try_downcast_column(column)?, nulls)),
            _ => None,
        }
    }

    fn upcast_scalar(scalar: Self::Scalar) -> Scalar {
        match scalar {
            Some(scalar) => T::upcast_scalar(scalar),
            None => Scalar::Null,
        }
    }

    fn upcast_column((col, nulls): Self::Column) -> Column {
        Column::Nullable {
            column: Box::new(T::upcast_column(col)),
            nulls,
        }
    }
}

impl<T: ColumnViewer> ColumnViewer for NullableType<T>
where
    T::Scalar: Default,
{
    type ScalarBorrow<'a> = Option<T::ScalarBorrow<'a>>;
    type ColumnIterator<'a> = NullableIterator<'a, T>;

    fn scalar_borrow_to_ref<'a>(scalar: &'a Self::ScalarBorrow<'a>) -> Self::ScalarRef<'a> {
        scalar.as_ref().map(T::scalar_borrow_to_ref)
    }

    fn column_len<'a>((_, nulls): Self::ColumnRef<'a>) -> usize {
        nulls.len()
    }

    fn index_column<'a>((col, nulls): Self::ColumnRef<'a>, index: usize) -> Self::ScalarBorrow<'a> {
        let scalar = T::index_column(col, index);
        if nulls[index] {
            Some(scalar)
        } else {
            None
        }
    }

    fn slice_column<'a>(
        (col, nulls): Self::ColumnRef<'a>,
        range: Range<usize>,
    ) -> Self::ColumnRef<'a> {
        (T::slice_column(col, range.clone()), &nulls[range])
    }

    fn iter_column<'a>((col, nulls): Self::ColumnRef<'a>) -> Self::ColumnIterator<'a> {
        NullableIterator {
            iter: T::iter_column(col),
            nulls: nulls.iter(),
        }
    }
}

pub struct NullableIterator<'a, T: ColumnViewer> {
    iter: T::ColumnIterator<'a>,
    nulls: std::slice::Iter<'a, bool>,
}

impl<'a, T: ColumnViewer> Iterator for NullableIterator<'a, T> {
    type Item = Option<<T as ColumnViewer>::ScalarBorrow<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().zip(self.nulls.next()).map(
            |(scalar, is_null)| {
                if *is_null {
                    None
                } else {
                    Some(scalar)
                }
            },
        )
    }
}

impl<T: ColumnBuilder> ColumnBuilder for NullableType<T>
where
    T::Scalar: Default,
{
    fn empty_column(capacity: usize) -> Self::Column {
        (T::empty_column(capacity), Vec::with_capacity(capacity))
    }

    fn push_column((mut col, mut nulls): Self::Column, item: Self::Scalar) -> Self::Column {
        match item {
            Some(scalar) => {
                col = T::push_column(col, scalar);
                nulls.push(false);
            }
            None => {
                col = T::push_column(col, T::Scalar::default());
                nulls.push(true);
            }
        }
        (col, nulls)
    }

    fn append_column(
        (col, mut nulls): Self::Column,
        (other_col, mut other_nulls): Self::Column,
    ) -> Self::Column {
        let col = T::append_column(col, other_col);
        nulls.append(&mut other_nulls);
        (col, nulls)
    }
}
