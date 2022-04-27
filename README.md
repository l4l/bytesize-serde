Alternative serde implementation for `bytesize` crate based on its `FromStr` implementation for human-readable serializers.

# Usage

```rust
use bytesize::ByteSize;
use serde::{Serialize, Deserialize};

# fn main() {
#[derive(Serialize, Deserialize)]
struct T {
    #[serde(with = "bytesize_serde")]
    x: ByteSize,
}

let t: T = serde_json::from_str(r#"{ "x": "5 MB" }"#).unwrap();
assert_eq!(t.x, "5 MB".parse::<ByteSize>().unwrap());
# }
```
