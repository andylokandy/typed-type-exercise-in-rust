use std::{marker::PhantomData, sync::Arc};

use crate::values::{Column, Scalar};

use super::{DataType, Type};

pub struct NullableType<T: Type>(PhantomData<T>);

impl<T: Type> Type for NullableType<T> {
    type WrapNullable = Self;
    type Scalar = Option<T::Scalar>;
    type ScalarRef<'a> = Option<T::ScalarRef<'a>>;
    type Column = (T::Column, Vec<bool>);
    type ColumnRef<'a> = (T::ColumnRef<'a>, &'a [bool]);
    type ColumnIterator<'a> = NullableIterator<'a, T>;

    fn data_type() -> DataType {
        DataType::Nullable(Box::new(T::data_type()))
    }

    fn try_downcast_scalar<'a>(scalar: &'a Scalar) -> Option<Self::ScalarRef<'a>> {
        match scalar {
            Scalar::Null => Some(None),
            scalar => Some(Some(T::try_downcast_scalar(scalar)?)),
        }
    }

    fn try_downcast_column<'a>(col: &'a Arc<Column>) -> Option<Self::ColumnRef<'a>> {
        match &**col {
            Column::Nullable(col, nulls) => Some((T::try_downcast_column(col)?, nulls)),
            _ => None,
        }
    }

    fn upcast_scalar(scalar: Self::Scalar) -> Scalar {
        match scalar {
            Some(scalar) => T::upcast_scalar(scalar),
            None => Scalar::Null,
        }
    }

    fn upcast_column(col: Self::Column) -> Arc<Column> {
        let (col, nulls) = col;
        Arc::new(Column::Nullable(T::upcast_column(col), nulls))
    }

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

    fn index_column<'a>((col, nulls): Self::ColumnRef<'a>, index: usize) -> Self::ScalarRef<'a> {
        if nulls[index] {
            None
        } else {
            Some(T::index_column(col, index))
        }
    }

    fn iter_column<'a>((col, nulls): Self::ColumnRef<'a>) -> Self::ColumnIterator<'a> {
        NullableIterator {
            column: col,
            nulls: &nulls,
            len: nulls.len(),
            index: 0,
        }
    }

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
}

pub struct NullableIterator<'a, T: Type> {
    column: T::ColumnRef<'a>,
    nulls: &'a [bool],
    len: usize,
    index: usize,
}

impl<'a, T: Type> Iterator for NullableIterator<'a, T> {
    type Item = Option<T::ScalarRef<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let scalar = if self.nulls[self.index] {
                None
            } else {
                Some(T::index_column(self.column.clone(), self.index))
            };
            self.index += 1;
            Some(scalar)
        } else {
            None
        }
    }
}
