use proc_macro2::Ident;
use enumflags2::bitflags;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[bitflags]
#[repr(u8)]
pub enum ReflectionMode {
    Recursive,
    // dynamic dispatch (object safe only)
    Dyn,
}

impl ReflectionMode {
    pub fn from_ident(ident: Ident) -> Option<Self> {
        match ident.to_string().as_str() {
            "recursive" => Some(Self::Recursive),
            "dyn" => Some(Self::Dyn),
            _ => None,
        }
    }
}
