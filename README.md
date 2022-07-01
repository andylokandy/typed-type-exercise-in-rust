# Typed Type Exercise in Rust

Build database expression type checker and vectorized runtime executor in type-safe Rust.

> This project is highly inspired by [@skyzh](https://github.com/skyzh)'s [type-exercise-in-rust](https://github.com/skyzh/type-exercise-in-rust). While adopting his idea in [Databend](https://github.com/datafuselabs/databend), I also implemented a few features that I think are useful:

1. **Type checking**. The type checker can catch all type errors in the SQL compilation phase with a set of carefully defined typing rules. The type checker outputs a totally untyped expression that is ready for runtime execution. So this makes the runtime free of any type information.

2. **Type-safe downcast**. Function authors no longer have to worry about downcasting runtime inputs. Thanks to Rust's type system, so long as your function compiles, the downcast is always successful.

3. **Enum-dispatched columns**. Use enum to exhaustive all column types and scalar types. They should further minimize runtime overhead and mental effort, compared to `dyn`-dispatched strategy.

4. **Generic types**. Define

## Snippet of code

Define a fast, type-safe, auto-downcating and vectorized binary function in several lines of code:

```rust
registry.register_2_arg::<BooleanType, BooleanType, BooleanType, _>(
    "and",
    FunctionProperty::default(),
    |lhs, rhs| lhs && rhs,
);
```

Define a generic function `get` which returns an item of an array by the index:

```rust
registry.register_2_arg::<ArrayType<GenericType<0>>, Int16Type, GenericType<0>, _>(
    "get",
    FunctionProperty::default(),
    |array, idx| array.index(idx as usize).to_owned(),
);
```

## Run

```
cargo run
```

## Things to do

- [x] Automatcially generate the nullable function.
- [x] Implement arrays.
- [ ] Implement unlimited-length tuples.
- [x] Implment generic functions.
- [x] Implment functions properties.
- [x] Implment variadic functions.
- [ ] Implment sparse columns (some of the rows in a column are hidden).
- [ ] Check ambiguity between function overloads.
- [ ] Read material for the project.

## Reading material

- [Databend/RFC: Formal Type System](https://github.com/datafuselabs/databend/discussions/5438)
- [type-exercise-in-rust](https://github.com/skyzh/type-exercise-in-rust)
- [数据库表达式执行的黑魔法：用 Rust 做类型体操](https://zhuanlan.zhihu.com/p/460702914)
- [Book: Types and Progaming Language](https://www.amazon.com/Types-Programming-Languages-MIT-Press/dp/0262162091) 
- [Paper: Type inference with simple subtypes](https://www.cambridge.org/core/services/aop-cambridge-core/content/view/S0956796800000113)
