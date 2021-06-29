# Finte

Finte is a proc-macro crate to auto generate conversion code between integer and Rust enum

## Example

```rust
#[derive(finte::IntEnum)]
#[repr(u16)]
pub enum RustEdition {
    Prev = 2015,
    Now  = 2018,
    Next = 2021,
}


// the above generates

impl finte::IntEnum for RustEdition {
    type Int = u16;

    fn try_from_int(value: Self::Int) -> Option<Self> {
        match value {
            2015 => Some(Self::Prev),
            2018 => Some(Self::Now),
            2021 => Some(Self::Next),
            _ => None,
        }
    }

    fn int_value(&self) -> Self::Int {
        match self {
            Self::Prev => 2015,
            Self::Now  => 2018,
            Self::Next => 2021,
        }
    }
}
```
