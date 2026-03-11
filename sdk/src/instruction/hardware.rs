use crate::instruction::{
    hardware::helpers::itype_instr,
    instruction::{Instruction, InstructionMetaExplain, InstructionMetaExplainVariant},
    parser::{Parser, ParserTypes},
    raw::RawInstruction,
    registry::InstructionParserResult,
    types::{Distance, Fun3, Immediate, Location, OpCode, Register},
};

/// Generates an `or` instruction from the given registers
pub fn gen_or(rd: Register, rs1: Register, rs2: Register) -> RawInstruction {
    RawInstruction::r_type(rs1, Fun3::<0b000>, rs2, rd, OpCode::<0b0000>)
}

fn instr_or(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::rtype_instr(parser, gen_or)
}

inventory::submit! {
    Instruction::new_with_explain(
        "or",
        instr_or,
        "Bitwise OR",
        "rd = rs1 | rs2",
        || InstructionMetaExplain::new(
            InstructionMetaExplainVariant::new("Performs a bitwise OR on the values in rs1 and rs2, placing the result into rd.")
                .arg("rd", ParserTypes::Register)
                .arg("rs1", ParserTypes::Register)
                .arg("rs2", ParserTypes::Register)
        )
    )
}

/// Generates an `xor` instruction from the given registers
pub fn gen_xor(rd: Register, rs1: Register, rs2: Register) -> RawInstruction {
    RawInstruction::r_type(rs1, Fun3::<0b001>, rs2, rd, OpCode::<0b0000>)
}

fn instr_xor(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::rtype_instr(parser, gen_xor)
}

inventory::submit! {
    Instruction::new_with_meta("xor", instr_xor, "Bitwise XOR", "rd = rs1 ^ rs2")
}

/// Generates an `and` instruction from the given registers
pub fn gen_and(rd: Register, rs1: Register, rs2: Register) -> RawInstruction {
    RawInstruction::r_type(rs1, Fun3::<0b010>, rs2, rd, OpCode::<0b0000>)
}

fn instr_and(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::rtype_instr(parser, gen_and)
}

inventory::submit! {
    Instruction::new_with_meta("and", instr_and, "Bitwise AND", "rd = rs1 & rs2")
}

/// Generates an `mul` instruction from the given registers
pub fn gen_mul(rd: Register, rs1: Register, rs2: Register) -> RawInstruction {
    RawInstruction::r_type(rs1, Fun3::<0b011>, rs2, rd, OpCode::<0b0000>)
}

fn instr_mul(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::rtype_instr(parser, gen_mul)
}

inventory::submit! {
    Instruction::new_with_meta("mul", instr_mul, "Multiplication", "rd, CRY = rs1 * rs2  *signed")
}

/// Generates an `add` instruction from the given registers
pub fn gen_add(rd: Register, rs1: Register, rs2: Register) -> RawInstruction {
    RawInstruction::r_type(rs1, Fun3::<0b100>, rs2, rd, OpCode::<0b0000>)
}

fn instr_add(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::rtype_instr(parser, gen_add)
}

inventory::submit! {
    Instruction::new_with_meta("add", instr_add, "Addition", "rd, CRY = rs1 + rs2")
}

/// Generates an `sub` instruction from the given registers
pub fn gen_sub(rd: Register, rs1: Register, rs2: Register) -> RawInstruction {
    RawInstruction::r_type(rs1, Fun3::<0b101>, rs2, rd, OpCode::<0b0000>)
}

fn instr_sub(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::rtype_instr(parser, gen_sub)
}

inventory::submit! {
    Instruction::new_with_meta("sub", instr_sub, "Subtraction", "rd, CRY = rs1 - rs2")
}

/// Generates an `sl` instruction from the given registers
pub fn gen_sl(rd: Register, rs1: Register, rs2: Register) -> RawInstruction {
    RawInstruction::r_type(rs1, Fun3::<0b110>, rs2, rd, OpCode::<0b0000>)
}

fn instr_sl(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::rtype_instr(parser, gen_sl)
}

inventory::submit! {
    Instruction::new_with_meta("sl", instr_sl, "Shift Left", "rd, CRY = rs1 << rs2")
}

/// Generates an `sr` instruction from the given registers
pub fn gen_sr(rd: Register, rs1: Register, rs2: Register) -> RawInstruction {
    RawInstruction::r_type(rs1, Fun3::<0b111>, rs2, rd, OpCode::<0b0000>)
}

fn instr_sr(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::rtype_instr(parser, gen_sr)
}

inventory::submit! {
    Instruction::new_with_meta("sr", instr_sr, "Shift Right", "rd, CRY = rs1 >> rs2")
}

/// Generates an `addc` instruction from the given registers
pub fn gen_addc(rd: Register, rs1: Register, rs2: Register) -> RawInstruction {
    RawInstruction::r_type(rs1, Fun3::<0b100>, rs2, rd, OpCode::<0b0001>)
}

fn instr_addc(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::rtype_instr(parser, gen_addc)
}

inventory::submit! {
    Instruction::new_with_meta("addc", instr_addc, "Addition with Carry", "rd, CRY = rs1 + rs2 + CRY")
}

/// Generates an `subc` instruction from the given registers
pub fn gen_subc(rd: Register, rs1: Register, rs2: Register) -> RawInstruction {
    RawInstruction::r_type(rs1, Fun3::<0b101>, rs2, rd, OpCode::<0b0001>)
}

fn instr_subc(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::rtype_instr(parser, gen_subc)
}

inventory::submit! {
    Instruction::new_with_meta("subc", instr_subc, "Subtraction with Carry", "rd, CRY = rs1 - rs2 - CRY")
}

/// Generates an `slc` instruction from the given registers
pub fn gen_slc(rd: Register, rs1: Register, rs2: Register) -> RawInstruction {
    RawInstruction::r_type(rs1, Fun3::<0b110>, rs2, rd, OpCode::<0b0001>)
}

fn instr_slc(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::rtype_instr(parser, gen_slc)
}

inventory::submit! {
    Instruction::new_with_meta("slc", instr_slc, "Shift Left with Carry", "rd, CRY = rs1 << rs2  *carry-filled")
}

/// Generates an `src` instruction from the given registers
pub fn gen_src(rd: Register, rs1: Register, rs2: Register) -> RawInstruction {
    RawInstruction::r_type(rs1, Fun3::<0b111>, rs2, rd, OpCode::<0b0001>)
}

fn instr_src(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::rtype_instr(parser, gen_src)
}

inventory::submit! {
    Instruction::new_with_meta("src", instr_src, "Shift Right with Carry", "rd, CRY = rs1 >> rs2  *carry-filled")
}

/// Generates an `ori` instruction from the given registers and immediate
pub fn gen_ori(rd: Register, rs1: Register, imm: Immediate) -> RawInstruction {
    RawInstruction::i_type(rs1, Fun3::<0b000>, imm, rd, OpCode::<0b0010>)
}

fn instr_ori(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::itype_instr(parser, gen_ori)
}

inventory::submit! {
    Instruction::new_with_explain(
        "ori",
        instr_ori,
        "Bitwise OR with Immediate",
        "rd = rs1 | imm",
        || InstructionMetaExplain::new(
            InstructionMetaExplainVariant::new("Performs a bitwise OR on the value in rs1 with the immediate imm, placing the result into rd.")
            .arg("rd", ParserTypes::Register)
            .arg("rs1", ParserTypes::Register)
            .arg("imm", ParserTypes::Immediate.with_expected_bits(3))
        )
    )
}

/// Generates an `xori` instruction from the given registers and immediate
pub fn gen_xori(rd: Register, rs1: Register, imm: Immediate) -> RawInstruction {
    RawInstruction::i_type(rs1, Fun3::<0b001>, imm, rd, OpCode::<0b0010>)
}

fn instr_xori(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::itype_instr(parser, gen_xori)
}

inventory::submit! {
    Instruction::new("xori", instr_xori)
}

/// Generates an `andi` instruction from the given registers and immediate
pub fn gen_andi(rd: Register, rs1: Register, imm: Immediate) -> RawInstruction {
    RawInstruction::i_type(rs1, Fun3::<0b010>, imm, rd, OpCode::<0b0010>)
}

fn instr_andi(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::itype_instr(parser, gen_andi)
}

inventory::submit! {
    Instruction::new("andi", instr_andi)
}

/// Generates an `muli` instruction from the given registers and immediate
pub fn gen_muli(rd: Register, rs1: Register, imm: Immediate) -> RawInstruction {
    RawInstruction::i_type(rs1, Fun3::<0b011>, imm, rd, OpCode::<0b0010>).into()
}

fn instr_muli(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::itype_instr(parser, gen_muli)
}

inventory::submit! {
    Instruction::new_with_meta("muli", instr_muli, "Multiply with Immediate", "rd, CRY = rs1 * imm  *signed")
}

/// Generates an `addi` instruction from the given registers and immediate
pub fn gen_addi(rd: Register, rs1: Register, imm: Immediate) -> RawInstruction {
    RawInstruction::i_type(rs1, Fun3::<0b100>, imm, rd, OpCode::<0b0010>).into()
}

fn instr_addi(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::itype_instr(parser, gen_addi)
}

inventory::submit! {
    Instruction::new("addi", instr_addi)
}

/// Generates an `subi` instruction from the given registers and immediate
pub fn gen_subi(rd: Register, rs1: Register, imm: Immediate) -> RawInstruction {
    RawInstruction::i_type(rs1, Fun3::<0b101>, imm, rd, OpCode::<0b0010>).into()
}

fn instr_subi(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::itype_instr(parser, gen_subi)
}

inventory::submit! {
    Instruction::new("subi", instr_subi)
}

/// Generates an `sli` instruction from the given registers and immediate
pub fn gen_sli(rd: Register, rs1: Register, imm: Immediate) -> RawInstruction {
    RawInstruction::i_type(rs1, Fun3::<0b110>, imm, rd, OpCode::<0b0010>).into()
}

/// Generates an alternate `sli` instruction from the given registers and immediate
pub fn gen_sli_alternate(rd: Register, rs1: Register, imm: Immediate) -> RawInstruction {
    RawInstruction::i_type(rs1, Fun3::<0b110>, imm, rd, OpCode::<0b1111>).into()
}

fn instr_sli(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    itype_instr(parser, |rd, rs1, imm| {
        if imm > 3 && imm < 12 {
            gen_sli_alternate(rd, rs1, imm)
        } else {
            gen_sli(rd, rs1, imm)
        }
    })
}

inventory::submit! {
    Instruction::new("sli", instr_sli)
}

/// Generates an `sri` instruction from the given registers and immediate
pub fn gen_sri(rd: Register, rs1: Register, imm: Immediate) -> RawInstruction {
    RawInstruction::i_type(rs1, Fun3::<0b111>, imm, rd, OpCode::<0b0010>).into()
}

/// Generates an alternate `sri` instruction from the given registers and immediate
pub fn gen_sri_alternate(rd: Register, rs1: Register, imm: Immediate) -> RawInstruction {
    RawInstruction::i_type(rs1, Fun3::<0b111>, imm, rd, OpCode::<0b1111>).into()
}

fn instr_sri(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    itype_instr(parser, |rd, rs1, imm| {
        if imm > 3 && imm < 12 {
            gen_sri_alternate(rd, rs1, imm)
        } else {
            gen_sri(rd, rs1, imm)
        }
    })
}

inventory::submit! {
    Instruction::new_with_meta("sri", instr_sri, "Shift Right with Immediate", "rd, CRY = rs1 >> imm")
}

// TODO: Accumulate & Extended instructions

/// Generates an `li` instruction from the given register and immediate
pub fn gen_li(rd: Register, imm: Immediate) -> RawInstruction {
    RawInstruction::j_type(imm, rd, OpCode::<0b0101>)
}

fn instr_li(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::jtype_instr(parser, gen_li)
}

inventory::submit! {
    Instruction::new("li", instr_li)
}

/// Generates an `lie` instruction from the given register and immediate
pub fn gen_lie(rd: Register, imm: Immediate) -> RawInstruction {
    RawInstruction::e_type(
        imm,
        Register::from(0),
        Fun3::<0b000>,
        Register::from(0),
        rd,
        OpCode::<0b0110>,
    )
}

fn instr_lie(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    let rd = parser.require_register()?;
    let imm = parser.require_immediate(16)?;
    Ok(gen_lie(rd, imm).into())
}

inventory::submit! {
    Instruction::new("lie", instr_lie)
}

/// Generates an `lm` instruction from the given registers and immediate
pub fn gen_lm(dest: Register, addr: Register, imm: Immediate) -> RawInstruction {
    RawInstruction::l_type(addr, imm, dest, OpCode::<0b0111>)
}

fn instr_lm(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::mem_instr(parser, gen_lm)
}

inventory::submit! {
    Instruction::new_with_explain(
        "lm",
        instr_lm,
        "Store Memory",
        "rd = M[rs1 + imm]",
        || InstructionMetaExplain::new(
            InstructionMetaExplainVariant::new("Loads the value stored in memory at the address of rs1 + imm into rs2.")
            .arg("rd", ParserTypes::Register)
            .arg("rs1", ParserTypes::Register)
            .arg("imm", ParserTypes::Immediate.with_expected_bits(6))
        )
    )
}

/// Generates an `sm` instruction from the given registers and immediate
pub fn gen_sm(source: Register, addr: Register, imm: Immediate) -> RawInstruction {
    RawInstruction::s_type(addr, source, imm, OpCode::<0b1000>)
}

fn instr_sm(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::mem_instr(parser, gen_sm)
}

inventory::submit! {
    Instruction::new_with_explain(
        "sm",
        instr_sm,
        "Store Memory",
        "M[rs1 + imm] = rs2",
        || InstructionMetaExplain::new(
            InstructionMetaExplainVariant::new("Stores the value stored in rs2 at the address of rs1 + imm in memory.")
            .arg("rs2", ParserTypes::Register)
            .arg("rs1", ParserTypes::Register)
            .arg("imm", ParserTypes::Immediate.with_expected_bits(6))
        )
    )
}

/// Generates a `beqz` instruction from the given register and distance
pub fn gen_beqz(source: Register, distance: Distance) -> RawInstruction {
    RawInstruction::b_type(source, Fun3::<0b001>, distance.into(), OpCode::<0b1001>)
}

fn instr_beqz(pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::btype_instr(pc, parser, gen_beqz)
}

inventory::submit! {
    Instruction::new("beqz", instr_beqz)
}

/// Generates a `bnez` instruction from the given register and distance
pub fn gen_bnez(source: Register, distance: Distance) -> RawInstruction {
    RawInstruction::b_type(source, Fun3::<0b010>, distance.into(), OpCode::<0b1001>)
}

fn instr_bnez(pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::btype_instr(pc, parser, gen_bnez)
}

inventory::submit! {
    Instruction::new("bnez", instr_bnez)
}

/// Generates a `bgtz` instruction from the given register and distance
pub fn gen_bgtz(source: Register, distance: Distance) -> RawInstruction {
    RawInstruction::b_type(source, Fun3::<0b011>, distance.into(), OpCode::<0b1001>)
}

fn instr_bgtz(pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::btype_instr(pc, parser, gen_bgtz)
}

inventory::submit! {
    Instruction::new("bgtz", instr_bgtz)
}

/// Generates a `blez` instruction from the given register and distance
pub fn gen_blez(source: Register, distance: Distance) -> RawInstruction {
    RawInstruction::b_type(source, Fun3::<0b100>, distance.into(), OpCode::<0b1001>)
}

fn instr_blez(pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::btype_instr(pc, parser, gen_blez)
}

inventory::submit! {
    Instruction::new("blez", instr_blez)
}

/// Generates a `bltz` instruction from the given register and distance
pub fn gen_bltz(source: Register, distance: Distance) -> RawInstruction {
    RawInstruction::b_type(source, Fun3::<0b101>, distance.into(), OpCode::<0b1001>)
}

fn instr_bltz(pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::btype_instr(pc, parser, gen_bltz)
}

inventory::submit! {
    Instruction::new("bltz", instr_bltz)
}

/// Generates a `bgez` instruction from the given register and distance
pub fn gen_bgez(source: Register, distance: Distance) -> RawInstruction {
    RawInstruction::b_type(source, Fun3::<0b110>, distance.into(), OpCode::<0b1001>)
}

fn instr_bgez(pc: Location, parser: &mut Parser) -> InstructionParserResult {
    helpers::btype_instr(pc, parser, gen_bgez)
}

inventory::submit! {
    Instruction::new("bgez", instr_bgez)
}

// TODO: Implement alts & Jumps

/// Generates a `in` instruction from the given register and immediate
pub fn gen_in(rd: Register, imm: Immediate) -> RawInstruction {
    RawInstruction::i_type(0.into(), Fun3::<000>, imm, rd, OpCode::<0b1111>)
}

fn instr_in(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    let rd = parser.require_register()?;
    let imm = parser.require_immediate(3)?;
    Ok(gen_in(rd, imm).into())
}

inventory::submit! {
    Instruction::new("in", instr_in)
}
/// Generates a `out` instruction from the given register and immediate
pub fn gen_out(rs1: Register, imm: Immediate) -> RawInstruction {
    RawInstruction::i_type(0.into(), Fun3::<000>, imm, rs1, OpCode::<0b1111>)
}

fn instr_out(_pc: Location, parser: &mut Parser) -> InstructionParserResult {
    let rsr1 = parser.require_register()?;
    let imm = parser.require_immediate(3)?;
    Ok(gen_out(rsr1, imm).into())
}

inventory::submit! {
    Instruction::new("out", instr_out)
}

mod helpers {
    use super::*;

    #[inline(always)]
    pub(super) fn rtype_instr<F>(parser: &mut Parser, generator: F) -> InstructionParserResult
    where
        F: Fn(Register, Register, Register) -> RawInstruction,
    {
        let rd = parser.require_register()?;
        let rs1 = parser.require_register()?;
        let rs2 = parser.require_register()?;
        Ok(generator(rd, rs1, rs2).into())
    }

    #[inline(always)]
    pub(super) fn itype_instr<F>(parser: &mut Parser, generator: F) -> InstructionParserResult
    where
        F: Fn(Register, Register, Immediate) -> RawInstruction,
    {
        let rd = parser.require_register()?;
        let rs1 = parser.require_register()?;
        let imm = parser.require_immediate(3)?;
        Ok(generator(rd, rs1, imm).into())
    }

    #[inline(always)]
    pub(super) fn jtype_instr<F>(parser: &mut Parser, generator: F) -> InstructionParserResult
    where
        F: Fn(Register, Immediate) -> RawInstruction,
    {
        let rd = parser.require_register()?;
        let imm = parser.require_immediate(9)?;
        Ok(generator(rd, imm).into())
    }

    #[inline(always)]
    pub(super) fn btype_instr<F>(
        pc: Location,
        parser: &mut Parser,
        generator: F,
    ) -> InstructionParserResult
    where
        F: Fn(Register, Distance) -> RawInstruction,
    {
        let rd = parser.require_register()?;
        let distance = parser.require_distance_to_label(pc, 6)?;
        Ok(generator(rd, distance).into())
    }

    #[inline(always)]
    pub(super) fn mem_instr<F>(parser: &mut Parser, generator: F) -> InstructionParserResult
    where
        F: Fn(Register, Register, Immediate) -> RawInstruction,
    {
        let source_or_dest_reg = parser.require_register()?;
        let address_reg = parser.require_register()?;
        let addr_offset = parser.require_immediate(6)?;
        Ok(generator(source_or_dest_reg, address_reg, addr_offset).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn or_instr() {
        assert_eq!(
            RawInstruction::Small(0b010_000_100_001_0000),
            gen_or(
                Register::mock(0b001),
                Register::mock(0b010),
                Register::mock(0b100)
            ),
        );
    }
    #[test]
    fn xor_instr() {
        assert_eq!(
            RawInstruction::Small(0b010_001_100_001_0000),
            gen_xor(
                Register::mock(0b001),
                Register::mock(0b010),
                Register::mock(0b100)
            ),
        );
    }
    #[test]
    fn and_instr() {
        assert_eq!(
            RawInstruction::Small(0b010_010_100_001_0000),
            gen_and(
                Register::mock(0b001),
                Register::mock(0b010),
                Register::mock(0b100)
            ),
        );
    }
}
