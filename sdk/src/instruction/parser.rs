use std::{
    num::{IntErrorKind, ParseIntError},
    ops::BitOrAssign,
};

use crate::{
    assemble::{
        error::{
            Diagnostic,
            annotations::{self, Annotatable},
        },
        labels::LabelRegistry,
        line_span::{Span, SpannedLine},
    },
    instruction::types::{Immediate, Location, Register},
};

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

    pub(super) fn require_register(&mut self) -> Result<Register, ParseError> {
        let reg_val = self.bump();
        parse_register(reg_val)
    }

    pub(super) fn require_immediate(&mut self, expected_bits: u8) -> Result<Immediate, ParseError> {
        let imm_val = self.bump();
        parse_immediate(imm_val, expected_bits)
    }

    pub(super) fn require_unsigned_immediate(
        &mut self,
        expected_bits: u8,
    ) -> Result<Immediate, ParseError> {
        let imm_val = self.bump();
        parse_unsigned_immediate(imm_val, expected_bits)
    }

    pub(super) fn require_label(&mut self) -> Result<Location, ParseError> {
        let label_val = self.bump();
        parse_label_location(label_val, self.label_registry)
    }

    // fn require(&mut self, types: ParserTypes) -> ParsedType {}

    pub(super) fn require_empty(self) -> Result<(), ParseError> {
        match self.src.collapse() {
            Some(span) => Err(ParseError::ExcessContent(span.trim().as_span().into())),
            None => Ok(()),
        }
    }
}

fn parse_register(reg_val: SpannedLine<'_>) -> Result<Register, ParseError> {
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
            Err(ParseError::MissingRegister(annotations::LineChar::new(
                span.line(),
                span.start(),
            )))
        }
    }
    .map(Register::from)
}

fn parse_immediate(imm_val: SpannedLine<'_>, expected_bits: u8) -> Result<Immediate, ParseError> {
    let Some(imm_val) = imm_val.collapse() else {
        let span = imm_val.trim().as_span();
        return Err(ParseError::MissingImmediate(annotations::LineChar::new(
            span.line(),
            span.start(),
        )));
    };

    let radix = match imm_val.take(2).as_str() {
        "0b" => 2,
        "0o" => 8,
        "0x" => 16,
        _ => 10,
    };

    let res = if radix != 10 {
        parse_and_restrict_unsigned(imm_val, expected_bits, radix)
    } else {
        i16::from_str_radix(imm_val.as_str(), radix)
            .map_err(|err| handle_parse_int_error(err, expected_bits, imm_val.as_span()))
            .and_then(|value| {
                if expected_bits < 16 {
                    let max = (0b1i16 << (expected_bits - 1)) - 1;
                    let min = !max;
                    if value > max || value < min {
                        Err(ParseError::OversizedImmediate {
                            span: imm_val.as_span().into(),
                            expected_bits,
                        })
                    } else {
                        Ok(value as u16)
                    }
                } else {
                    Ok(value as u16)
                }
            })
    };

    res.map(Immediate::from)
}

fn parse_unsigned_immediate(
    imm_val: SpannedLine<'_>,
    expected_bits: u8,
) -> Result<Immediate, ParseError> {
    let Some(imm_val) = imm_val.collapse() else {
        let span = imm_val.trim().as_span();
        return Err(ParseError::MissingImmediate(annotations::LineChar::new(
            span.line(),
            span.start(),
        )));
    };

    let radix = match imm_val.take(2).as_str() {
        "0b" => 2,
        "0o" => 8,
        "0x" => 16,
        _ => 10,
    };

    parse_and_restrict_unsigned(imm_val, expected_bits, radix).map(Immediate::from)
}

fn handle_parse_int_error(err: ParseIntError, expected_bits: u8, imm_span: Span) -> ParseError {
    match err.kind() {
        // Shouldn't be possible, but still accounted for
        IntErrorKind::Empty => ParseError::MissingImmediate(annotations::LineChar::new(
            imm_span.line(),
            imm_span.start(),
        )),
        IntErrorKind::InvalidDigit => ParseError::NotImmediate(imm_span.into()),
        IntErrorKind::PosOverflow => ParseError::OversizedImmediate {
            span: imm_span.into(),
            expected_bits,
        },
        IntErrorKind::NegOverflow => ParseError::OversizedImmediate {
            span: imm_span.into(),
            expected_bits,
        },
        IntErrorKind::Zero => unreachable!("should never result in a Zero error"),
        _ => panic!("unexpected int parsing error encountered"),
    }
}

fn parse_and_restrict_unsigned(
    imm_val: SpannedLine<'_>,
    expected_bits: u8,
    radix: u32,
) -> Result<u16, ParseError> {
    let imm_str = if radix == 10 {
        imm_val.as_str()
    } else {
        imm_val.skip(2).as_str()
    };
    u16::from_str_radix(imm_str, radix)
        .map_err(|err| handle_parse_int_error(err, expected_bits, imm_val.as_span()))
        .and_then(|value| {
            if expected_bits < 16 {
                let max = (0b1u16 << (expected_bits)) - 1;
                if value > max {
                    Err(ParseError::OversizedImmediate {
                        span: imm_val.as_span().into(),
                        expected_bits,
                    })
                } else {
                    Ok(value)
                }
            } else {
                Ok(value)
            }
        })
}

fn parse_label_location(
    label_val: SpannedLine<'_>,
    labels: &LabelRegistry<'_>,
) -> Result<Location, ParseError> {
    if let Some(label_span) = label_val.collapse()
        && let mut chars = label_span.as_str().chars()
        && let Some(prefix) = chars.next()
    {
        let span = label_span.as_span();
        if (!prefix.is_ascii_alphabetic() && prefix != '_')
            || chars.any(|ch| !ch.is_ascii_alphanumeric() && ch != '_')
        {
            Err(ParseError::NotLabel(annotations::LineSpan::new(
                span.line(),
                span.start(),
                span.end(),
            )))
        } else {
            labels
                .get_label_location(label_span.as_str())
                .ok_or(ParseError::UnknownLabel(span.into()))
        }
    } else {
        let span = label_val.trim().as_span();
        Err(ParseError::MissingLabel(annotations::LineChar::new(
            span.line(),
            span.start(),
        )))
    }
}

bitflags::bitflags! {
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub struct ParserTypes: u8 {
        const Register = 0b00000001;
        const Immediate = 0b00000010;
        const UImmediate = 0b00000100;
        const Label = 0b00001000;
    }
}

impl ParserTypes {
    pub fn is_immediate(&self) -> bool {
        self.intersects(ParserTypes::Immediate | ParserTypes::UImmediate)
    }

    pub(crate) fn with_expected_bits(self, expected_bits: u8) -> Self {
        Self::from_bits_retain((self.bits() & 0b00001111) | (expected_bits.saturating_sub(1) << 4))
    }

    pub fn expected_bits(&self) -> u8 {
        (self.bits() >> 4) + 1
    }
}

enum ParsedType {
    Register(Register),
    Immediate(Immediate),
    UImmediate(Immediate),
    Label(Location),
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    // Instruction Errors
    /// The given instruction could not be identified
    UnknownInstruction(annotations::LineSpan),

    // Register Errors
    /// A register was expected, but no content could be found
    MissingRegister(annotations::LineChar),
    /// A register was expected, but the found content did not match any known register
    NotRegister(annotations::LineSpan),

    // Immediate Errors
    /// An immediate value was expected, but no content could be found
    MissingImmediate(annotations::LineChar),
    /// An immediate value was expected, but the found content was not a valid immediate
    NotImmediate(annotations::LineSpan),
    /// An unsigned immediate value was expected, but the found content was not a valid unsigned immediate
    NotUnsignedImmediate(annotations::LineSpan),
    /// An immediate value with a certain number of bits was expected, and too many were given
    OversizedImmediate {
        span: annotations::LineSpan,
        expected_bits: u8,
    },

    // Label Errors
    /// A label was expected, but no content could be found
    MissingLabel(annotations::LineChar),
    /// A label was expected, but the found content did not match any known label
    UnknownLabel(annotations::LineSpan),
    /// A label was expected, but the found content could never be a label
    NotLabel(annotations::LineSpan),
    /// A label was both expected and found, but is too far from the actual label's location to jump
    DistantLabel(annotations::CausalMultiLineSpan),

    // Mixed values
    /// A mixed value was expected, but not content could be found
    MissingValue(annotations::LineChar, ParserTypes),
    /// A mixed value was expected, but the found content did not match any possible value
    NotValue(annotations::LineChar, ParserTypes),

    // Other Errors
    /// Excess content was found in the current line
    ExcessContent(annotations::LineSpan),
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

    fn annotation(&self, source: &str) -> Option<annotations::Annotation> {
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
            Self::NotUnsignedImmediate(span) => span.annotate(
                source,
                ["This is not a valid unsigned immediate value"],
                Some(
                    "Unsigned immediate values may be written in decimal,
                    binary, octal, or hexadecimal.",
                ),
            ),
            Self::OversizedImmediate {
                span,
                expected_bits,
            } => span.annotate(
                source,
                [|imm| format!("The immediate \"{imm}\" does not fit in {expected_bits} bits")],
                Some("Some instructions can have larger immediates than others."),
            ),

            Self::ExcessContent(span) => {
                span.annotate(source, ["Unexpected values found here"], None)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assemble::error::annotations::LineSpan;

    use super::*;
    #[test]
    fn registers() {
        let mut parser = Parser::new(
            &LabelRegistry::Incomplete,
            SpannedLine::mock("x0 x1 x2 x3 x4 x5 x6 x7 ra sp t0 t1 s0 s1 a0 a1"),
        );

        assert_eq!(Ok(Register::mock(0)), parser.require_register());
        assert_eq!(Ok(Register::mock(1)), parser.require_register());
        assert_eq!(Ok(Register::mock(2)), parser.require_register());
        assert_eq!(Ok(Register::mock(3)), parser.require_register());
        assert_eq!(Ok(Register::mock(4)), parser.require_register());
        assert_eq!(Ok(Register::mock(5)), parser.require_register());
        assert_eq!(Ok(Register::mock(6)), parser.require_register());
        assert_eq!(Ok(Register::mock(7)), parser.require_register());
        assert_eq!(Ok(Register::mock(0)), parser.require_register());
        assert_eq!(Ok(Register::mock(1)), parser.require_register());
        assert_eq!(Ok(Register::mock(2)), parser.require_register());
        assert_eq!(Ok(Register::mock(3)), parser.require_register());
        assert_eq!(Ok(Register::mock(4)), parser.require_register());
        assert_eq!(Ok(Register::mock(5)), parser.require_register());
        assert_eq!(Ok(Register::mock(6)), parser.require_register());
        assert_eq!(Ok(Register::mock(7)), parser.require_register());
        assert_eq!(Ok(()), parser.require_empty())
    }

    #[test]
    fn not_registers() {
        let mut parser = Parser::new(
            &LabelRegistry::Incomplete,
            SpannedLine::mock("x0x x8 x10 rax spx t0x t2 t10 s0x s2 s10 a0x a2 a10"),
        );

        assert_eq!(
            Err(ParseError::NotRegister(LineSpan::new(0, 0, 3))),
            parser.require_register()
        );
        assert_eq!(
            Err(ParseError::NotRegister(LineSpan::new(0, 4, 6))),
            parser.require_register()
        );
        assert_eq!(
            Err(ParseError::NotRegister(LineSpan::new(0, 7, 10))),
            parser.require_register()
        );
        assert_eq!(
            Err(ParseError::NotRegister(LineSpan::new(0, 11, 14))),
            parser.require_register()
        );
        assert_eq!(
            Err(ParseError::NotRegister(LineSpan::new(0, 15, 18))),
            parser.require_register()
        );
        assert_eq!(
            Err(ParseError::NotRegister(LineSpan::new(0, 19, 22))),
            parser.require_register()
        );
        assert_eq!(
            Err(ParseError::NotRegister(LineSpan::new(0, 23, 25))),
            parser.require_register()
        );
        assert_eq!(
            Err(ParseError::NotRegister(LineSpan::new(0, 26, 29))),
            parser.require_register()
        );
        assert_eq!(
            Err(ParseError::NotRegister(LineSpan::new(0, 30, 33))),
            parser.require_register()
        );
        assert_eq!(
            Err(ParseError::NotRegister(LineSpan::new(0, 34, 36))),
            parser.require_register()
        );
        assert_eq!(
            Err(ParseError::NotRegister(LineSpan::new(0, 37, 40))),
            parser.require_register()
        );
        assert_eq!(
            Err(ParseError::NotRegister(LineSpan::new(0, 41, 44))),
            parser.require_register()
        );
        assert_eq!(
            Err(ParseError::NotRegister(LineSpan::new(0, 45, 47))),
            parser.require_register()
        );
        assert_eq!(
            Err(ParseError::NotRegister(LineSpan::new(0, 48, 51))),
            parser.require_register()
        );
        assert_eq!(Ok(()), parser.require_empty())
    }

    #[test]
    fn parser_types() {
        let types = ParserTypes::Register | ParserTypes::UImmediate;
        assert_eq!(0b00000101, types.bits());
        let with_expected = types.with_expected_bits(10);
        assert_eq!(0b10010101, with_expected.bits());
        assert_eq!(10, with_expected.expected_bits());
        let with_expected = types.with_expected_bits(0);
        assert_eq!(0b00000101, with_expected.bits());
        assert_eq!(1, with_expected.expected_bits());
    }
}
