use enumflags2::bitflags;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[bitflags]
#[repr(u8)]
pub enum ReflectionMode {
    Recursive,
    // dynamic dispatch (object safe only)
    Dyn,
}