# Clap Value Enum Catch All

This project provides a derive macro to enable `ValueEnum` semantics on enums where 1 variant
has associated data that can be parsed from a string. For example

```rust
#[derive(Debug, Parser)]
pub struct Args {
    arg1: MyEnum
}
#[derive(Debug, Clone, ValueEnumCatchall)]
pub enum MyEnum {
    One,
    Other(uuid::Uuid),
}
```

The nice part of `#[derive(ValueEnum)]` is that the help output always contains the potential values.
This project allows more helpful output:

```shell
error: invalid value for one of the arguments

  tip: some similar values exist: 'One', '<uuid>'

For more information, try '--help'.
```
