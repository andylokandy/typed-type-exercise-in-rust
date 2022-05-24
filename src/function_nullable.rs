use crate::{
    runtime::{Value, ValueRef},
    types::*,
    values::combine_nulls,
};

pub(crate) fn erase_function_generic_1_nullable<I1: Type, O: Type>(
    func: impl for<'a> Fn(ValueRef<'a, I1>) -> Value<O>,
) -> impl Fn(&[ValueRef<AnyType>]) -> Value<AnyType> {
    move |args| {
        let mut nulls = vec![];
        let arg1 = match &args[0] {
            ValueRef::Scalar(scalar) => {
                let v = <NullableType<I1>>::try_downcast_scalar(scalar).unwrap();
                ValueRef::Scalar(v.unwrap())
            }
            ValueRef::Column(col) => {
                let (col, null) = <NullableType<I1>>::try_downcast_column(&col).unwrap();
                nulls.push(null);
                ValueRef::Column(col)
            }
        };
        let result = func(arg1);
        let nulls = combine_nulls(&nulls);

        match result {
            Value::Scalar(scalar) => Value::Scalar(<NullableType<O>>::upcast_scalar(Some(scalar))),
            Value::Column(col) => Value::Column(<NullableType<O>>::upcast_column((col, nulls))),
        }
    }
}

pub(crate) fn erase_function_generic_2_nullable<I1: Type, I2: Type, O: Type>(
    func: impl for<'a> Fn(ValueRef<'a, I1>, ValueRef<'a, I2>) -> Value<O>,
) -> impl Fn(&[ValueRef<AnyType>]) -> Value<AnyType> {
    move |args| {
        let mut nulls = vec![];
        let arg1 = match &args[0] {
            ValueRef::Scalar(scalar) => {
                let v = <NullableType<I1>>::try_downcast_scalar(scalar).unwrap();
                ValueRef::Scalar(v.unwrap())
            }
            ValueRef::Column(col) => {
                let (col, null) = <NullableType<I1>>::try_downcast_column(&col).unwrap();
                nulls.push(null);
                ValueRef::Column(col)
            }
        };
        let arg2 = match &args[1] {
            ValueRef::Scalar(scalar) => {
                let v = <NullableType<I2>>::try_downcast_scalar(scalar).unwrap();
                ValueRef::Scalar(v.unwrap())
            }
            ValueRef::Column(col) => {
                let (col, null) = <NullableType<I2>>::try_downcast_column(&col).unwrap();
                nulls.push(null);
                ValueRef::Column(col)
            }
        };

        let result = func(arg1, arg2);
        let nulls = combine_nulls(&nulls);

        match result {
            Value::Scalar(scalar) => Value::Scalar(<NullableType<O>>::upcast_scalar(Some(scalar))),
            Value::Column(col) => Value::Column(<NullableType<O>>::upcast_column((col, nulls))),
        }
    }
}
