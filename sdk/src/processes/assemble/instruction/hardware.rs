use super::{InstrParserResult, Instruction, RawInstruction, parser::Parser};

pub fn gen_add(rd: u16, rs1: u16, rs2: u16) -> Vec<RawInstruction> {
    RawInstruction::r_type(rs1, 0b000, rs2, rd, 0b0000).into()
}

fn instr_add(_pc: usize, parser: &mut Parser) -> InstrParserResult {
    let rd = parser.require_register()?;
    let rs1 = parser.require_register()?;
    let rs2 = parser.require_register()?;
    Ok(gen_add(rd, rs1, rs2))
}

inventory::submit! {
    Instruction::new("add", instr_add)
}
