use std::ops::Range;

use arrow2::{
    bitmap::{Bitmap, MutableBitmap},
    buffer::Buffer,
    trusted_len::TrustedLen,
};
use enum_as_inner::EnumAsInner;

use crate::{
    types::*,
    util::{append_bitmap, bitmap_into_mut, buffer_into_mut, constant_bitmap},
};

#[derive(EnumAsInner)]
pub enum Value<T: ValueType> {
    Scalar(T::Scalar),
    Column(T::Column),
}

#[derive(EnumAsInner)]
pub enum ValueRef<'a, T: ValueType> {
    Scalar(T::ScalarRef<'a>),
    Column(T::Column),
}

#[derive(Debug, Clone, Default, EnumAsInner)]
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

#[derive(Debug, Clone, Default, EnumAsInner)]
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
    Array(Column),
    Tuple(Vec<ScalarRef<'a>>),
}

#[derive(Debug, Clone, EnumAsInner)]
pub enum Column {
    Null {
        len: usize,
    },
    EmptyArray {
        len: usize,
    },
    Int8(Buffer<i8>),
    Int16(Buffer<i16>),
    UInt8(Buffer<u8>),
    UInt16(Buffer<u16>),
    Boolean(Bitmap),
    String(Vec<String>),
    Array {
        array: Box<Column>,
        offsets: Vec<Range<usize>>,
    },
    Nullable {
        column: Box<Column>,
        validity: Bitmap,
    },
    Tuple {
        fields: Vec<Column>,
        len: usize,
    },
}

#[derive(Debug, Clone, EnumAsInner)]
pub enum ColumnBuilder {
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
    Boolean(MutableBitmap),
    String(Vec<String>),
    Array {
        array: Box<ColumnBuilder>,
        offsets: Vec<Range<usize>>,
    },
    Nullable {
        column: Box<ColumnBuilder>,
        validity: MutableBitmap,
    },
    Tuple {
        fields: Vec<ColumnBuilder>,
        len: usize,
    },
}

impl<'a, T: ValueType> ValueRef<'a, T> {
    pub fn to_owned(self) -> Value<T> {
        match self {
            ValueRef::Scalar(scalar) => Value::Scalar(T::to_owned_scalar(scalar)),
            ValueRef::Column(col) => Value::Column(col),
        }
    }
}

impl<'a, T: ValueType> Value<T> {
    pub fn as_ref(&'a self) -> ValueRef<'a, T> {
        match self {
            Value::Scalar(scalar) => ValueRef::Scalar(T::to_scalar_ref(scalar)),
            Value::Column(col) => ValueRef::Column(col.clone()),
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
            Scalar::Array(col) => ScalarRef::Array(col.clone()),
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
            ScalarRef::Array(col) => Scalar::Array(col.clone()),
            ScalarRef::Tuple(fields) => {
                Scalar::Tuple(fields.iter().map(ScalarRef::to_owned).collect())
            }
        }
    }

    pub fn repeat(&self, n: usize) -> ColumnBuilder {
        match self {
            ScalarRef::Null => ColumnBuilder::Null { len: n },
            ScalarRef::EmptyArray => ColumnBuilder::EmptyArray { len: n },
            ScalarRef::Int8(i) => ColumnBuilder::Int8(vec![*i; n]),
            ScalarRef::Int16(i) => ColumnBuilder::Int16(vec![*i; n]),
            ScalarRef::UInt8(i) => ColumnBuilder::UInt8(vec![*i; n]),
            ScalarRef::UInt16(i) => ColumnBuilder::UInt16(vec![*i; n]),
            ScalarRef::Boolean(b) => ColumnBuilder::Boolean(constant_bitmap(*b, n)),
            ScalarRef::String(s) => ColumnBuilder::String(vec![s.to_string(); n]),
            ScalarRef::Array(col) => ColumnBuilder::Array {
                array: Box::new(ColumnBuilder::from_column(col.clone())),
                // TODO: use arrow offsets
                offsets: vec![0..col.len(); n],
            },
            ScalarRef::Tuple(fields) => ColumnBuilder::Tuple {
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
            Column::Nullable {
                column: _,
                validity,
            } => validity.len(),
            Column::Tuple { len, .. } => *len,
        }
    }

    pub fn index(&self, index: usize) -> ScalarRef {
        match self {
            Column::Null { .. } => ScalarRef::Null,
            Column::EmptyArray { .. } => ScalarRef::EmptyArray,
            Column::Int8(col) => ScalarRef::Int8(col[index]),
            Column::Int16(col) => ScalarRef::Int16(col[index]),
            Column::UInt8(col) => ScalarRef::UInt8(col[index]),
            Column::UInt16(col) => ScalarRef::UInt16(col[index]),
            Column::Boolean(col) => ScalarRef::Boolean(col.get(index).unwrap()),
            Column::String(col) => ScalarRef::String(&col[index]),
            Column::Array { array, offsets } => {
                ScalarRef::Array((*array).clone().slice(offsets[index].clone()))
            }
            Column::Nullable { column, validity } => {
                if validity.get(index).unwrap() {
                    column.index(index)
                } else {
                    ScalarRef::Null
                }
            }
            Column::Tuple { fields, .. } => {
                ScalarRef::Tuple(fields.iter().map(|field| field.index(index)).collect())
            }
        }
    }

    pub fn slice(&self, range: Range<usize>) -> Self {
        assert!(
            range.end <= self.len(),
            "range {:?} out of len {}",
            range,
            self.len()
        );
        match self {
            Column::Null { .. } => Column::Null {
                len: range.end - range.start,
            },
            Column::EmptyArray { .. } => Column::EmptyArray {
                len: range.end - range.start,
            },
            Column::Int8(col) => {
                Column::Int8(col.clone().slice(range.start, range.end - range.start))
            }
            Column::Int16(col) => {
                Column::Int16(col.clone().slice(range.start, range.end - range.start))
            }
            Column::UInt8(col) => {
                Column::UInt8(col.clone().slice(range.start, range.end - range.start))
            }
            Column::UInt16(col) => {
                Column::UInt16(col.clone().slice(range.start, range.end - range.start))
            }
            Column::Boolean(col) => {
                Column::Boolean(col.clone().slice(range.start, range.end - range.start))
            }
            Column::String(col) => Column::String(col[range].to_vec()),
            Column::Array { array, offsets } => {
                let offsets = offsets[range].to_vec();
                Column::Array {
                    array: array.clone(),
                    offsets,
                }
            }
            Column::Nullable { column, validity } => {
                let validity = validity.clone().slice(range.start, range.end - range.start);
                Column::Nullable {
                    column: Box::new(column.slice(range)),
                    validity,
                }
            }
            Column::Tuple { fields, .. } => Column::Tuple {
                fields: fields
                    .iter()
                    .map(|field| field.slice(range.clone()))
                    .collect(),
                len: range.end - range.start,
            },
        }
    }

    pub fn iter(&self) -> ColumnIterator {
        ColumnIterator {
            column: self,
            index: 0,
            len: self.len(),
        }
    }
}

impl ColumnBuilder {
    pub fn from_column(col: Column) -> Self {
        match col {
            Column::Null { len } => ColumnBuilder::Null { len },
            Column::EmptyArray { len } => ColumnBuilder::EmptyArray { len },
            Column::Int8(col) => ColumnBuilder::Int8(buffer_into_mut(col)),
            Column::Int16(col) => ColumnBuilder::Int16(buffer_into_mut(col)),
            Column::UInt8(col) => ColumnBuilder::UInt8(buffer_into_mut(col)),
            Column::UInt16(col) => ColumnBuilder::UInt16(buffer_into_mut(col)),
            Column::Boolean(col) => ColumnBuilder::Boolean(bitmap_into_mut(col)),
            Column::String(col) => ColumnBuilder::String(col),
            Column::Array { array, offsets } => ColumnBuilder::Array {
                array: Box::new(ColumnBuilder::from_column(*array)),
                offsets,
            },
            Column::Nullable { column, validity } => ColumnBuilder::Nullable {
                column: Box::new(ColumnBuilder::from_column(*column)),
                validity: bitmap_into_mut(validity),
            },
            Column::Tuple { fields, len } => ColumnBuilder::Tuple {
                fields: fields
                    .iter()
                    .map(|col| ColumnBuilder::from_column(col.clone()))
                    .collect(),
                len,
            },
        }
    }

    pub fn len(&self) -> usize {
        match self {
            ColumnBuilder::Null { len } => *len,
            ColumnBuilder::EmptyArray { len } => *len,
            ColumnBuilder::Int8(col) => col.len(),
            ColumnBuilder::Int16(col) => col.len(),
            ColumnBuilder::UInt8(col) => col.len(),
            ColumnBuilder::UInt16(col) => col.len(),
            ColumnBuilder::Boolean(col) => col.len(),
            ColumnBuilder::String(col) => col.len(),
            ColumnBuilder::Array { array: _, offsets } => offsets.len(),
            ColumnBuilder::Nullable {
                column: _,
                validity,
            } => validity.len(),
            ColumnBuilder::Tuple { len, .. } => *len,
        }
    }

    pub fn with_capacity(ty: &DataType, capacity: usize) -> ColumnBuilder {
        match ty {
            DataType::Null => ColumnBuilder::Null { len: 0 },
            DataType::EmptyArray => ColumnBuilder::EmptyArray { len: 0 },
            DataType::Boolean => ColumnBuilder::Boolean(MutableBitmap::with_capacity(capacity)),
            DataType::String => ColumnBuilder::String(Vec::with_capacity(capacity)),
            DataType::UInt8 => ColumnBuilder::UInt8(Vec::with_capacity(capacity)),
            DataType::UInt16 => ColumnBuilder::UInt16(Vec::with_capacity(capacity)),
            DataType::Int8 => ColumnBuilder::Int8(Vec::with_capacity(capacity)),
            DataType::Int16 => ColumnBuilder::Int16(Vec::with_capacity(capacity)),
            DataType::Nullable(ty) => ColumnBuilder::Nullable {
                column: Box::new(Self::with_capacity(ty, capacity)),
                validity: MutableBitmap::with_capacity(capacity),
            },
            DataType::Array(ty) => ColumnBuilder::Array {
                array: Box::new(Self::with_capacity(ty, 0)),
                offsets: Vec::with_capacity(capacity),
            },
            DataType::Tuple(fields) => ColumnBuilder::Tuple {
                fields: fields
                    .iter()
                    .map(|field| Self::with_capacity(field, capacity))
                    .collect(),
                len: 0,
            },
            DataType::Generic(_) => unreachable!(),
        }
    }

    pub fn push(&mut self, item: ScalarRef) {
        match (self, item) {
            (ColumnBuilder::Null { len }, ScalarRef::Null) => *len += 1,
            (ColumnBuilder::EmptyArray { len }, ScalarRef::EmptyArray) => *len += 1,
            (ColumnBuilder::Int8(col), ScalarRef::Int8(value)) => col.push(value),
            (ColumnBuilder::Int16(col), ScalarRef::Int16(value)) => col.push(value),
            (ColumnBuilder::UInt8(col), ScalarRef::UInt8(value)) => col.push(value),
            (ColumnBuilder::UInt16(col), ScalarRef::UInt16(value)) => col.push(value),
            (ColumnBuilder::Boolean(col), ScalarRef::Boolean(value)) => col.push(value),
            (ColumnBuilder::String(col), ScalarRef::String(value)) => col.push(value.to_string()),
            (ColumnBuilder::Array { array, offsets }, ScalarRef::Array(value)) => {
                let start = array.len();
                let end = start + value.len();
                offsets.push(start..end);
                array.append(&ColumnBuilder::from_column(value));
            }
            (ColumnBuilder::Nullable { column, validity }, ScalarRef::Null) => {
                column.push_default();
                validity.push(false);
            }
            (ColumnBuilder::Nullable { column, validity }, scalar) => {
                column.push(scalar);
                validity.push(true);
            }
            (ColumnBuilder::Tuple { fields, len }, ScalarRef::Tuple(value)) => {
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
            ColumnBuilder::Null { len } => *len += 1,
            ColumnBuilder::EmptyArray { len } => *len += 1,
            ColumnBuilder::Int8(col) => col.push(0),
            ColumnBuilder::Int16(col) => col.push(0),
            ColumnBuilder::UInt8(col) => col.push(0),
            ColumnBuilder::UInt16(col) => col.push(0),
            ColumnBuilder::Boolean(col) => col.push(false),
            ColumnBuilder::String(col) => col.push(String::new()),
            ColumnBuilder::Array { array, offsets } => {
                let start = array.len();
                offsets.push(start..start);
            }
            ColumnBuilder::Nullable { column, validity } => {
                column.push_default();
                validity.push(true);
            }
            ColumnBuilder::Tuple { fields, len } => {
                for field in fields {
                    field.push_default();
                }
                *len += 1;
            }
        }
    }

    pub fn append(&mut self, other: &ColumnBuilder) {
        match (self, other) {
            (ColumnBuilder::Null { len }, ColumnBuilder::Null { len: other_len }) => {
                *len += other_len;
            }
            (ColumnBuilder::EmptyArray { len }, ColumnBuilder::EmptyArray { len: other_len }) => {
                *len += other_len;
            }
            (ColumnBuilder::Int8(builder), ColumnBuilder::Int8(other_builder)) => {
                builder.extend_from_slice(other_builder);
            }
            (ColumnBuilder::Int16(builder), ColumnBuilder::Int16(other_builder)) => {
                builder.extend_from_slice(other_builder);
            }
            (ColumnBuilder::UInt8(builder), ColumnBuilder::UInt8(other_builder)) => {
                builder.extend_from_slice(other_builder);
            }
            (ColumnBuilder::UInt16(builder), ColumnBuilder::UInt16(other_builder)) => {
                builder.extend_from_slice(other_builder);
            }
            (ColumnBuilder::Boolean(builder), ColumnBuilder::Boolean(other_builder)) => {
                append_bitmap(builder, other_builder);
            }
            (ColumnBuilder::String(builder), ColumnBuilder::String(other_builder)) => {
                builder.extend_from_slice(other_builder);
            }
            (
                ColumnBuilder::Array { array, offsets },
                ColumnBuilder::Array {
                    array: other_array,
                    offsets: other_offsets,
                },
            ) => {
                array.append(other_array);
                let base = offsets.last().map(|range| range.end).unwrap_or(0);
                offsets.extend(
                    other_offsets
                        .iter()
                        .map(|range| (range.start + base)..(range.end + base)),
                );
            }
            (
                ColumnBuilder::Nullable { column, validity },
                ColumnBuilder::Nullable {
                    column: other_column,
                    validity: other_validity,
                },
            ) => {
                column.append(other_column);
                append_bitmap(validity, other_validity);
            }
            (
                ColumnBuilder::Tuple { fields, len },
                ColumnBuilder::Tuple {
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

    pub fn build(self) -> Column {
        match self {
            ColumnBuilder::Null { len } => Column::Null { len },
            ColumnBuilder::EmptyArray { len } => Column::EmptyArray { len },
            ColumnBuilder::Int8(builder) => Column::Int8(builder.into()),
            ColumnBuilder::Int16(builder) => Column::Int16(builder.into()),
            ColumnBuilder::UInt8(builder) => Column::UInt8(builder.into()),
            ColumnBuilder::UInt16(builder) => Column::UInt16(builder.into()),
            ColumnBuilder::Boolean(builder) => Column::Boolean(builder.into()),
            ColumnBuilder::String(builder) => Column::String(builder),
            ColumnBuilder::Array { array, offsets } => Column::Array {
                array: Box::new(array.build()),
                offsets,
            },
            ColumnBuilder::Nullable { column, validity } => Column::Nullable {
                column: Box::new(column.build()),
                validity: validity.into(),
            },
            ColumnBuilder::Tuple { fields, len } => Column::Tuple {
                fields: fields.into_iter().map(|field| field.build()).collect(),
                len,
            },
        }
    }

    pub fn build_scalar(self) -> Scalar {
        match self {
            ColumnBuilder::Null { len } => {
                assert_eq!(len, 1);
                Scalar::Null
            }
            ColumnBuilder::EmptyArray { len } => {
                assert_eq!(len, 1);
                Scalar::EmptyArray
            }
            ColumnBuilder::Int8(builder) => {
                assert_eq!(builder.len(), 1);
                Scalar::Int8(builder[0])
            }
            ColumnBuilder::Int16(builder) => {
                assert_eq!(builder.len(), 1);
                Scalar::Int16(builder[0])
            }
            ColumnBuilder::UInt8(builder) => {
                assert_eq!(builder.len(), 1);
                Scalar::UInt8(builder[0])
            }
            ColumnBuilder::UInt16(builder) => {
                assert_eq!(builder.len(), 1);
                Scalar::UInt16(builder[0])
            }
            ColumnBuilder::Boolean(builder) => {
                assert_eq!(builder.len(), 1);
                Scalar::Boolean(builder.get(0))
            }
            ColumnBuilder::String(mut builder) => {
                assert_eq!(builder.len(), 1);
                Scalar::String(builder.remove(0))
            }
            ColumnBuilder::Array { array, offsets } => {
                assert_eq!(array.len(), 1);
                assert_eq!(offsets.len(), 1);
                Scalar::Array(array.build())
            }
            ColumnBuilder::Nullable { column, validity } => {
                assert_eq!(column.len(), 1);
                assert_eq!(validity.len(), 1);
                if validity.get(0) {
                    column.build_scalar()
                } else {
                    Scalar::Null
                }
            }
            ColumnBuilder::Tuple { fields, len } => {
                assert_eq!(len, 1);
                Scalar::Tuple(
                    fields
                        .into_iter()
                        .map(|field| field.build_scalar())
                        .collect(),
                )
            }
        }
    }
}

pub struct ColumnIterator<'a> {
    column: &'a Column,
    index: usize,
    len: usize,
}

impl<'a> Iterator for ColumnIterator<'a> {
    type Item = ScalarRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let item = self.column.index(self.index);
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remain = self.len - self.index;
        (remain, Some(remain))
    }
}

unsafe impl<'a> TrustedLen for ColumnIterator<'a> {}
