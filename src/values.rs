use std::sync::Arc;

use enum_as_inner::EnumAsInner;

#[derive(Debug, Clone, PartialEq, Eq, Default, EnumAsInner)]
pub enum Scalar {
    #[default]
    Null,
    Int8(i8),
    Int16(i16),
    UInt8(u8),
    UInt16(u16),
    Boolean(bool),
    String(String),
    Array(Array),
}

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum Column {
    Int8(Vec<i8>),
    Int16(Vec<i16>),
    UInt8(Vec<u8>),
    UInt16(Vec<u16>),
    Boolean(Vec<bool>),
    String(Vec<String>),
    Array(Vec<Array>),
    Nullable(Box<Column>, Vec<bool>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default, EnumAsInner)]
pub enum Array {
    #[default]
    Empty,
    Any(Vec<Scalar>),
    Uniform(Arc<Column>),
}

impl Column {
    pub fn get(&self, index: usize) -> Scalar {
        match self {
            Column::Int8(values) => Scalar::Int8(values[index]),
            Column::Int16(values) => Scalar::Int16(values[index]),
            Column::UInt8(values) => Scalar::UInt8(values[index]),
            Column::UInt16(values) => Scalar::UInt16(values[index]),
            Column::Boolean(values) => Scalar::Boolean(values[index]),
            Column::String(values) => Scalar::String(values[index].clone()),
            Column::Array(_) => todo!(),
            Column::Nullable(column, nulls) => {
                if nulls.get(index).cloned().unwrap_or(false) {
                    Scalar::Null
                } else {
                    column.get(index)
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Column::Int8(col) => col.len(),
            Column::Int16(col) => col.len(),
            Column::UInt8(col) => col.len(),
            Column::UInt16(col) => col.len(),
            Column::Boolean(col) => col.len(),
            Column::String(col) => col.len(),
            Column::Array(_) => todo!(),
            Column::Nullable(col, _) => col.len(),
        }
    }

    // pub fn push(self, item: Scalar) -> Option<Self> {
    //     match (self, item) {
    //         (Column::Int8(mut col), Scalar::Int8(item)) => {
    //             col.push(item);
    //             Some(Column::Int8(item))
    //         }
    //         (Column::Int16(mut col), Scalar::Int16(item)) => {
    //             col.push(item);
    //             Some(Column::Int16(item))
    //         }
    //         (Column::UInt8(mut col), Scalar::UInt8(item)) => {
    //             col.push(item);
    //             Some(Column::UInt8(item))
    //         }
    //         (Column::UInt16(mut col), Scalar::UInt16(item)) => {
    //             col.push(item);
    //             Some(Column::UInt16(item))
    //         }
    //         (Column::Boolean(mut col), Scalar::Boolean(item)) => {
    //             col.push(item);
    //             Some(Column::Boolean(item))
    //         }
    //         (Column::String(mut col), Scalar::String(item)) => {
    //             col.push(item);
    //             Some(Column::String(item))
    //         }
    //         (Column::Array(mut col), Scalar::Array(item)) => {
    //             col.push(item);
    //             Some(Column::Array(item))
    //         },
    //         (Column::Nullable(col, mut nulls), Scalar::Null) => {
    //             nulls.push(true);
    //             Some(Column::Nullable(col.push_default(), nulls))
    //         }
    //         (Column::Nullable(col, mut nulls), scalar) => {
    //             nulls.push(false);
    //             Some(Column::Nullable(col.push(item)?, nulls))
    //         }
    //         _ => None,
    //     }
    // }

    // pub fn push_default(self) -> Self {
    //     match self {
    //         Column::Int8(mut col) => {
    //             col.push(0);
    //             Column::Int8(col)
    //         }
    //         Column::Int16(mut col) => {
    //             col.push(0);
    //             Column::Int16(col)
    //         }
    //         Column::UInt8(mut col) => {
    //             col.push(0);
    //             Column::UInt8(col)
    //         }
    //         Column::UInt16(mut col) => {
    //             col.push(0);
    //             Column::UInt16(col)
    //         }
    //         Column::Boolean(mut col) => {
    //             col.push(false);
    //             Column::Boolean(col)
    //         }
    //         Column::String(mut col) => {
    //             col.push(String::new());
    //             Column::String(col)
    //         }
    //         Column::Array(mut col) => {
    //             col.push(Array::Empty);
    //             Column::Array(col)
    //         }
    //         Column::Nullable(mut col, mut nulls) => {
    //             col.push_default();
    //             nulls.push(true);
    //             Column::Nullable(col, nulls)
    //         }
    //     }
    // }

    pub fn iter(self: Arc<Self>) -> ColumnIter {
        ColumnIter {
            len: self.len(),
            column: self,
            index: 0,
        }
    }
}

pub struct ColumnIter {
    column: Arc<Column>,
    len: usize,
    index: usize,
}

impl Iterator for ColumnIter {
    type Item = Scalar;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let item = self.column.get(self.index);
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}
