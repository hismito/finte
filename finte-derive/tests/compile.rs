#[repr(u16)]
#[derive(finte_derive::IntEnum)]
pub(crate) enum RustEdition {
    Prev = 2015,

    Now = 2018,

    Next = 2021,
}
