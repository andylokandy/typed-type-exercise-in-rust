# Typed Type Exercise in Rust

Build database expression type checker and vectorized runtime executor in type-safe Rust.

> This project is highly inspired by [@skyzh](https://github.com/skyzh)'s [type-exercise-in-rust](https://github.com/skyzh/type-exercise-in-rust). While adopting his idea in [Databend](https://github.com/datafuselabs/databend), I also implemented a few features that I think are useful:

1. **Type checking**. The type checker can catch all type errors in the SQL compilation phase with a set of carefully defined typing rules. The type checker outputs a totally untyped expression that is ready for runtime execution. This makes the runtime free of type errors.

2. **Type-safe downcast**. Function authors not longer worries about downcasting runtime inputs. Thanks to Rust's type system, so long as your function compiles, the downcast always succeeds.

3. **All-in-one generic trait**. We've only one trait `Type`. All other traits like `Array`, `ArrayBuilder`, `ArrayRef` and their sophisticated trait bound are all wiped out.

4. **Enum-dispatched columns**. Use enum to exhaustive all column types and scalar types. They should further minize runtime overhead and mental effort, comparing to `dyn`-dispatched strategy.

## Snippet of code

Define a fast, type-safe, auto-downcating and vectorized binary function in three lines of code:

```rust
let bool_and = Function::new_2_arg::<BooleanType, BooleanType, BooleanType, _>("and", |lhs, rhs| {
    vectorize_binary(lhs, rhs, |lhs: &bool, rhs: &bool| *lhs && *rhs)
});
```

## Run

```
cargo run
```

## Things to do

- [ ] Check ambiguity between function overloads.
- [ ] Read material for the project.

## Reading material

- [Databend/RFC: Formal Type System](https://github.com/datafuselabs/databend/discussions/5438)
- [type-exercise-in-rust](https://github.com/skyzh/type-exercise-in-rust)
- [数据库表达式执行的黑魔法：用 Rust 做类型体操](https://zhuanlan.zhihu.com/p/460702914)
