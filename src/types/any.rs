use crate::values::{Column, Scalar};

use super::ValueType;

pub struct AnyType;

impl ValueType for AnyType {
    type Scalar = Scalar;
    type ScalarRef<'a> = &'a Scalar;
    type Column = Column;

    fn to_owned_scalar<'a>(scalar: Self::ScalarRef<'a>) -> Self::Scalar {
        scalar.clone()
    }

    fn to_scalar_ref<'a>(scalar: &'a Self::Scalar) -> Self::ScalarRef<'a> {
        scalar
    }
}
