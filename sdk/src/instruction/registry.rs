use std::collections::{HashMap, hash_map::Entry};

use anyhow::Result;

use crate::{
    assemble::{labels::LabelRegistry, line_span::SpannedLine},
    instruction::{
        instruction::{Instruction, InstructionMeta},
        parser::{self, ParseError},
        raw::{RawInstruction, RawInstructionVec},
        types::Location,
    },
};

pub(super) type InstructionParserResult = Result<Vec<RawInstruction>, ParseError>;
pub(super) type InstructionParser =
    fn(Location, &mut parser::Parser<'_>) -> InstructionParserResult;

pub(crate) struct InstructionParserRegistry {
    parsers: HashMap<&'static str, InstructionParser>,
}

inventory::collect!(Instruction);

impl InstructionParserRegistry {
    pub fn initialize() -> Result<Self> {
        let mut parsers = HashMap::new();
        for Instruction { name, parser, .. } in inventory::iter::<Instruction> {
            match parsers.entry(*name) {
                Entry::Vacant(vacant) => {
                    vacant.insert(parser.clone());
                }
                Entry::Occupied(occupied) => {
                    anyhow::bail!(
                        "error initializing instruction parser registry: instruction name `{}` already taken",
                        occupied.key()
                    )
                }
            }
        }
        Ok(Self { parsers })
    }

    pub fn parse_instruction(
        &self,
        labels: &LabelRegistry<'_>,
        pc: Location,
        instr_block: SpannedLine<'_>,
    ) -> Result<RawInstructionVec, ParseError> {
        let (instr_name, remainder) = instr_block.break_and_trim();
        if instr_name.is_empty() {
            return Ok(Vec::new().into());
        }
        let instr_parser = self
            .parsers
            .get(instr_name.as_str())
            .ok_or(ParseError::UnknownInstruction(instr_name.as_span().into()))?;
        let mut parser = parser::Parser::new(labels, remainder);
        let instructions = instr_parser(pc, &mut parser)?;
        parser.require_empty()?;
        Ok(instructions.into())
    }
}

pub fn get_instruction_meta() -> Vec<&'static InstructionMeta> {
    let mut meta = Vec::new();
    for instruction in inventory::iter::<Instruction>() {
        meta.push(&instruction.meta);
    }
    meta
}
