# cecetype-macros

Procedural macros for [cecetype](https://docs.rs/cecetype).

## Schema Derive

```rust
use cecetype::Schema;

#[derive(Schema)]
struct User {
    name: String,
    age: u32,
}

// User::SCHEMA is &cecetype::StaticSchema
```

## Attributes

| Attribute                    | Description                        |
| ---------------------------- | ---------------------------------- |
| `#[schema(ref(Name))]`       | Direct reference to another type   |
| `#[schema(ref(Name, list))]` | Slice/list reference               |
| `#[schema(as(Type))]`        | Represent as different schema type |
| `#[schema(transparent)]`     | Wrap same as inner field           |
| `#[schema(bounds(...))]`     | Additional trait bounds            |
| `#[serde(rename = "...")]`   | Override serialized name           |
| `#[serde(skip)]`             | Skip field/variant                 |

Serde `from`, `try_from`, `into` are also supported for `as` replacement.
