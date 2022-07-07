use arrow2::bitmap::{Bitmap, MutableBitmap};

pub fn bitmap_into_mut(bitmap: Bitmap) -> MutableBitmap {
    bitmap
        .into_mut()
        .map_left(|bitmap| {
            let mut builder = MutableBitmap::new();
            builder.extend_from_bitmap(&bitmap);
            builder
        })
        .into_inner()
}

pub fn repeat_bitmap(bitmap: &mut Bitmap, n: usize) -> MutableBitmap {
    let mut builder = MutableBitmap::new();
    for _ in 0..n {
        builder.extend_from_bitmap(bitmap);
    }
    builder
}

pub fn append_bitmap(bitmap: &mut MutableBitmap, other: &MutableBitmap) {
    bitmap.extend_from_slice(other.as_slice(), 0, other.len());
}

pub fn constant_bitmap(value: bool, len: usize) -> MutableBitmap {
    let mut builder = MutableBitmap::new();
    builder.extend_constant(len, value);
    builder
}
