#[cfg(feature = "derive")]
pub use finte_derive::IntEnum;

pub trait IntEnum: Sized {
    type Int;

    fn int_value(&self) -> Self::Int;

    fn try_from_int(value: Self::Int) -> Option<Self>;
}
