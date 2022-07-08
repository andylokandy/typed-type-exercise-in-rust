use std::ops::Range;

use arrow2::{buffer::Buffer, trusted_len::TrustedLen};

use crate::{
    util::buffer_into_mut,
    values::{Column, Scalar},
};

use super::{ArgType, DataType, GenericMap, ValueType};

pub struct StringType;

impl ValueType for StringType {
    type Scalar = Vec<u8>;
    type ScalarRef<'a> = &'a [u8];
    type Column = (Buffer<u8>, Vec<usize>);

    fn to_owned_scalar<'a>(scalar: Self::ScalarRef<'a>) -> Self::Scalar {
        scalar.to_vec()
    }

    fn to_scalar_ref<'a>(scalar: &'a Self::Scalar) -> Self::ScalarRef<'a> {
        scalar
    }
}

impl ArgType for StringType {
    type ColumnIterator<'a> = StringIterator<'a>;
    type ColumnBuilder = (Vec<u8>, Vec<usize>);

    fn data_type() -> DataType {
        DataType::String
    }

    fn try_downcast_scalar<'a>(scalar: &'a Scalar) -> Option<Self::ScalarRef<'a>> {
        scalar.as_string().map(Vec::as_slice)
    }

    fn try_downcast_column<'a>(col: &'a Column) -> Option<Self::Column> {
        col.as_string()
            .map(|(data, offsets)| (data.clone(), offsets.clone()))
    }

    fn upcast_scalar(scalar: Self::Scalar) -> Scalar {
        Scalar::String(scalar)
    }

    fn upcast_column((data, offsets): Self::Column) -> Column {
        Column::String { data, offsets }
    }

    fn column_len<'a>((_, offsets): &'a Self::Column) -> usize {
        offsets.len() - 1
    }

    fn index_column<'a>((data, offsets): &'a Self::Column, index: usize) -> Self::ScalarRef<'a> {
        &data[offsets[index]..offsets[index + 1]]
    }

    fn slice_column<'a>((data, offsets): &'a Self::Column, range: Range<usize>) -> Self::Column {
        let offsets = offsets[range.start..(range.end + 1)].to_vec();
        (data.clone(), offsets)
    }

    fn iter_column<'a>((data, offsets): &'a Self::Column) -> Self::ColumnIterator<'a> {
        StringIterator {
            data,
            offsets: offsets.windows(2),
        }
    }

    fn create_builder(capacity: usize, _: &GenericMap) -> Self::ColumnBuilder {
        let mut offsets = Vec::with_capacity(capacity + 1);
        offsets.push(0);
        (Vec::new(), offsets)
    }

    fn column_to_builder((data, offsets): Self::Column) -> Self::ColumnBuilder {
        (buffer_into_mut(data), offsets)
    }

    fn builder_len((_, offsets): &Self::ColumnBuilder) -> usize {
        offsets.len() - 1
    }

    fn push_item((data, offsets): &mut Self::ColumnBuilder, item: Self::ScalarRef<'_>) {
        data.extend_from_slice(item);
        offsets.push(data.len());
    }

    fn push_default((data, offsets): &mut Self::ColumnBuilder) {
        offsets.push(data.len());
    }

    fn append_builder(
        (data, offsets): &mut Self::ColumnBuilder,
        (other_data, other_offsets): &Self::ColumnBuilder,
    ) {
        data.extend_from_slice(other_data);
        let start = offsets.last().cloned().unwrap();
        offsets.extend(other_offsets.iter().skip(1).map(|offset| start + offset));
    }

    fn build_column((data, offsets): Self::ColumnBuilder) -> Self::Column {
        (data.into(), offsets)
    }

    fn build_scalar((data, offsets): Self::ColumnBuilder) -> Self::Scalar {
        assert_eq!(data.len(), 1);
        assert_eq!(offsets.len(), 2);
        data[offsets[0]..offsets[1]].to_vec()
    }
}

pub struct StringIterator<'a> {
    data: &'a Buffer<u8>,
    offsets: std::slice::Windows<'a, usize>,
}

impl<'a> Iterator for StringIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        self.offsets
            .next()
            .map(|range| &self.data[range[0]..range[1]])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.offsets.size_hint()
    }
}

unsafe impl<'a> TrustedLen for StringIterator<'a> {}
