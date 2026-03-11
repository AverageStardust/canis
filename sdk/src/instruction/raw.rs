use crate::instruction::types::{Immediate, Register, ValidFun3, ValidOpCode};

#[derive(PartialEq)]
pub enum RawInstruction {
    Small(u16),
    Large(u32),
}

impl RawInstruction {
    pub(super) fn r_type(
        rs1: Register,
        fun3: impl ValidFun3,
        rs2: Register,
        rd: Register,
        op: impl ValidOpCode,
    ) -> Self {
        Self::Small(standard_form(*rs1, *fun3, *rs2, *rd, *op))
    }

    pub(super) fn i_type(
        rs1: Register,
        fun3: impl ValidFun3,
        imm: Immediate,
        rd: Register,
        op: impl ValidOpCode,
    ) -> Self {
        Self::Small(standard_form(*rs1, *fun3, *imm, *rd, *op))
    }

    pub(super) fn j_type(imm: Immediate, rd: Register, op: impl ValidOpCode) -> Self {
        let mut instr = 0u16;
        instr |= *op & 0b0000000000001111;
        instr |= (*rd << 4) & 0b0000000001110000;
        instr |= (*imm << 7) & 0b1111111110000000;
        Self::Small(instr)
    }

    pub(super) fn b_type(
        rs1: Register,
        fun3: impl ValidFun3,
        imm: Immediate,
        op: impl ValidOpCode,
    ) -> Self {
        let mut instr = *op & 0b0000000000001111;
        instr |= (*imm << 4) & 0b0000001111110000;
        instr |= (*fun3 << 10) & 0b0001110000000000;
        instr |= (*rs1 << 13) & 0b1110000000000000;
        Self::Small(instr)
    }

    pub(super) fn l_type(
        rs1: Register,
        imm: Immediate,
        rd: Register,
        op: impl ValidOpCode,
    ) -> Self {
        let mut instr = *op & 0b0000000000001111;
        instr |= (*rd << 4) & 0b0000000001110000;
        instr |= (*imm << 7) & 0b0001111110000000;
        instr |= (*rs1 << 13) & 0b1110000000000000;
        Self::Small(instr)
    }

    pub(super) fn s_type(
        rs1: Register,
        rs2: Register,
        imm: Immediate,
        op: impl ValidOpCode,
    ) -> Self {
        let upper = (*imm & 0b111000) >> 3;
        let lower = *imm & 0b000111;
        let mut instr = *op & 0b0000000000001111;
        instr |= (lower << 4) & 0b0000000001110000;
        instr |= (*rs2 << 7) & 0b0000001110000000;
        instr |= (upper << 10) & 0b0001110000000000;
        instr |= (*rs1 << 13) & 0b1110000000000000;
        Self::Small(instr)
    }

    pub(super) fn e_type(
        imm: Immediate,
        rs1: Register,
        fun3: impl ValidFun3,
        rs2: Register,
        rd: Register,
        op: impl ValidOpCode,
    ) -> Self {
        let mut instr = standard_form(*rs1, *fun3, *rs2, *rd, *op) as u32;
        instr |= (*imm as u32) << 16;
        Self::Large(instr)
    }

    pub(super) fn break_type() -> Self {
        Self::Small(0b1111111111111111)
    }

    pub fn get_size(&self) -> usize {
        match self {
            Self::Small(_) => 1,
            Self::Large(_) => 2,
        }
    }
}

impl std::fmt::Debug for RawInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RawInstruction::Small(val) => write!(f, "RawInstruction::Small(0b{val:0>16b})"),
            RawInstruction::Large(val) => write!(f, "RawInstruction::Large(0b{val:0>32b})"),
        }
    }
}

fn standard_form(a: u16, b: u16, c: u16, d: u16, op: u16) -> u16 {
    let mut instr = 0u16;
    instr |= op & 0b0000000000001111;
    instr |= (d << 4) & 0b0000000001110000;
    instr |= (c << 7) & 0b0000001110000000;
    instr |= (b << 10) & 0b0001110000000000;
    instr |= (a << 13) & 0b1110000000000000;
    instr
}

impl From<u16> for RawInstruction {
    fn from(value: u16) -> Self {
        Self::Small(value)
    }
}

impl From<u32> for RawInstruction {
    fn from(value: u32) -> Self {
        Self::Large(value)
    }
}

impl From<RawInstruction> for Vec<RawInstruction> {
    fn from(value: RawInstruction) -> Self {
        vec![value]
    }
}

impl RawInstructionVec {
    pub fn get_size(&self) -> usize {
        self.0.iter().map(|instr| instr.get_size()).sum()
    }
}

#[repr(transparent)]
pub struct RawInstructionVec(Vec<RawInstruction>);

impl From<Vec<RawInstruction>> for RawInstructionVec {
    fn from(value: Vec<RawInstruction>) -> Self {
        Self(value)
    }
}

impl IntoIterator for RawInstructionVec {
    type Item = RawInstruction;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
