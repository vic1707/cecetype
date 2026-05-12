# cecetype

Runtime type schema definitions with serde interop. `no_std` compatible.

## Quick Start

```rust
use cecetype::Schema;

#[derive(Schema)]
struct User {
    name: String,
    age: u32,
}

assert_eq!(
    User::SCHEMA.to_string(),
    "User { name: str, age: u32 }"
);
```

## Schema

`Schema::SCHEMA` provides a compile-time description of any type:

```rust
use cecetype::{Schema, StaticSchema};

#[derive(Schema)]
struct Point3D {
    x: f32,
    y: f32,
    z: f32,
}

let schema: &StaticSchema = Point3D::SCHEMA;
assert_eq!(schema.to_string(), "Point3D { x: f32, y: f32, z: f32 }");
```

Use schemas to deserialize data from any serde deserializer into a `Value`:

```rust,ignore
// See tests/roundtrip.rs for full examples
let value: OwnedValue = schema.decode_value(deserializer)?;
```

## CLI Parsing

Parse text input into `Value` using a schema. The CLI parser uses **positional** syntax for struct fields:

```rust,ignore
// See tests/cli_parser.rs for full examples
use cecetype::{Schema, OwnedValue, parse::cli::Parser};

#[derive(Schema)]
struct Request { id: u32, msg: String }

// Fields parsed positionally after type name
let mut parser = Parser::<cecetype::flavors::Owned>::new("Request 42 'hello'");
let value: OwnedValue = Request::SCHEMA.build_value(&mut parser).unwrap();
```

## Custom Flavors

Flavors define memory models. Implement [`SchemaFlavor`](crate::flavors::SchemaFlavor) for zero-copy borrowing, or [`OwnedSchemaFlavor`](crate::flavors::OwnedSchemaFlavor) for owned data:

```rust
use cecetype::flavors::SchemaFlavor;
use core::fmt;

pub struct MyFlavor;

impl<'s> SchemaFlavor<'s> for MyFlavor {
    type Ptr<T: 's + Clone + PartialEq + fmt::Debug> = &'s T;
    type List<T: 's + Clone + PartialEq + fmt::Debug> = &'s [&'s T];
    type Str = &'s str;
}
```

## Custom Parsers

Parsers aren't limited to text. Implement [`Parser`](crate::parse::Parser) trait:

```rust,ignore
// See tests/cli_parser.rs for a full example
impl<'s, VB: ValueBuilder> Parser<'s, VB> for MyParser {
    type Error = /* ... */;
    type Atom = /* ... */;

    fn next_atom(&mut self) -> Result<Self::Atom, Self::Error> { todo!() }
    // implement remaining methods...
}
```

## Derive Attributes

```rust
use cecetype::Schema;

// Reference another type (for recursive/cyclic types)
#[derive(Schema)]
#[schema(ref(TreeNode))]
struct TreeNode {
    value: i32,
    children: Vec<TreeNode>,
}

// Represent as a different type
#[derive(Schema)]
#[schema(as(&str))]
struct Name(String);
```

Serde `rename` and `skip` attributes are also supported.

## Features

| Feature    | Description                         |
| ---------- | ----------------------------------- |
| `default`  | Enables `alloc` + `cli`             |
| `alloc`    | Owned types: `Box`, `Vec`, `String` |
| `std`      | Full std support (implies `alloc`)  |
| `heapless` | `heapless::Vec` support             |
| `cli`      | CLI text parser                     |

## Acknowledgements

Inspired by [`serde_describe`](https://docs.rs/serde_describe/latest/serde_describe/) and [`postcard-schema`](https://docs.rs/postcard-schema/latest/postcard_schema/).
