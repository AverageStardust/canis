use std::collections::{HashMap, hash_map::Entry};

mod hardware;

use anyhow::Result;

use crate::processes::assemble::{
    LabelRegistry,
    diagnostics::{self, Annotatable},
    error::Diagnostic,
    line_span::SpannedLine,
};

type InstrParserResult = Result<Vec<RawInstruction>, ParseError>;
type InstrParser = fn(usize, &mut parser::Parser<'_>) -> InstrParserResult;

pub struct InstructionParserRegistry {
    parsers: HashMap<&'static str, InstrParser>,
}

impl InstructionParserRegistry {
    pub fn initialize() -> Result<Self> {
        let mut parsers = HashMap::new();
        for Instruction { name, parser } in inventory::iter::<Instruction> {
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
        pc: usize,
        instr_block: SpannedLine<'_>,
    ) -> Result<InstrVec, ParseError> {
        let (instr_name, remainder) = instr_block.break_and_trim();
        if instr_name.is_empty() {
            return Ok(InstrVec(vec![]));
        }
        let instr_parser = self
            .parsers
            .get(instr_name.as_str())
            .ok_or(ParseError::UnknownInstruction(instr_name.as_span().into()))?;
        let mut parser = parser::Parser::new(labels, remainder);
        let instructions = instr_parser(pc, &mut parser)?;
        parser.require_empty()?;
        Ok(InstrVec(instructions))
    }
}

mod parser {
    use super::*;

    pub(super) struct Parser<'a> {
        label_registry: &'a LabelRegistry<'a>,
        src: SpannedLine<'a>,
    }

    impl<'a> Parser<'a> {
        pub(super) fn new(labels: &'a LabelRegistry<'a>, remainder: SpannedLine<'a>) -> Self {
            Self {
                label_registry: labels,
                src: remainder,
            }
        }

        fn bump(&mut self) -> SpannedLine<'a> {
            let (val, rest) = self.src.break_and_trim();
            self.src = rest;
            val
        }

        pub(super) fn require_register(&mut self) -> Result<u16, ParseError> {
            let reg_val = self.bump();
            parse_register(reg_val)
        }

        // fn require(&mut self, types: ParserTypes) -> ParsedType {}

        pub(super) fn require_empty(self) -> Result<(), ParseError> {
            match self.src.collapse() {
                Some(span) => Err(ParseError::ExcessContent(span.trim().as_span().into())),
                None => Ok(()),
            }
        }
    }

    fn parse_register(reg_val: SpannedLine<'_>) -> Result<u16, ParseError> {
        match reg_val.collapse() {
            Some(reg) => match reg.as_str() {
                "ra" | "x0" => Ok(0),
                "sp" | "x1" => Ok(1),
                "t0" | "x2" => Ok(2),
                "t1" | "x3" => Ok(3),
                "s0" | "x4" => Ok(4),
                "s1" | "x5" => Ok(5),
                "a0" | "x6" => Ok(6),
                "a1" | "x7" => Ok(7),
                _ => Err(ParseError::NotRegister(reg_val.as_span().into())),
            },
            None => {
                let span = reg_val.trim().as_span();
                Err(ParseError::MissingRegister(diagnostics::LineChar::new(
                    span.line(),
                    span.start(),
                )))
            }
        }
    }
}

pub struct Instruction {
    name: &'static str,
    parser: InstrParser,
}

impl Instruction {
    const fn new(name: &'static str, parser: InstrParser) -> Self {
        Self { name, parser }
    }
}

inventory::collect!(Instruction);

pub enum RawInstruction {
    Small(u16),
    Large(u32),
}

impl RawInstruction {
    fn r_type(rs1: u16, fun3: u16, rs2: u16, rd: u16, op: u16) -> Self {
        let mut instr = 0u16;
        instr |= op & 0b0000000000001111;
        instr |= (rd << 4) & 0b0000000001110000;
        instr |= (rs2 << 7) & 0b0000001110000000;
        instr |= (fun3 << 10) & 0b0001110000000000;
        instr |= (rs1 << 13) & 0b1110000000000000;
        Self::Small(instr)
    }

    pub fn get_size(&self) -> usize {
        match self {
            Self::Small(_) => 1,
            Self::Large(_) => 2,
        }
    }
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

impl InstrVec {
    pub fn get_size(&self) -> usize {
        self.0.iter().map(|instr| instr.get_size()).sum()
    }
}

#[repr(transparent)]
pub struct InstrVec(Vec<RawInstruction>);

impl IntoIterator for InstrVec {
    type Item = RawInstruction;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

bitflags::bitflags! {
    #[derive(Debug)]
    pub struct ParserTypes: u8 {
        const Register = 0b00000001;
        const Immediate = 0b00000010;
        const UImmediate = 0b00000100;
        const Label = 0b00001000;
    }
}

enum ParsedType {
    Register(u16),
    Immediate(i16),
    UImmediate(u16),
    Label(usize),
}

#[derive(Debug)]
pub enum ParseError {
    // Instruction Errors
    /// The given instruction could not be identified
    UnknownInstruction(diagnostics::LineSpan),

    // Register Errors
    /// A register was expected, but no content could be found
    MissingRegister(diagnostics::LineChar),
    /// A register was expected, but the found content did not match any known register
    NotRegister(diagnostics::LineSpan),

    // Immediate Errors
    /// An immediate value was expected, but no content could be found
    MissingImmediate(diagnostics::LineChar),
    /// An immediate value was expected, but the found content was not a valid immediate
    NotImmediate(diagnostics::LineSpan),
    /// An unsigned immediate value was expected, but the found content was not a valid unsigned immediate
    NotUnsignedImmediate(diagnostics::LineSpan),
    /// An immediate value with a certain number of bits was expected, and too many were given
    OversizedImmediate {
        span: diagnostics::LineSpan,
        expected_bits: u8,
    },

    // Label Errors
    /// A label was expected, but no content could be found
    MissingLabel(diagnostics::LineChar),
    /// A label was expected, but the found content did not match any known label
    UnknownLabel(diagnostics::LineSpan),
    /// A label was expected, but the found content could never be a label
    NotLabel(diagnostics::LineSpan),
    /// A label was both expected and found, but is too far from the actual label's location to jump
    DistantLabel(diagnostics::CausalMultiLineSpan),

    // Mixed values
    /// A mixed value was expected, but not content could be found
    MissingValue(diagnostics::LineChar, ParserTypes),
    /// A mixed value was expected, but the found content did not match any possible value
    NotValue(diagnostics::LineChar, ParserTypes),

    // Other Errors
    /// Excess content was found in the current line
    ExcessContent(diagnostics::LineSpan),
}

impl Diagnostic for ParseError {
    fn code(&self) -> usize {
        match self {
            Self::UnknownInstruction(_) => 200,
            Self::MissingRegister(_) => 201,
            Self::NotRegister(_) => 202,
            Self::MissingImmediate(_) => 203,
            Self::NotImmediate(_) => 204,
            Self::NotUnsignedImmediate(_) => 205,
            Self::OversizedImmediate { .. } => 206,
            Self::MissingLabel(_) => 207,
            Self::UnknownLabel(_) => 208,
            Self::NotLabel(_) => 209,
            Self::DistantLabel(_) => 210,
            Self::MissingValue(_, _) => 211,
            Self::NotValue(_, _) => 212,
            Self::ExcessContent(_) => 213,
        }
    }

    fn overview(&self) -> String {
        match self {
            Self::UnknownInstruction(_) => "unknown instruction found",
            Self::MissingRegister(_) => "missing register argument",
            Self::NotRegister(_) => "invalid register",
            Self::MissingImmediate(_) => "missing immediate argument",
            Self::NotImmediate(_) => "invalid immediate",
            Self::NotUnsignedImmediate(_) => "invalid unsigned immediate",
            Self::OversizedImmediate { .. } => "immediate too large",
            Self::MissingLabel(_) => "missing label argument",
            Self::UnknownLabel(_) => "unknown label",
            Self::NotLabel(_) => "invalid label",
            Self::DistantLabel(_) => "label too far",
            Self::MissingValue(_, _) => "missing argument",
            Self::NotValue(_, _) => "invalid argument",
            Self::ExcessContent(_) => "excess values found",
        }
        .to_string()
    }

    fn annotation(&self, source: &str) -> Option<diagnostics::Annotation> {
        match self {
            Self::UnknownInstruction(span) => span.annotate(
                source,
                [|instr| format!("The instruction \"{instr}\" does not exist")],
                None,
            ),
            Self::MissingRegister(span) => {
                span.annotate(source, ["Expected a register argument here"], None)
            }
            Self::NotRegister(span) => span.annotate(
                source,
                [|reg| format!("The register \"{reg}\" does not exist")],
                Some("Valid registers are: ra, sp, t0, t1, s0, s1, a0, a1, or x0-7."),
            ),
            Self::MissingImmediate(span) => {
                span.annotate(source, ["Expected an immediate value here"], None)
            }
            Self::NotImmediate(span) => span.annotate(
                source,
                ["This is not a valid immediate value"],
                Some(
                    "Immediate values may be written with signed decimal,
                        binary, octal, or hexadecimal.",
                ),
            ),

            Self::ExcessContent(span) => {
                span.annotate(source, ["Unexpected values found here"], None)
            }
            _ => None,
        }
    }
}
