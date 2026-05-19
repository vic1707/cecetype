# cecetype

`cecetype` is a small schema and dynamic value layer for serde-shaped Rust types.

It is built for code that wants to describe a type once, ship that description somewhere else, and still serialize dynamic values exactly as if they were the original Rust type. On the producer side, schemas can be fully static and `no_std`/`no_alloc`; on the consumer side, the same schema can drive a CLI, web UI, test tool, or any other dynamic interface.

## Quick Start

```rust
use cecetype::Schema;

#[derive(Schema)]
struct User {
    name: String,
    age: u32,
}

assert_eq!(User::SCHEMA.to_string(), "User { name: str, age: u32 }");
```

`Schema::SCHEMA` is a compile-time description of the type. With the default features, the derive supports common Rust, serde, `alloc`, and CLI-facing types.

## Dynamic Values

A schema can decode serde input into a `Value`, or build one from a parser:

```rust,ignore
use cecetype::{OwnedValue, Schema, flavors::Owned, parse::cli::Parser};

#[derive(Schema)]
struct Request {
    id: u32,
    msg: String,
}

let mut parser = Parser::new("42 'hello'");
let value: OwnedValue = Request::SCHEMA.build_value::<Owned, _>(&mut parser)?;
```

The important bit is that `Value` serializes like the type it represents, not like a `Value` enum. If `value` was built from `Request::SCHEMA`, serializing it with postcard, JSON, MessagePack, etc. produces the same shape as serializing a real `Request`.

That makes `cecetype` useful as a bridge: the host can build a dynamic value from a schema, while the device or server still receives its normal typed request.

## CLI Parser Syntax

The CLI parser is intentionally positional. It is meant for generated command help and REPLs, not for a general configuration language.

If a command has request type:

```rust
use cecetype::Schema;

#[derive(Schema)]
struct Update {
    id: u32,
    name: Option<String>,
    flags: [u8; 3],
}
```

the parser sees only the argument string, not the command name:

```text
42 some('controller') [1, 2, 3]
```

Basic syntax:

| Rust shape        | CLI syntax                                          |
| ----------------- | --------------------------------------------------- |
| integers / floats | `42`, `-3`, `1.5`                                   |
| bool              | `true`, `false`, `yes`, `no`                        |
| string / char     | quoted: `'hello'`, `"x"`                            |
| unit              | nothing, or `()`                                    |
| option            | `none`, `null`, or `some(<value>)`                  |
| tuple             | `1 false` at the top level, `(1 false)` when nested |
| struct            | fields in declaration order: `42 true`              |
| enum              | variant name, then payload: `Tuple 42 true`         |
| array             | `[1, 2, 3]`                                         |
| slice / vec       | `[1, 2, 3]`                                         |
| map               | `{'key': 1, 'other': 2}`                            |

Parentheses are used when one value contains several positional children and that value is itself nested inside another value:

```rust
use cecetype::Schema;

#[derive(Schema)]
struct Inner {
    a: u32,
    b: bool,
}

#[derive(Schema)]
struct Outer {
    inner: Inner,
    tail: u8,
}
```

`Inner` alone is parsed without grouping:

```text
1 false
```

`Outer` needs `inner` grouped so the parser can see it as one field:

```text
(1 false) 7
```

Enums are matched by variant name, case-insensitively:

```rust
use cecetype::Schema;

#[derive(Schema)]
enum Action {
    Unit,
    Tuple(u32, bool),
    Struct { x: u8, y: u8 },
}
```

```text
unit
Tuple 42 true
Struct 3 4
```

Strings and chars must be quoted. Backslashes are only used by the lexer to let a quoted word continue past the next byte; the parser does not currently unescape sequences like `\n`.

## Generated Command Help

`parse::cli::spec::Spec` formats a command name, description, request schema, response schema, usage line, and example. This is meant for capability-driven tools:

```rust
use cecetype::{Schema, flavors::Static, parse::cli::spec::Spec};

let spec = Spec::<Static>::new(
    "update",
    "Update read frequency",
    <u8 as Schema>::SCHEMA,
    <() as Schema>::SCHEMA,
).unwrap();

assert_eq!(spec.to_string(), "\
update -- Update read frequency

USAGE:
	update <u8>

EXAMPLE:
	update 0

RESPONSE:
	()

")
```

## Why

`cecetype` was extracted from `ceceti` (more on that later), where the firmware can expose a set of commands and capabilities over USB CDC.
The host CLI should not need to be rebuilt for every firmware build and effectively require one CLI build per firmware build.
Capabilities let the device describe itself instead.

The requirements were:

- static schema data on the device, with `no_std` and no allocation required;
- minimal runtime cost on small hardware;
- dynamic clients that can build requests and decode responses from schemas;
- serde-compatible wire data, so dynamic `Value`s are invisible to the typed endpoint receiving them;
- ability to create custom parsers.

`serde_describe`, `serde-reflection`, and the postcard schema/dynamic crates all overlap with parts of this, but none matched that exact combination.
`cecetype` is the small piece that fell out of needing all of them at once.

It also started as a "surely this is a week" project and became the familiar longer, more interesting version of that story.

## Why That Name?

`cecetype` is named after `ceceti`, the project it was extracted from.
`ceceti` is not public yet; its name comes from misguessing that the French `cécité` ("blindness") would become something like `cecety` or `cecity` in English.
The project name stuck because the goal was to not lose sight of old consoles.

The name fits here too: a dynamic client does not know the concrete Rust types at compile time, but it can still see their schemas.

## Features

| Feature        | Description                                          |
| -------------- | ---------------------------------------------------- |
| `default`      | Enables `derive`, `alloc`, and `cli`                 |
| `derive`       | Re-exports `#[derive(Schema)]`                       |
| `alloc`        | Owned schemas/values with `Box`, `Vec`, and `String` |
| `std`          | Standard-library support, implies `alloc`            |
| `heapless`     | `heapless` collection support                        |
| `cli`          | Text parser and command spec formatting              |
| `miette`       | CLI parser diagnostics, implies `std` and `cli`      |
| `miette-fancy` | Graphical `miette` diagnostics                       |

## Acknowledgements

The design was informed by several adjacent crates:

- [`serde_describe`](https://docs.rs/serde_describe/latest/serde_describe/) explores deriving type descriptions from serde-shaped Rust types.
- [`serde-reflection`](https://docs.rs/serde-reflection/latest/serde_reflection/) records serde formats by tracing sample values.
- [`postcard-schema`](https://docs.rs/postcard-schema/latest/postcard_schema/) provides static schema metadata for postcard-compatible types, including owned schema forms.
- [`postcard-dyn`](https://docs.rs/postcard-dyn/latest/postcard_dyn/) goes further and provides dynamic postcard encode/decode from postcard schemas.

The postcard crates are closest in spirit.
`cecetype` differs by carrying its own schema-shaped `Value` instead of using `serde_json::Value`, and by making that value serialize through serde as the represented type.
That keeps it usable with postcard, JSON, MessagePack, CBOR, or any other serde format, and gives the CLI parser and command help generator a common value model to target.
