use std::ops::Range;

use enum_as_inner::EnumAsInner;

use crate::types::*;

#[derive(EnumAsInner)]
pub enum Value<T: ValueType> {
    Scalar(T::Scalar),
    Column(T::Column),
}

#[derive(EnumAsInner)]
pub enum ValueRef<'a, T: ValueType> {
    Scalar(T::ScalarRef<'a>),
    Column(T::ColumnRef<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default, EnumAsInner)]
pub enum Scalar {
    #[default]
    Null,
    EmptyArray,
    Int8(i8),
    Int16(i16),
    UInt8(u8),
    UInt16(u16),
    Boolean(bool),
    String(String),
    Array(Column),
}

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum Column {
    Null {
        len: usize,
    },
    EmptyArray {
        len: usize,
    },
    Int8(Vec<i8>),
    Int16(Vec<i16>),
    UInt8(Vec<u8>),
    UInt16(Vec<u16>),
    Boolean(Vec<bool>),
    String(Vec<String>),
    Array {
        array: Box<Column>,
        offsets: Vec<Range<usize>>,
    },
    Nullable {
        column: Box<Column>,
        nulls: Vec<bool>,
    },
}

impl<'a, T: ValueType> ValueRef<'a, T> {
    pub fn to_owned(self) -> Value<T> {
        match self {
            ValueRef::Scalar(scalar) => Value::Scalar(T::to_owned_scalar(scalar)),
            ValueRef::Column(col) => Value::Column(T::to_owned_column(col)),
        }
    }
}

impl<'a, T: ValueType> Value<T> {
    pub fn as_ref(&'a self) -> ValueRef<'a, T> {
        match self {
            Value::Scalar(scalar) => ValueRef::Scalar(T::to_scalar_ref(scalar)),
            Value::Column(col) => ValueRef::Column(T::to_column_ref(col)),
        }
    }
}

impl<'a, T: ValueType> Clone for ValueRef<'a, T> {
    fn clone(&self) -> Self {
        match self {
            ValueRef::Scalar(scalar) => ValueRef::Scalar(scalar.clone()),
            ValueRef::Column(col) => ValueRef::Column(col.clone()),
        }
    }
}

impl Column {
    pub fn index(&self, index: usize) -> Scalar {
        match self {
            Column::Null { len } => {
                if index >= *len {
                    panic!("Column index {index} out of range 0..{len}")
                } else {
                    Scalar::Null
                }
            }
            Column::EmptyArray { len } => {
                if index >= *len {
                    panic!("Column index {index} out of range 0..{len}")
                } else {
                    Scalar::EmptyArray
                }
            }
            Column::Int8(values) => Scalar::Int8(values[index]),
            Column::Int16(values) => Scalar::Int16(values[index]),
            Column::UInt8(values) => Scalar::UInt8(values[index]),
            Column::UInt16(values) => Scalar::UInt16(values[index]),
            Column::Boolean(values) => Scalar::Boolean(values[index]),
            Column::String(values) => Scalar::String(values[index].clone()),
            Column::Array { array, offsets } => Scalar::Array(array.slice(offsets[index].clone())),
            Column::Nullable { column: col, nulls } => {
                if nulls.get(index).cloned().unwrap_or(false) {
                    Scalar::Null
                } else {
                    col.index(index)
                }
            }
        }
    }

    pub fn slice(&self, range: Range<usize>) -> Column {
        match self {
            Column::Null { len } => {
                if range.end > *len {
                    panic!("Column index {end} out of range 0..{len}", end = range.end)
                } else {
                    Column::Null {
                        len: range.end - range.start,
                    }
                }
            }
            Column::EmptyArray { len } => {
                if range.end > *len {
                    panic!("Column index {end} out of range 0..{len}", end = range.end)
                } else {
                    Column::EmptyArray {
                        len: range.end - range.start,
                    }
                }
            }
            Column::Int8(values) => Column::Int8(values[range].to_vec()),
            Column::Int16(values) => Column::Int16(values[range].to_vec()),
            Column::UInt8(values) => Column::UInt8(values[range].to_vec()),
            Column::UInt16(values) => Column::UInt16(values[range].to_vec()),
            Column::Boolean(values) => Column::Boolean(values[range].to_vec()),
            Column::String(values) => Column::String(values[range].to_vec()),
            Column::Array { array, offsets } => {
                let start = offsets
                    .get(range.start)
                    .map(|range| range.start)
                    .unwrap_or(0);
                let end = range
                    .end
                    .checked_sub(1)
                    .and_then(|end| offsets.get(end))
                    .map(|range| range.end)
                    .unwrap_or(0);
                Column::Array {
                    array: Box::new(array.slice(start..end)),
                    offsets: offsets[range]
                        .iter()
                        .map(|range| range.start - start..range.end - start)
                        .collect(),
                }
            }
            Column::Nullable { column, nulls } => Column::Nullable {
                column: Box::new(column.slice(range.clone())),
                nulls: nulls[range].to_vec(),
            },
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Column::Null { len } => *len,
            Column::EmptyArray { len } => *len,
            Column::Int8(col) => col.len(),
            Column::Int16(col) => col.len(),
            Column::UInt8(col) => col.len(),
            Column::UInt16(col) => col.len(),
            Column::Boolean(col) => col.len(),
            Column::String(col) => col.len(),
            Column::Array { array: _, offsets } => offsets.len(),
            Column::Nullable { column: _, nulls } => nulls.len(),
        }
    }

    pub fn iter(&self) -> ColumnIterator {
        ColumnIterator {
            len: self.len(),
            column: self,
            index: 0,
        }
    }

    pub fn push(&mut self, item: Scalar) {
        match (self, item) {
            (Column::Null { len }, Scalar::Null) => *len += 1,
            (Column::EmptyArray { len }, Scalar::EmptyArray) => *len += 1,
            (Column::Int8(col), Scalar::Int8(value)) => col.push(value),
            (Column::Int16(col), Scalar::Int16(value)) => col.push(value),
            (Column::UInt8(col), Scalar::UInt8(value)) => col.push(value),
            (Column::UInt16(col), Scalar::UInt16(value)) => col.push(value),
            (Column::Boolean(col), Scalar::Boolean(value)) => col.push(value),
            (Column::String(col), Scalar::String(value)) => col.push(value),
            (Column::Array { array, offsets }, Scalar::Array(value)) => {
                let start = array.len();
                let end = start + value.len();
                offsets.push(start..end);
                array.append(&value);
            }
            (Column::Nullable { column, nulls }, Scalar::Null) => {
                column.push_default();
                nulls.push(true);
            }
            (Column::Nullable { column, nulls }, scalar) => {
                column.push(scalar);
                nulls.push(false);
            }
            _ => unreachable!(),
        }
    }

    pub fn push_default(&mut self) {
        match self {
            Column::Null { len } => *len += 1,
            Column::EmptyArray { len } => *len += 1,
            Column::Int8(col) => col.push(0),
            Column::Int16(col) => col.push(0),
            Column::UInt8(col) => col.push(0),
            Column::UInt16(col) => col.push(0),
            Column::Boolean(col) => col.push(false),
            Column::String(col) => col.push(String::new()),
            Column::Array { array, offsets } => {
                let start = array.len();
                offsets.push(start..start);
            }
            Column::Nullable { column, nulls } => {
                column.push_default();
                nulls.push(true);
            }
        }
    }

    pub fn append(&mut self, other: &Column) {
        match (self, other) {
            (Column::Null { len }, Column::Null { len: other_len }) => *len += other_len,
            (Column::EmptyArray { len }, Column::EmptyArray { len: other_len }) => {
                *len += other_len
            }
            (Column::Int8(col), Column::Int8(other_col)) => col.extend_from_slice(other_col),
            (Column::Int16(col), Column::Int16(other_col)) => col.extend_from_slice(other_col),
            (Column::UInt8(col), Column::UInt8(other_col)) => col.extend_from_slice(other_col),
            (Column::UInt16(col), Column::UInt16(other_col)) => col.extend_from_slice(other_col),
            (Column::Boolean(col), Column::Boolean(other_col)) => col.extend_from_slice(other_col),
            (Column::String(col), Column::String(other_col)) => col.extend_from_slice(other_col),
            (
                Column::Array { array, offsets },
                Column::Array {
                    array: other_array,
                    offsets: other_offsets,
                },
            ) => {
                let base = offsets.last().map(|range| range.end).unwrap_or(0);
                offsets.extend(
                    other_offsets
                        .iter()
                        .map(|range| (range.start + base)..(range.end + base)),
                );
                array.append(other_array);
            }
            (
                Column::Nullable { column, nulls },
                Column::Nullable {
                    column: other_column,
                    nulls: other_nulls,
                },
            ) => {
                column.append(other_column);
                nulls.extend_from_slice(other_nulls);
            }
            _ => unreachable!(),
        }
    }
}

impl Default for Column {
    fn default() -> Self {
        Column::Null { len: 0 }
    }
}

pub struct ColumnIterator<'a> {
    column: &'a Column,
    len: usize,
    index: usize,
}

impl<'a> Iterator for ColumnIterator<'a> {
    type Item = Scalar;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let item = self.column.index(self.index);
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

// #[macro_export]
// macro_rules! dispatch_column_type {
//     ($ty:expr, $expr:expr) => {{
//         match_template::match_template! {
//             TYPE = [
//                 Boolean => $crate::types::BooleanType,
//             ],
//             match $api_version {
//                 $crate::types::DataType::TYPE => $e,
//             }
//         }
//     }};
// }
