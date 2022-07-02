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
    Tuple(Vec<Scalar>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default, EnumAsInner)]
pub enum ScalarRef<'a> {
    #[default]
    Null,
    EmptyArray,
    Int8(i8),
    Int16(i16),
    UInt8(u8),
    UInt16(u16),
    Boolean(bool),
    String(&'a str),
    Array(ColumnRef<'a>),
    Tuple(Vec<ScalarRef<'a>>),
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
    Tuple {
        fields: Vec<Column>,
        len: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnRef<'a> {
    pub column: &'a Column,
    pub range: Range<usize>,
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

impl Scalar {
    pub fn as_ref(&self) -> ScalarRef {
        match self {
            Scalar::Null => ScalarRef::Null,
            Scalar::EmptyArray => ScalarRef::EmptyArray,
            Scalar::Int8(i) => ScalarRef::Int8(*i),
            Scalar::Int16(i) => ScalarRef::Int16(*i),
            Scalar::UInt8(i) => ScalarRef::UInt8(*i),
            Scalar::UInt16(i) => ScalarRef::UInt16(*i),
            Scalar::Boolean(b) => ScalarRef::Boolean(*b),
            Scalar::String(s) => ScalarRef::String(s.as_str()),
            Scalar::Array(col) => ScalarRef::Array(col.slice_all()),
            Scalar::Tuple(fields) => ScalarRef::Tuple(fields.iter().map(Scalar::as_ref).collect()),
        }
    }
}

impl<'a> ScalarRef<'a> {
    pub fn to_owned(&self) -> Scalar {
        match self {
            ScalarRef::Null => Scalar::Null,
            ScalarRef::EmptyArray => Scalar::EmptyArray,
            ScalarRef::Int8(i) => Scalar::Int8(*i),
            ScalarRef::Int16(i) => Scalar::Int16(*i),
            ScalarRef::UInt8(i) => Scalar::UInt8(*i),
            ScalarRef::UInt16(i) => Scalar::UInt16(*i),
            ScalarRef::Boolean(b) => Scalar::Boolean(*b),
            ScalarRef::String(s) => Scalar::String(s.to_string()),
            ScalarRef::Array(col) => Scalar::Array(col.to_owned()),
            ScalarRef::Tuple(fields) => {
                Scalar::Tuple(fields.iter().map(ScalarRef::to_owned).collect())
            }
        }
    }

    pub fn repeat(&self, n: usize) -> Column {
        match self {
            ScalarRef::Null => Column::Null { len: n },
            ScalarRef::EmptyArray => Column::EmptyArray { len: n },
            ScalarRef::Int8(i) => Column::Int8(vec![*i; n]),
            ScalarRef::Int16(i) => Column::Int16(vec![*i; n]),
            ScalarRef::UInt8(i) => Column::UInt8(vec![*i; n]),
            ScalarRef::UInt16(i) => Column::UInt16(vec![*i; n]),
            ScalarRef::Boolean(b) => Column::Boolean(vec![*b; n]),
            ScalarRef::String(s) => Column::String(vec![s.to_string(); n]),
            ScalarRef::Array(col) => Column::Array {
                array: Box::new(col.to_owned()),
                offsets: vec![0..col.len(); n],
            },
            ScalarRef::Tuple(fields) => Column::Tuple {
                fields: fields.iter().map(|field| field.repeat(n)).collect(),
                len: n,
            },
        }
    }
}

impl Column {
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
            Column::Tuple { len, .. } => *len,
        }
    }

    pub fn slice(&self, range: Range<usize>) -> ColumnRef {
        ColumnRef {
            column: self,
            range,
        }
    }

    pub fn slice_all(&self) -> ColumnRef {
        ColumnRef {
            column: self,
            range: 0..self.len(),
        }
    }

    pub fn create_and_fill_default(ty: &DataType, len: usize) -> Column {
        match ty {
            DataType::Null => Column::Null { len },
            DataType::EmptyArray => Column::EmptyArray { len },
            DataType::Boolean => Column::Boolean(vec![false; len]),
            DataType::String => Column::String(vec![String::new(); len]),
            DataType::UInt8 => Column::UInt8(vec![0; len]),
            DataType::UInt16 => Column::UInt16(vec![0; len]),
            DataType::Int8 => Column::Int8(vec![0; len]),
            DataType::Int16 => Column::Int16(vec![0; len]),
            DataType::Nullable(ty) => Column::Nullable {
                column: Box::new(Self::create_and_fill_default(ty, len)),
                nulls: vec![false; len],
            },
            DataType::Array(ty) => Column::Array {
                array: Box::new(Self::create_and_fill_default(ty, 0)),
                offsets: vec![0..0; len],
            },
            DataType::Tuple(fields) => Column::Tuple {
                fields: fields
                    .iter()
                    .map(|field| Self::create_and_fill_default(field, len))
                    .collect(),
                len,
            },
            DataType::Generic(_) => unreachable!(),
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
            (Column::Tuple { fields, len }, Scalar::Tuple(value)) => {
                assert_eq!(fields.len(), value.len());
                for (field, scalar) in fields.iter_mut().zip(value.iter()) {
                    field.push(scalar.clone());
                }
                *len += 1;
            }
            (c, s) => unreachable!("{c:?} {s:?}"),
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
            Column::Tuple { fields, len } => {
                for field in fields {
                    field.push_default();
                }
                *len += 1;
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
            (
                Column::Tuple { fields, len },
                Column::Tuple {
                    fields: other_fields,
                    len: other_len,
                },
            ) => {
                assert_eq!(fields.len(), other_fields.len());
                for (field, other_field) in fields.iter_mut().zip(other_fields.iter()) {
                    field.append(other_field);
                }
                *len += other_len;
            }
            _ => unreachable!(),
        }
    }
}

impl<'a> ColumnRef<'a> {
    pub fn index(&self, index: usize) -> ScalarRef<'a> {
        if index >= self.range.end {
            panic!(
                "Column reference index {index} out of range 0..{}",
                self.range.end
            )
        }
        let index = self.range.start + index;
        match self.column {
            Column::Null { .. } => ScalarRef::Null,
            Column::EmptyArray { .. } => ScalarRef::EmptyArray,
            Column::Int8(values) => ScalarRef::Int8(values[index]),
            Column::Int16(values) => ScalarRef::Int16(values[index]),
            Column::UInt8(values) => ScalarRef::UInt8(values[index]),
            Column::UInt16(values) => ScalarRef::UInt16(values[index]),
            Column::Boolean(values) => ScalarRef::Boolean(values[index]),
            Column::String(values) => ScalarRef::String(&values[index]),
            Column::Array { array, offsets } => {
                ScalarRef::Array(array.slice(offsets[index].clone()))
            }
            Column::Nullable { column: col, nulls } => {
                if nulls.get(index).cloned().unwrap_or(false) {
                    ScalarRef::Null
                } else {
                    col.slice_all().index(index)
                }
            }
            Column::Tuple { fields, .. } => {
                let fields = fields
                    .iter()
                    .map(|field| field.slice_all().index(index))
                    .collect();
                ScalarRef::Tuple(fields)
            }
        }
    }

    pub fn len(&self) -> usize {
        self.range.end - self.range.start
    }

    pub fn slice(&self, range: Range<usize>) -> ColumnRef<'a> {
        if range.end > self.len() {
            panic!(
                "Column reference range {:?} out of range 0..{}",
                range,
                self.len()
            )
        } else {
            ColumnRef {
                column: self.column,
                range: (self.range.start + range.start)..(self.range.start + range.end),
            }
        }
    }

    pub fn iter(&self) -> ColumnIterator<'a> {
        ColumnIterator {
            column: self.clone(),
            index: 0,
        }
    }

    pub fn to_owned(&self) -> Column {
        match self.column {
            Column::Null { len } => {
                if self.range.end > *len {
                    panic!(
                        "Column index {end} out of range 0..{len}",
                        end = self.range.end
                    )
                } else {
                    Column::Null {
                        len: self.range.end - self.range.start,
                    }
                }
            }
            Column::EmptyArray { len } => {
                if self.range.end > *len {
                    panic!(
                        "Column index {end} out of range 0..{len}",
                        end = self.range.end
                    )
                } else {
                    Column::EmptyArray {
                        len: self.range.end - self.range.start,
                    }
                }
            }
            Column::Int8(values) => Column::Int8(values[self.range.clone()].to_vec()),
            Column::Int16(values) => Column::Int16(values[self.range.clone()].to_vec()),
            Column::UInt8(values) => Column::UInt8(values[self.range.clone()].to_vec()),
            Column::UInt16(values) => Column::UInt16(values[self.range.clone()].to_vec()),
            Column::Boolean(values) => Column::Boolean(values[self.range.clone()].to_vec()),
            Column::String(values) => Column::String(values[self.range.clone()].to_vec()),
            Column::Array { array, offsets } => {
                let start = offsets
                    .get(self.range.start)
                    .map(|range| range.start)
                    .unwrap_or(0);
                let end = self
                    .range
                    .end
                    .checked_sub(1)
                    .and_then(|end| offsets.get(end))
                    .map(|range| range.end)
                    .unwrap_or(0);
                Column::Array {
                    array: Box::new(array.slice(start..end).to_owned()),
                    offsets: offsets[self.range.clone()]
                        .iter()
                        .map(|range| range.start - start..range.end - start)
                        .collect(),
                }
            }
            Column::Nullable { column, nulls } => Column::Nullable {
                column: Box::new(column.slice(self.range.clone()).to_owned()),
                nulls: nulls[self.range.clone()].to_vec(),
            },
            Column::Tuple { fields, len } => {
                if self.range.end > *len {
                    panic!(
                        "Column index {end} out of range 0..{len}",
                        end = self.range.end
                    )
                } else {
                    let fields = fields
                        .iter()
                        .map(|field| field.slice(self.range.clone()).to_owned())
                        .collect();
                    Column::Tuple {
                        fields,
                        len: self.range.end - self.range.start,
                    }
                }
            }
        }
    }
}

impl Default for Column {
    fn default() -> Self {
        Column::Null { len: 0 }
    }
}

pub struct ColumnIterator<'a> {
    column: ColumnRef<'a>,
    index: usize,
}

impl<'a> Iterator for ColumnIterator<'a> {
    type Item = ScalarRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.column.len() {
            let item = self.column.index(self.index);
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}
