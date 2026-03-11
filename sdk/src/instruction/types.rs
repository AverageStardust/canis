use std::ops::{Add, AddAssign, Deref, Sub};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location(usize);

impl Location {
    pub fn new() -> Self {
        Self(0)
    }
}

impl Add<usize> for Location {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<usize> for Location {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}

impl Sub<Location> for Location {
    type Output = Distance;
    fn sub(self, rhs: Location) -> Self::Output {
        Distance((self.0 as isize) - (rhs.0 as isize))
    }
}

pub struct Distance(isize);

#[derive(Debug, PartialEq)]
pub struct Register(u16);

impl Register {
    #[cfg(test)]
    pub(crate) fn mock(val: u16) -> Self {
        Self(val)
    }
}

impl From<u16> for Register {
    fn from(value: u16) -> Self {
        Register(value)
    }
}

impl Deref for Register {
    type Target = u16;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct Immediate(u16);

impl Immediate {
    #[cfg(test)]
    pub(crate) fn mock(val: u16) -> Self {
        Self(val)
    }
}

impl PartialEq<u16> for Immediate {
    fn eq(&self, other: &u16) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<u16> for Immediate {
    fn partial_cmp(&self, other: &u16) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl From<u16> for Immediate {
    fn from(value: u16) -> Self {
        Immediate(value)
    }
}

impl Deref for Immediate {
    type Target = u16;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct OpCode<const N: u16>;

impl<const N: u16> Deref for OpCode<N> {
    type Target = u16;
    fn deref(&self) -> &Self::Target {
        &N
    }
}

pub struct Fun3<const N: u16>;

impl<const N: u16> Deref for Fun3<N> {
    type Target = u16;
    fn deref(&self) -> &Self::Target {
        &N
    }
}

mod sealed {
    use std::ops::Deref;

    use crate::instruction::types::{Fun3, OpCode};

    trait SealedOpCode {}
    trait SealedFun3 {}

    #[allow(private_bounds)]
    pub trait ValidOpCode: Deref<Target = u16> + SealedOpCode {}
    impl<T: SealedOpCode + Deref<Target = u16>> ValidOpCode for T {}

    #[allow(private_bounds)]
    pub trait ValidFun3: Deref<Target = u16> + SealedFun3 {}
    impl<T: SealedFun3 + Deref<Target = u16>> ValidFun3 for T {}

    impl SealedOpCode for OpCode<0b0000> {}
    impl SealedOpCode for OpCode<0b0001> {}
    impl SealedOpCode for OpCode<0b0010> {}
    impl SealedOpCode for OpCode<0b0011> {}
    impl SealedOpCode for OpCode<0b0100> {}
    impl SealedOpCode for OpCode<0b0101> {}
    impl SealedOpCode for OpCode<0b0110> {}
    impl SealedOpCode for OpCode<0b0111> {}
    impl SealedOpCode for OpCode<0b1000> {}
    impl SealedOpCode for OpCode<0b1001> {}
    impl SealedOpCode for OpCode<0b1010> {}
    impl SealedOpCode for OpCode<0b1011> {}
    impl SealedOpCode for OpCode<0b1100> {}
    impl SealedOpCode for OpCode<0b1101> {}
    impl SealedOpCode for OpCode<0b1110> {}
    impl SealedOpCode for OpCode<0b1111> {}

    impl SealedFun3 for Fun3<0b0000> {}
    impl SealedFun3 for Fun3<0b0001> {}
    impl SealedFun3 for Fun3<0b0010> {}
    impl SealedFun3 for Fun3<0b0011> {}
    impl SealedFun3 for Fun3<0b0100> {}
    impl SealedFun3 for Fun3<0b0101> {}
    impl SealedFun3 for Fun3<0b0110> {}
    impl SealedFun3 for Fun3<0b0111> {}
}

pub use sealed::{ValidFun3, ValidOpCode};
