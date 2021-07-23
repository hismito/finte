use std::{fmt, marker::PhantomData};

#[cfg(feature = "derive")]
pub use finte_derive::IntEnum;

pub trait IntEnum: Sized {
    type Int;

    fn int_value(&self) -> Self::Int;

    fn try_from_int(value: Self::Int) -> Result<Self, TryFromIntError<Self>>;
}

pub struct TryFromIntError<T: IntEnum> {
    pub invalid_value: T::Int,
    ty: PhantomData<T>,
}

impl<T: IntEnum> TryFromIntError<T> {
    pub fn new(invalid_value: T::Int) -> Self {
        Self {
            invalid_value,
            ty: PhantomData,
        }
    }
}

impl<T: IntEnum> fmt::Display for TryFromIntError<T>
where
    T::Int: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid value for int enum")
    }
}

impl<T: IntEnum> fmt::Debug for TryFromIntError<T>
where
    T::Int: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid value {:?} for int enum {}",
            self.invalid_value,
            std::any::type_name::<T>()
        )
    }
}

impl<T: IntEnum> std::error::Error for TryFromIntError<T> where T::Int: fmt::Display + fmt::Debug {}
